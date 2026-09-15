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

#[must_use]
pub fn validate_basic_pair(photo: &Path, motion: &Path) -> ValidationReport {
    let mut issues = Vec::new();
    check_resource(photo, &["heic", "heif", "jpg", "jpeg"], "photo", &mut issues);
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

fn check_resource(path: &Path, extensions: &[&str], label: &str, issues: &mut Vec<ValidationIssue>) {
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
