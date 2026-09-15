use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const QUICKTIME_CONTENT_IDENTIFIER: &str = "com.apple.quicktime.content.identifier";
pub const QUICKTIME_STILL_IMAGE_TIME: &str = "com.apple.quicktime.still-image-time";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePhotoIdentity {
    pub asset_identifier: String,
}

impl LivePhotoIdentity {
    #[must_use]
    pub fn generate() -> Self {
        Self {
            asset_identifier: Uuid::new_v4().hyphenated().to_string().to_uppercase(),
        }
    }

    #[must_use]
    pub fn from_identifier(identifier: impl Into<String>) -> Self {
        Self {
            asset_identifier: identifier.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataPlan {
    pub identity: LivePhotoIdentity,
    pub still_image_time_seconds: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_identifier_is_a_uuid() {
        let identity = LivePhotoIdentity::generate();
        assert!(Uuid::parse_str(&identity.asset_identifier).is_ok());
    }
}
