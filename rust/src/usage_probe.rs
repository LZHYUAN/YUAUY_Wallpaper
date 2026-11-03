//! Read-only mathematical interpretation of one-shot system utilization data.
//! Collection is platform-specific and belongs to the separate diagnostic
//! binary; this module never starts monitoring or holds persistent handles.
use crate::platform::UsageReading;
use std::collections::HashMap;

/// Win32 GetSystemTimes reports aggregate FILETIME ticks for *all processors*.
/// Idle time is part of kernel time: do not add it twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuTimes {
    pub idle: u64,
    pub kernel: u64,
    pub user: u64,
}

pub fn cpu_busy_percent(before: CpuTimes, after: CpuTimes) -> UsageReading {
    let (Some(idle), Some(kernel), Some(user)) = (
        after.idle.checked_sub(before.idle),
        after.kernel.checked_sub(before.kernel),
        after.user.checked_sub(before.user),
    ) else {
        return UsageReading::Unknown;
    };
    if idle > kernel {
        return UsageReading::Unknown;
    }
    let Some(total) = kernel.checked_add(user) else {
        return UsageReading::Unknown;
    };
    if total == 0 {
        return UsageReading::Unknown;
    }
    let busy = 100.0 * (total - idle) as f64 / total as f64;
    UsageReading::from_percent(busy as f32)
}

/// Extract the physical GPU and engine identity from GPU Engine PDH instances
/// while ignoring PID. PDH exposes per-process *contributions* to physical
/// engines, so adding all instances into a single percentage is incorrect.
fn physical_engine_key(path: &str) -> Option<&str> {
    if !path.contains("GPU Engine(") || !path.contains("Utilization Percentage") {
        return None;
    }
    let start = path.find("_luid_")?;
    let tail = &path[start..];
    let end = tail.find("_engtype_")?;
    let key = &tail[..end];
    if !key.contains("_phys_") || !key.contains("_eng_") {
        return None;
    }
    Some(key)
}

/// Task Manager uses the busiest GPU engine as aggregate GPU load. This
/// diagnostic attempts the same policy by summing per-process contributions
/// *per physical engine* then taking the maximum. The accuracy and PDH sample
/// interval MUST be compared with a real GPU before production use.
/// Reject missing/malformed/oversubscribed readings; never call them zero.
pub fn busiest_gpu_engine_percent(rows: &[(&str, f64)]) -> UsageReading {
    if rows.is_empty() {
        return UsageReading::Unknown;
    }
    let mut totals: HashMap<&str, f64> = HashMap::new();
    for &(path, value) in rows {
        if !value.is_finite() || !(0.0..=100.0).contains(&value) {
            return UsageReading::Unknown;
        }
        let Some(key) = physical_engine_key(path) else {
            return UsageReading::Unknown;
        };
        *totals.entry(key).or_default() += value;
    }
    let busiest = totals.values().copied().fold(0_f64, f64::max);
    if busiest > 100.0 || !busiest.is_finite() {
        UsageReading::Unknown
    } else {
        UsageReading::from_percent(busiest as f32)
    }
}
