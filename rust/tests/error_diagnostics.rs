//! T013: diagnostics must be enumerable, bounded, and never interpolate
//! user-provided image paths, YAML text, or decoded bytes into logs.
use std::{io::ErrorKind, num::NonZeroUsize};
use yuauy_wallpaper::{
    diagnostics::{DiagnosticCode, DiagnosticLog},
    error::{AppError, NativeOperation},
};

#[test]
fn yaml_failure_and_unavailable_load_are_distinct() {
    let yaml = AppError::InvalidYaml;
    let unknown = AppError::UnknownLoad;
    assert_ne!(yaml, unknown);
    assert_ne!(yaml.to_string(), unknown.to_string());
    assert_eq!(DiagnosticCode::from(&yaml), DiagnosticCode::InvalidYaml);
    assert_eq!(DiagnosticCode::from(&unknown), DiagnosticCode::UnknownLoad);
}

#[test]
fn native_errors_are_typed_codes_not_arbitrary_messages() {
    let error = AppError::Native {
        operation: NativeOperation::DxgiPresent,
        hresult: 0x887A_0005_u32 as i32,
    };
    let output = error.to_string();
    assert!(output.contains("DxgiPresent"));
    assert!(output.contains("887a0005"));
    assert!(!output.contains("C:\\Users"));
    assert_eq!(DiagnosticCode::from(&error), DiagnosticCode::NativeFailure);
}

#[test]
fn filesystem_errors_log_error_kinds_not_paths() {
    let private_filename = r"C:\Users\Private\Pictures\photo.png";
    let error = AppError::Filesystem(ErrorKind::PermissionDenied);
    let displayed = error.to_string();
    assert!(displayed.contains("PermissionDenied"));
    assert!(!displayed.contains(private_filename));
    assert!(!displayed.contains("photo.png"));
}

#[test]
fn diagnostic_log_is_bounded_and_does_not_capture_user_payloads() {
    let mut log = DiagnosticLog::new(NonZeroUsize::new(2).unwrap());
    log.record(DiagnosticCode::NoImages);
    log.record(DiagnosticCode::UnknownLoad);
    log.record(DiagnosticCode::InvalidYaml);
    assert_eq!(
        log.events(),
        &[DiagnosticCode::UnknownLoad, DiagnosticCode::InvalidYaml]
    );
    let formatted = log.summary();
    assert!(formatted.contains("InvalidYaml"));
    assert!(!formatted.contains("C:\\"));
    assert!(!formatted.contains("JPG"));
}

#[test]
fn diagnostic_empty_state_is_explicit() {
    let log = DiagnosticLog::new(NonZeroUsize::new(4).unwrap());
    assert_eq!(log.events(), &[]);
    assert_eq!(log.summary(), "no diagnostic events");
}
