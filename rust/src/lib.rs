//! YUAUY Wallpaper's testable library boundary.
//!
//! Product baseline: Windows 11 24H2 or newer, x64 MSVC, Direct3D 11.
//! Animation, image selection, scheduling, and Windows adapters belong to later
//! Spec Kit tasks. This setup scaffold performs no OS or photo operations.

pub mod diagnostics;
pub mod error;
pub mod monitor;
pub mod pacing;
pub mod platform;
pub mod usage_probe;
pub mod workerw_probe_options;
