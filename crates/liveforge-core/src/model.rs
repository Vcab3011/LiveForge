use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaInfo {
    pub source: PathBuf,
    pub duration_seconds: f64,
    pub width: u32,
    pub height: u32,
    pub frames_per_second: f64,
    pub container: String,
    pub video_codec: String,
    pub pixel_format: Option<String>,
    pub bit_depth: Option<u8>,
    pub color_primaries: Option<String>,
    pub color_transfer: Option<String>,
    pub color_space: Option<String>,
    pub audio_codec: Option<String>,
}

impl MediaInfo {
    #[must_use]
    pub fn hdr_kind(&self) -> HdrKind {
        let transfer = self.color_transfer.as_deref().unwrap_or_default();
        match transfer {
            "smpte2084" => HdrKind::Hdr10,
            "arib-std-b67" => HdrKind::Hlg,
            _ if self.video_codec.contains("dovi") => HdrKind::DolbyVision,
            _ => HdrKind::SdrOrUnknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HdrKind {
    SdrOrUnknown,
    Hdr10,
    Hlg,
    DolbyVision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityMode {
    Original,
    MaximumCompatibility,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversionOptions {
    pub key_time_seconds: f64,
    pub duration_seconds: f64,
    pub quality: QualityMode,
    pub preserve_audio: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingStrategy {
    RemuxCopy,
    TranscodeOnce,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversionPlan {
    pub clip_start_seconds: f64,
    pub key_time_in_clip_seconds: f64,
    pub clip_duration_seconds: f64,
    pub strategy: ProcessingStrategy,
    pub source_hdr: HdrKind,
    pub preserve_audio: bool,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}
