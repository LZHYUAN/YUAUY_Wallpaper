# T011 — bounded one-shot system-wide CPU and GPU counter probe

**Status: T011 API-probe task COMPLETED. Windows-hosted diagnostic results recorded; production GPU guard and WindowsEnv/physical GPU acceptance explicitly remain unverified.**

## API selections and guarantees

- CPU total: two read-only Win32 `GetSystemTimes` snapshots separated by a short sleep (up to 200 ms); `kernel` **includes** idle, so the utilization is `(delta_kernel + delta_user - delta_idle)/(delta_kernel + delta_user) * 100`. This is whole-machine, not per-process. Invalid/zero/regressing counters produce **Unknown**, not 0%. The default CLI budget is 400 ms.
- GPU availability and experimental aggregation: expand local `\GPU Engine(*)\Utilization Percentage` with `PdhExpandWildCardPathW`; use up to 256 process-engine counter instances, `PdhOpenQueryW`, `PdhAddCounterW`, `PdhCollectQueryData` twice and `PdhGetFormattedCounterValue`. Group process contributions by LUID, physical adapter and engine; report the **busiest physical engine** as a diagnostic proxy for total GPU usage, rather than summing unrelated engines or checking only the wallpaper process.
- Missing/invalid/no counter samples, excessive cardinality, malformed instance identity and out-of-range totals all return **SKIPPED/Unknown**. No fallback to idle/zero, no automatic CPU fallback when metric is explicitly `gpu_total`. For GPU-less WindowsEnv, user-approved YAML `cpu_total` can be selected separately for tests.
- Every invocation is **one-shot**. No persistent PDH query, thread, watcher or monitor remains after return. CPU and GPU timing is recorded with `Instant`, and the probe refuses to return values when the specified budget has been exceeded. **Caveat:** Windows API calls are synchronous and cannot be forcibly interrupted by this probe; the time budget is checked between calls, not a guaranteed hard OS-call timeout. Production code must resolve this before relying on it for guaranteed responsiveness.
- The GPU PDH sample interval is just **100 ms**, while the general Windows PDH guidance recommends at least 1 second between samples for rate counters. Even a finite formatted result is labelled **PROVISIONAL**, **not** validated for threshold decisions, pending a comparison on a real WDDM GPU. There is no lengthy continuous calibration.

## Official references

- [Microsoft DirectX engineering — GPUs in Task Manager](https://devblogs.microsoft.com/directx/gpus-in-the-task-manager/): system-wide GPU usage is represented by its **busiest engine**. A GPU with many independent engines must not be averaged or naively summed.
- [Microsoft Learn — PdhExpandWildCardPathW](https://learn.microsoft.com/en-us/windows/win32/api/pdh/nf-pdh-pdhexpandwildcardpathw): use a two-pass wildcard counter-path expansion, then a MULTI_SZ buffer.
- [Microsoft Learn — collecting performance data](https://learn.microsoft.com/en-us/windows/win32/perfctrs/collecting-performance-data): two collections and a longer sample period for rate counters.

## Tests and acceptance status

- TDD RED: `cargo test --locked --test usage_probe_math --target x86_64-unknown-linux-gnu` initially failed with E0432 (missing module). GREEN: **5/5** tests for aggregate CPU math, counter invalidity, per-process GPU engine grouping, busiest engine and rejection of contradictory over-100% results.
- Windows MSVC `cargo clippy --locked --all-targets --target x86_64-pc-windows-msvc -- -D warnings` passed.
- GitHub hosted Windows runner one-shot CPU and GPU outputs: **PASS for CPU / SKIPPED for GPU** in [workflow 41679718337](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41679718337), job 126069840201, source commit bb40e93.
- WindowsEnv 25H2 CPU/GPU availability, per-call wall time and fallback: **SKIPPED** until non-disruptive command execution is available.
- Physical GPU Task Manager parity and threshold-60 acceptance: **SKIPPED**, production integration belongs to later tasks.

**Current verdict:** T011 has exercised and timed the Windows APIs and documented the missing-usable-GPU-sample fallback; it does not implement or approve the product GPU threshold gate. In particular, physical GPU usage remains **Unknown**, not zero, if the quick sample cannot be collected.

## Observed Windows GitHub runner evidence — 2026-11-04

[Windows CI workflow 41679718337](https://github.com/LZHYUAN/YUAUY_Wallpaper/actions/runs/41679718337), job `126069840201` successfully built and executed both one-shot probes. Exact output:

```text
PASS metric=cpu_total percent=4.000 elapsed_ms=200 global_system=true sample_count=2
SKIPPED metric=gpu_total reason=budget_exceeded_before_collect elapsed_ms=310
```

- **CPU**: whole-system calculation returned 4.0% from two system-wide samples in 200 ms on **that runner only**. It is not a WindowsEnv CPU-load measurement and is not a claim of accuracy on all hosts.
- **GPU**: wildcard-path discovery/expansion took too much of the 400 ms total budget before a reliable collection could begin (310 ms elapsed). The probe refused to sample and reported **SKIPPED** instead of 0. The presence of a wildcard PDH path does **not** establish presence of a discrete/physical GPU.
- **CPU fallback**: an explicit YAML `metric: cpu_total` chooses the whole-system CPU path for a GPU-less WindowsEnv. The probe deliberately does **not** auto-switch from a failing GPU choice to CPU. No continuous calibration, background polling or filesystem watcher was started.
- **Remaining product work**: T006/T008/T009 WindowsEnv gating, real GPU PDH/driver accuracy comparisons, a hard bounded execution strategy and normal-slide-deadline scheduling belong to later implementation and acceptance.
