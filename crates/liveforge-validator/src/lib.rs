use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationLevel {
    Basic,
    Structural,
    Decode,
    AppleNative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub passed: bool,
    pub highest_completed_level: ValidationLevel,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuralMetadataSnapshot {
    pub photo_asset_identifier: Option<String>,
    pub motion_content_identifier: Option<String>,
    pub still_image_time_seconds: Option<f64>,
    pub motion_duration_seconds: Option<f64>,
}

#[must_use]
pub fn validate_structural_snapshot(snapshot: &StructuralMetadataSnapshot) -> ValidationReport {
    let mut issues = Vec::new();

    required_value(
        snapshot.photo_asset_identifier.as_deref(),
        "MISSING_PHOTO_IDENTIFIER",
        "the still resource has no asset identifier",
        &mut issues,
    );
    required_value(
        snapshot.motion_content_identifier.as_deref(),
        "MISSING_MOTION_IDENTIFIER",
        "the motion resource has no content identifier",
        &mut issues,
    );

    let identifiers_mismatch = matches!(
        (
            snapshot.photo_asset_identifier.as_deref(),
            snapshot.motion_content_identifier.as_deref(),
        ),
        (Some(photo), Some(motion)) if photo != motion
    );
    if identifiers_mismatch {
        push_error(
            &mut issues,
            "IDENTIFIER_MISMATCH",
            "the still and motion resource identifiers do not match",
        );
    }

    if snapshot.still_image_time_seconds.is_none() {
        push_error(
            &mut issues,
            "MISSING_STILL_IMAGE_TIME",
            "the motion resource has no timed still-image metadata sample",
        );
    }
    if snapshot.motion_duration_seconds.is_none() {
        push_error(
            &mut issues,
            "MISSING_MOTION_DURATION",
            "the motion duration is unavailable",
        );
    }
    let time_out_of_range = match (
        snapshot.still_image_time_seconds,
        snapshot.motion_duration_seconds,
    ) {
        (Some(time), Some(duration)) => {
            !time.is_finite()
                || !duration.is_finite()
                || duration <= 0.0
                || !(0.0..=duration).contains(&time)
        }
        _ => false,
    };
    if time_out_of_range {
        push_error(
            &mut issues,
            "STILL_IMAGE_TIME_OUT_OF_RANGE",
            "the still-image timestamp is not inside the motion resource",
        );
    }

    ValidationReport {
        passed: !issues.iter().any(|issue| issue.severity == Severity::Error),
        highest_completed_level: ValidationLevel::Structural,
        issues,
    }
}

#[must_use]
pub fn validate_basic_pair(photo: &Path, motion: &Path) -> ValidationReport {
    let mut issues = Vec::new();
    check_resource(
        photo,
        &["heic", "heif", "jpg", "jpeg"],
        "photo",
        &mut issues,
    );
    check_resource(motion, &["mov"], "motion", &mut issues);

    issues.push(ValidationIssue {
        severity: Severity::Warning,
        code: "STRUCTURAL_VALIDATION_PENDING".into(),
        message: "asset identifiers and still-image-time metadata were not inspected".into(),
    });

    ValidationReport {
        passed: !issues.iter().any(|issue| issue.severity == Severity::Error),
        highest_completed_level: ValidationLevel::Basic,
        issues,
    }
}

fn check_resource(
    path: &Path,
    extensions: &[&str],
    label: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if !path.is_file() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            code: format!("MISSING_{}", label.to_uppercase()),
            message: format!("{} resource does not exist: {}", label, path.display()),
        });
        return;
    }

    if fs::metadata(path).map_or(true, |metadata| metadata.len() == 0) {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            code: format!("EMPTY_{}", label.to_uppercase()),
            message: format!("{} resource is empty: {}", label, path.display()),
        });
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !extensions.contains(&extension.as_str()) {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            code: format!("UNEXPECTED_{}_EXTENSION", label.to_uppercase()),
            message: format!("unexpected {} extension: .{}", label, extension),
        });
    }
}

fn required_value(
    value: Option<&str>,
    code: &str,
    message: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if value.is_none_or(str::is_empty) {
        push_error(issues, code, message);
    }
}

fn push_error(issues: &mut Vec<ValidationIssue>, code: &str, message: &str) {
    issues.push(ValidationIssue {
        severity: Severity::Error,
        code: code.into(),
        message: message.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_snapshot() -> StructuralMetadataSnapshot {
        StructuralMetadataSnapshot {
            photo_asset_identifier: Some("A-SHARED-ID".into()),
            motion_content_identifier: Some("A-SHARED-ID".into()),
            still_image_time_seconds: Some(1.5),
            motion_duration_seconds: Some(3.0),
        }
    }

    #[test]
    fn accepts_matching_identifiers_and_in_range_timestamp() {
        let report = validate_structural_snapshot(&valid_snapshot());
        assert!(report.passed);
        assert_eq!(report.highest_completed_level, ValidationLevel::Structural);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn rejects_mismatched_identifiers() {
        let mut snapshot = valid_snapshot();
        snapshot.motion_content_identifier = Some("ANOTHER-ID".into());

        let report = validate_structural_snapshot(&snapshot);

        assert!(!report.passed);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "IDENTIFIER_MISMATCH"));
    }

    #[test]
    fn rejects_out_of_range_still_image_time() {
        let mut snapshot = valid_snapshot();
        snapshot.still_image_time_seconds = Some(3.5);

        let report = validate_structural_snapshot(&snapshot);

        assert!(!report.passed);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "STILL_IMAGE_TIME_OUT_OF_RANGE"));
    }

    #[test]
    fn reports_every_missing_required_field() {
        let report = validate_structural_snapshot(&StructuralMetadataSnapshot {
            photo_asset_identifier: None,
            motion_content_identifier: None,
            still_image_time_seconds: None,
            motion_duration_seconds: None,
        });

        assert!(!report.passed);
        assert_eq!(report.issues.len(), 4);
    }
}
