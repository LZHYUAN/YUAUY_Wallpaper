//! Typed application failures. These public variants deliberately contain
//! no user-provided source text, photo bytes, paths, or raw YAML fragments.
use std::{fmt, io::ErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeOperation {
    D3d11CreateDevice,
    DxgiPresent,
    WorkerwAttach,
    ImageDecode,
    LoadUtilization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppError {
    InvalidYaml,
    UnknownLoad,
    NoImages,
    DesktopUnavailable,
    Filesystem(ErrorKind),
    Native {
        operation: NativeOperation,
        hresult: i32,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidYaml => {
                f.write_str("invalid YAML configuration (previous settings retained)")
            }
            Self::UnknownLoad => f.write_str("GPU/CPU total utilization unavailable"),
            Self::NoImages => f.write_str("no valid images available"),
            Self::DesktopUnavailable => f.write_str("desktop shell host unavailable"),
            Self::Filesystem(kind) => write!(f, "filesystem operation failed ({kind:?})"),
            Self::Native { operation, hresult } => {
                write!(
                    f,
                    "native operation {operation:?} failed (HRESULT=0x{:08x})",
                    *hresult as u32
                )
            }
        }
    }
}

impl std::error::Error for AppError {}
