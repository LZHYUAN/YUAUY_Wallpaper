//! T011 red tests for bounded total-CPU snapshots and a conservative GPU
//! busiest-engine diagnostic (not a confirmed production load guard).
use yuauy_wallpaper::platform::UsageReading;
use yuauy_wallpaper::usage_probe::{CpuTimes, busiest_gpu_engine_percent, cpu_busy_percent};

#[test]
fn overall_cpu_counts_kernel_idle_only_once() {
    let before = CpuTimes {
        idle: 100,
        kernel: 300,
        user: 100,
    };
    let after = CpuTimes {
        idle: 160,
        kernel: 400,
        user: 200,
    };
    // Total 100 kernel + 100 user, idle is part of kernel.
    assert_eq!(cpu_busy_percent(before, after), UsageReading::Percent(70.0));
}

#[test]
fn invalid_system_cpu_counters_are_unknown_not_zero() {
    let first = CpuTimes {
        idle: 100,
        kernel: 300,
        user: 100,
    };
    assert_eq!(cpu_busy_percent(first, first), UsageReading::Unknown);
    assert_eq!(
        cpu_busy_percent(
            first,
            CpuTimes {
                idle: 90,
                kernel: 390,
                user: 190
            }
        ),
        UsageReading::Unknown
    );
    assert_eq!(
        cpu_busy_percent(
            first,
            CpuTimes {
                idle: 160,
                kernel: 350,
                user: 100
            }
        ),
        UsageReading::Unknown
    );
}

#[test]
fn total_gpu_follows_busiest_physical_engine_after_per_pid_aggregation() {
    let rows = [
        (
            r"\GPU Engine(pid_100_luid_0x0000_0x0001_phys_0_eng_0_engtype_3D)\Utilization Percentage",
            35.0,
        ),
        (
            r"\GPU Engine(pid_200_luid_0x0000_0x0001_phys_0_eng_0_engtype_3D)\Utilization Percentage",
            24.0,
        ),
        (
            r"\GPU Engine(pid_300_luid_0x0000_0x0001_phys_0_eng_1_engtype_Copy)\Utilization Percentage",
            70.0,
        ),
        (
            r"\GPU Engine(pid_400_luid_0x0000_0x0002_phys_0_eng_0_engtype_3D)\Utilization Percentage",
            42.0,
        ),
    ];
    // Overall busiest engine=70%; summing 35+24+70+42 would be invalid.
    assert_eq!(
        busiest_gpu_engine_percent(&rows),
        UsageReading::Percent(70.0)
    );
}

#[test]
fn missing_or_invalid_gpu_counters_are_unknown() {
    let empty: [(&str, f64); 0] = [];
    assert_eq!(busiest_gpu_engine_percent(&empty), UsageReading::Unknown);
    assert_eq!(
        busiest_gpu_engine_percent(&[("not a GPU engine", 0.0)]),
        UsageReading::Unknown
    );
    let example =
        r"\GPU Engine(pid_10_luid_0x0000_0x0001_phys_0_eng_0_engtype_3D)\Utilization Percentage";
    assert_eq!(
        busiest_gpu_engine_percent(&[(example, -1.0)]),
        UsageReading::Unknown
    );
    assert_eq!(
        busiest_gpu_engine_percent(&[(example, f64::NAN)]),
        UsageReading::Unknown
    );
}

#[test]
fn duplicate_engine_sum_over_hundred_is_not_silently_clamped() {
    let a =
        r"\GPU Engine(pid_10_luid_0x0000_0x0001_phys_0_eng_0_engtype_3D)\Utilization Percentage";
    let b =
        r"\GPU Engine(pid_20_luid_0x0000_0x0001_phys_0_eng_0_engtype_3D)\Utilization Percentage";
    assert_eq!(
        busiest_gpu_engine_percent(&[(a, 80.0), (b, 80.0)]),
        UsageReading::Unknown
    );
}
