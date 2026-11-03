//! Redacted, bounded diagnostics. Only allowlisted codes are loggable;
//! callers cannot push arbitrary user text or private file paths.
use crate::error::AppError;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    NoImages,
    UnknownLoad,
    InvalidYaml,
    DesktopUnavailable,
    FilesystemFailure,
    NativeFailure,
}

impl From<&AppError> for DiagnosticCode {
    fn from(error: &AppError) -> Self {
        match error {
            AppError::InvalidYaml => Self::InvalidYaml,
            AppError::UnknownLoad => Self::UnknownLoad,
            AppError::NoImages => Self::NoImages,
            AppError::DesktopUnavailable => Self::DesktopUnavailable,
            AppError::Filesystem(_) => Self::FilesystemFailure,
            AppError::Native { .. } => Self::NativeFailure,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticLog {
    capacity: NonZeroUsize,
    events: Vec<DiagnosticCode>,
}
impl DiagnosticLog {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            events: Vec::with_capacity(capacity.get()),
        }
    }
    pub fn record(&mut self, code: DiagnosticCode) {
        if self.events.len() == self.capacity.get() {
            self.events.remove(0);
        }
        self.events.push(code);
    }
    pub fn events(&self) -> &[DiagnosticCode] {
        &self.events
    }
    pub fn summary(&self) -> String {
        if self.events.is_empty() {
            "no diagnostic events".to_string()
        } else {
            self.events
                .iter()
                .map(|code| format!("{code:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}
