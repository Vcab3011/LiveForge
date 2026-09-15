use crate::{ConversionOptions, ConversionPlan, MediaInfo, ProcessingStrategy, QualityMode};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum PlanError {
    #[error("source duration must be greater than zero")]
    InvalidSourceDuration,
    #[error("requested duration must be greater than zero")]
    InvalidClipDuration,
    #[error("key time {0:.3}s is outside the source")]
    KeyTimeOutOfRange(f64),
}

pub fn plan_conversion(
    media: &MediaInfo,
    options: &ConversionOptions,
) -> Result<ConversionPlan, PlanError> {
    if media.duration_seconds <= 0.0 {
        return Err(PlanError::InvalidSourceDuration);
    }
    if options.duration_seconds <= 0.0 {
        return Err(PlanError::InvalidClipDuration);
    }
    if !(0.0..=media.duration_seconds).contains(&options.key_time_seconds) {
        return Err(PlanError::KeyTimeOutOfRange(options.key_time_seconds));
    }

    let duration = options.duration_seconds.min(media.duration_seconds);
    let half = duration / 2.0;
    let start = (options.key_time_seconds - half)
        .max(0.0)
        .min(media.duration_seconds - duration);
    let key_in_clip = options.key_time_seconds - start;

    let video_compatible = matches!(media.video_codec.as_str(), "h264" | "hevc");
    let audio_compatible = media
        .audio_codec
        .as_deref()
        .is_none_or(|codec| codec == "aac");
    let original_requested = options.quality == QualityMode::Original;
    let can_copy = original_requested && video_compatible && audio_compatible;

    let mut reasons = Vec::new();
    let mut warnings = Vec::new();
    let strategy = if can_copy {
        reasons.push("source streams are compatible with the planned MOV resource".into());
        ProcessingStrategy::RemuxCopy
    } else {
        if !original_requested {
            reasons.push("maximum compatibility mode explicitly permits one transcode".into());
        }
        if !video_compatible {
            reasons.push(format!(
                "video codec '{}' requires compatibility conversion",
                media.video_codec
            ));
        }
        if !audio_compatible {
            reasons.push("audio must be converted to AAC or removed".into());
        }
        ProcessingStrategy::TranscodeOnce
    };

    if media.hdr_kind() != crate::HdrKind::SdrOrUnknown {
        warnings.push(
            "HDR was detected, but preservation must be confirmed by the completed pipeline".into(),
        );
    }
    if strategy == ProcessingStrategy::RemuxCopy {
        warnings.push(
            "exact trim behavior must be verified against packet/keyframe boundaries during muxing"
                .into(),
        );
    }

    Ok(ConversionPlan {
        clip_start_seconds: start,
        key_time_in_clip_seconds: key_in_clip,
        clip_duration_seconds: duration,
        strategy,
        source_hdr: media.hdr_kind(),
        preserve_audio: options.preserve_audio && media.audio_codec.is_some(),
        reasons,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn source(codec: &str, duration: f64) -> MediaInfo {
        MediaInfo {
            source: PathBuf::from("fixture.mov"),
            duration_seconds: duration,
            width: 3840,
            height: 2160,
            frames_per_second: 60.0,
            container: "mov".into(),
            video_codec: codec.into(),
            pixel_format: Some("yuv420p10le".into()),
            bit_depth: Some(10),
            color_primaries: Some("bt2020".into()),
            color_transfer: Some("arib-std-b67".into()),
            color_space: Some("bt2020nc".into()),
            audio_codec: Some("aac".into()),
        }
    }

    #[test]
    fn centers_three_second_clip_on_key_time() {
        let plan = plan_conversion(
            &source("hevc", 20.0),
            &ConversionOptions {
                key_time_seconds: 7.8,
                duration_seconds: 3.0,
                quality: QualityMode::Original,
                preserve_audio: true,
            },
        )
        .expect("valid plan");

        assert!((plan.clip_start_seconds - 6.3).abs() < f64::EPSILON);
        assert!((plan.key_time_in_clip_seconds - 1.5).abs() < f64::EPSILON);
        assert_eq!(plan.strategy, ProcessingStrategy::RemuxCopy);
    }

    #[test]
    fn clamps_clip_to_beginning() {
        let plan = plan_conversion(
            &source("h264", 20.0),
            &ConversionOptions {
                key_time_seconds: 0.4,
                duration_seconds: 3.0,
                quality: QualityMode::Original,
                preserve_audio: false,
            },
        )
        .expect("valid plan");

        assert_eq!(plan.clip_start_seconds, 0.0);
        assert_eq!(plan.key_time_in_clip_seconds, 0.4);
    }

    #[test]
    fn incompatible_codec_transcodes_once() {
        let plan = plan_conversion(
            &source("vp9", 20.0),
            &ConversionOptions {
                key_time_seconds: 10.0,
                duration_seconds: 3.0,
                quality: QualityMode::Original,
                preserve_audio: true,
            },
        )
        .expect("valid plan");

        assert_eq!(plan.strategy, ProcessingStrategy::TranscodeOnce);
    }
}
