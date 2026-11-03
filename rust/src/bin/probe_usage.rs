//! T011: single-shot, no-background-process total CPU / GPU-PDH diagnostic.
//!
//! - CPU: GetSystemTimes twice. Kernel time already includes idle.
//! - GPU: PDH "GPU Engine(*)/Utilization Percentage" enumeration and two
//!   samples, summed PER physical engine, then busiest engine.
//! - No background worker, sampler daemon, registry changes, image reads.
//! - PDH short-interval results are EXPERIMENTAL, NOT a validated load guard.

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("SKIPPED: probe_usage requires Windows.");
    std::process::ExitCode::from(2)
}

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    let mut metric = String::from("cpu_total");
    let mut budget_ms: u64 = 400;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--metric" => metric = args.next().unwrap_or_default(),
            "--budget-ms" => budget_ms = args.next().and_then(|v| v.parse().ok()).unwrap_or(0),
            _ => {
                eprintln!("FAIL: unexpected argument");
                return std::process::ExitCode::FAILURE;
            }
        }
    }
    if !["cpu_total", "gpu_total"].contains(&metric.as_str()) || !(100..=1000).contains(&budget_ms)
    {
        eprintln!("Usage: probe_usage [--metric cpu_total|gpu_total] [--budget-ms 100..1000]");
        return std::process::ExitCode::FAILURE;
    }
    let result = match metric.as_str() {
        "cpu_total" => probe::cpu(std::time::Duration::from_millis(budget_ms)),
        _ => probe::gpu(std::time::Duration::from_millis(budget_ms)),
    };
    println!("{result}");
    if result.starts_with("PASS ") || result.starts_with("PROVISIONAL ") {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::from(2)
    }
}

#[cfg(windows)]
mod probe {
    use std::{
        thread,
        time::{Duration, Instant},
    };
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::Performance::{
        PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
        PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA, PdhAddCounterW, PdhCloseQuery,
        PdhCollectQueryData, PdhExpandWildCardPathW, PdhGetFormattedCounterValue, PdhOpenQueryW,
    };
    use windows::Win32::System::Threading::GetSystemTimes;
    use windows::core::{PCWSTR, PWSTR, w};
    use yuauy_wallpaper::platform::UsageReading;
    use yuauy_wallpaper::usage_probe::{CpuTimes, busiest_gpu_engine_percent, cpu_busy_percent};

    fn ft(t: FILETIME) -> u64 {
        (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime)
    }

    fn times() -> windows::core::Result<CpuTimes> {
        let (mut idle, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        // SAFETY: three valid FILETIME out pointers. Read-only system-wide counters.
        unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user))? };
        Ok(CpuTimes {
            idle: ft(idle),
            kernel: ft(kernel),
            user: ft(user),
        })
    }

    pub(super) fn cpu(budget: Duration) -> String {
        let started = Instant::now();
        let Ok(before) = times() else {
            return "SKIPPED metric=cpu_total reason=GetSystemTimes_failed".into();
        };
        let sample_wait = (budget / 2).min(Duration::from_millis(200));
        thread::sleep(sample_wait);
        let Ok(after) = times() else {
            return "SKIPPED metric=cpu_total reason=GetSystemTimes_failed".into();
        };
        let elapsed = started.elapsed();
        if elapsed > budget {
            return format!(
                "SKIPPED metric=cpu_total reason=budget_exceeded elapsed_ms={}",
                elapsed.as_millis()
            );
        }
        match cpu_busy_percent(before, after) {
            UsageReading::Percent(p) => format!(
                "PASS metric=cpu_total percent={p:.3} elapsed_ms={} global_system=true sample_count=2",
                elapsed.as_millis()
            ),
            UsageReading::Unknown => format!(
                "SKIPPED metric=cpu_total reason=invalid_system_deltas elapsed_ms={}",
                elapsed.as_millis()
            ),
        }
    }

    struct Query(PDH_HQUERY);
    impl Drop for Query {
        fn drop(&mut self) {
            // SAFETY: PDH handle was obtained via PdhOpenQueryW and is owned.
            unsafe {
                let _ = PdhCloseQuery(self.0);
            }
        }
    }

    pub(super) fn gpu(budget: Duration) -> String {
        let started = Instant::now();
        let mut chars: u32 = 0;
        let null = PCWSTR(std::ptr::null());
        // SAFETY: PDH validates this well-formed wildcard counter name.
        let status = unsafe {
            PdhExpandWildCardPathW(
                null,
                w!("\\GPU Engine(*)\\Utilization Percentage"),
                None,
                &mut chars,
                0,
            )
        };
        if status != PDH_MORE_DATA || !(2..=65536).contains(&chars) {
            return format!(
                "SKIPPED metric=gpu_total reason=missing_GPU_Engine_PDH status=0x{status:08x} elapsed_ms={}",
                started.elapsed().as_millis()
            );
        }
        let mut paths = vec![0_u16; chars as usize];
        // SAFETY: allocated buffer sized per PDH first call.
        let status = unsafe {
            PdhExpandWildCardPathW(
                null,
                w!("\\GPU Engine(*)\\Utilization Percentage"),
                Some(PWSTR(paths.as_mut_ptr())),
                &mut chars,
                0,
            )
        };
        if status != 0 {
            return format!(
                "SKIPPED metric=gpu_total reason=PDH_expand_error status=0x{status:08x} elapsed_ms={}",
                started.elapsed().as_millis()
            );
        }
        // MULTI_SZ. Never log actual paths (they may contain process IDs).
        let mut spans = Vec::new();
        let mut cursor = 0;
        while cursor < paths.len() && paths[cursor] != 0 {
            let end = paths[cursor..]
                .iter()
                .position(|&x| x == 0)
                .map(|n| cursor + n);
            let Some(end) = end else {
                return "SKIPPED metric=gpu_total reason=malformed_counter_path_list".into();
            };
            spans.push((cursor, end));
            cursor = end + 1;
        }
        if spans.is_empty() || spans.len() > 256 {
            return format!(
                "SKIPPED metric=gpu_total reason=counter_count_invalid count={} elapsed_ms={}",
                spans.len(),
                started.elapsed().as_millis()
            );
        }
        if started.elapsed() > budget / 2 {
            return format!(
                "SKIPPED metric=gpu_total reason=budget_exceeded_before_collect elapsed_ms={}",
                started.elapsed().as_millis()
            );
        }
        let mut query = PDH_HQUERY::default();
        let open = unsafe { PdhOpenQueryW(null, 0, &mut query) };
        if open != 0 {
            return format!("SKIPPED metric=gpu_total reason=PDH_open_failed status=0x{open:08x}");
        }
        let query = Query(query);
        let mut counters = Vec::new();
        for &(begin, end) in &spans {
            if started.elapsed() >= budget / 2 {
                return format!(
                    "SKIPPED metric=gpu_total reason=budget_exceeded_during_add count={} elapsed_ms={}",
                    counters.len(),
                    started.elapsed().as_millis()
                );
            }
            let mut handle = PDH_HCOUNTER::default();
            // SAFETY: span ends at a NUL within the MULTI_SZ buffer.
            let rc = unsafe {
                PdhAddCounterW(query.0, PCWSTR(paths[begin..=end].as_ptr()), 0, &mut handle)
            };
            if rc != 0 {
                return format!(
                    "SKIPPED metric=gpu_total reason=PDH_counter_unavailable status=0x{rc:08x}"
                );
            }
            counters.push(handle);
        }
        let rc = unsafe { PdhCollectQueryData(query.0) };
        if rc != 0 {
            return format!(
                "SKIPPED metric=gpu_total reason=PDH_first_sample_failed status=0x{rc:08x}"
            );
        }
        // This probe uses a quick 100ms second sample only. Microsoft PDH
        // documentation recommends >=1s for rate counters; mark as provisional.
        let remaining = budget.saturating_sub(started.elapsed());
        if remaining < Duration::from_millis(110) {
            return format!(
                "SKIPPED metric=gpu_total reason=insufficient_sample_budget elapsed_ms={}",
                started.elapsed().as_millis()
            );
        }
        thread::sleep(Duration::from_millis(100));
        let rc = unsafe { PdhCollectQueryData(query.0) };
        if rc != 0 {
            return format!(
                "SKIPPED metric=gpu_total reason=PDH_second_sample_failed status=0x{rc:08x}"
            );
        }
        let mut owned = Vec::new();
        for ((begin, end), counter) in spans.iter().copied().zip(counters.iter().copied()) {
            let mut value = PDH_FMT_COUNTERVALUE::default();
            let rc =
                unsafe { PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, None, &mut value) };
            if rc != 0 || ![PDH_CSTATUS_VALID_DATA, PDH_CSTATUS_NEW_DATA].contains(&value.CStatus) {
                return format!(
                    "SKIPPED metric=gpu_total reason=PDH_invalid_formatted_value status=0x{rc:08x} elapsed_ms={}",
                    started.elapsed().as_millis()
                );
            }
            // SAFETY: PDH_FMT_DOUBLE selected the doubleValue union arm.
            let percent = unsafe { value.Anonymous.doubleValue };
            owned.push((String::from_utf16_lossy(&paths[begin..end]), percent));
        }
        let pairs: Vec<(&str, f64)> = owned
            .iter()
            .map(|(path, value)| (path.as_str(), *value))
            .collect();
        let elapsed = started.elapsed();
        if elapsed > budget {
            return format!(
                "SKIPPED metric=gpu_total reason=budget_exceeded elapsed_ms={}",
                elapsed.as_millis()
            );
        }
        match busiest_gpu_engine_percent(&pairs) {
            UsageReading::Percent(percent) => format!(
                "PROVISIONAL metric=gpu_total busiest_engine_percent={percent:.3} instances={} elapsed_ms={} sample_interval_ms=100 production_guard_approved=false",
                spans.len(),
                elapsed.as_millis()
            ),
            UsageReading::Unknown => format!(
                "SKIPPED metric=gpu_total reason=unreliable_aggregation instances={} elapsed_ms={}",
                spans.len(),
                elapsed.as_millis()
            ),
        }
    }
}
