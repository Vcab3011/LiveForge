use liveforge_core::MediaInfo;
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("input does not exist: {0}")]
    MissingInput(PathBuf),
    #[error("failed to start ffprobe; install FFmpeg and make ffprobe available on PATH: {0}")]
    Unavailable(#[source] std::io::Error),
    #[error("ffprobe failed: {0}")]
    Failed(String),
    #[error("could not parse ffprobe output: {0}")]
    InvalidOutput(#[from] serde_json::Error),
    #[error("ffprobe did not report a video stream")]
    MissingVideoStream,
    #[error("ffprobe did not report a valid duration")]
    MissingDuration,
}

#[derive(Debug, Deserialize)]
struct ProbeDocument {
    streams: Vec<ProbeStream>,
    format: ProbeFormat,
}

#[derive(Debug, Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
    format_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProbeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    avg_frame_rate: Option<String>,
    pix_fmt: Option<String>,
    bits_per_raw_sample: Option<String>,
    color_primaries: Option<String>,
    color_transfer: Option<String>,
    color_space: Option<String>,
}

pub fn probe(path: impl AsRef<Path>) -> Result<MediaInfo, ProbeError> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err(ProbeError::MissingInput(path.to_path_buf()));
    }

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration,format_name:stream=codec_type,codec_name,width,height,avg_frame_rate,pix_fmt,bits_per_raw_sample,color_primaries,color_transfer,color_space",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .map_err(ProbeError::Unavailable)?;

    if !output.status.success() {
        return Err(ProbeError::Failed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    let document: ProbeDocument = serde_json::from_slice(&output.stdout)?;
    normalize(path, document)
}

fn normalize(path: &Path, document: ProbeDocument) -> Result<MediaInfo, ProbeError> {
    let video = document
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("video"))
        .ok_or(ProbeError::MissingVideoStream)?;
    let audio = document
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"));
    let duration_seconds = document
        .format
        .duration
        .as_deref()
        .and_then(|value| value.parse().ok())
        .filter(|value: &f64| *value > 0.0)
        .ok_or(ProbeError::MissingDuration)?;

    Ok(MediaInfo {
        source: path.to_path_buf(),
        duration_seconds,
        width: video.width.unwrap_or_default(),
        height: video.height.unwrap_or_default(),
        frames_per_second: parse_rate(video.avg_frame_rate.as_deref()),
        container: document
            .format
            .format_name
            .unwrap_or_else(|| "unknown".into())
            .split(',')
            .next()
            .unwrap_or("unknown")
            .to_owned(),
        video_codec: video.codec_name.clone().unwrap_or_else(|| "unknown".into()),
        pixel_format: video.pix_fmt.clone(),
        bit_depth: video
            .bits_per_raw_sample
            .as_deref()
            .and_then(|value| value.parse().ok()),
        color_primaries: video.color_primaries.clone(),
        color_transfer: video.color_transfer.clone(),
        color_space: video.color_space.clone(),
        audio_codec: audio.and_then(|stream| stream.codec_name.clone()),
    })
}

fn parse_rate(value: Option<&str>) -> f64 {
    let Some(value) = value else {
        return 0.0;
    };
    let mut parts = value.split('/');
    let numerator = parts.next().and_then(|part| part.parse::<f64>().ok());
    let denominator = parts.next().and_then(|part| part.parse::<f64>().ok());
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) if denominator != 0.0 => numerator / denominator,
        (Some(rate), None) => rate,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_rate;

    #[test]
    fn parses_fractional_frame_rate() {
        assert!((parse_rate(Some("60000/1001")) - 59.940_059_94).abs() < 0.000_001);
    }

    #[test]
    fn handles_invalid_rate() {
        assert_eq!(parse_rate(Some("0/0")), 0.0);
        assert_eq!(parse_rate(None), 0.0);
    }
}
