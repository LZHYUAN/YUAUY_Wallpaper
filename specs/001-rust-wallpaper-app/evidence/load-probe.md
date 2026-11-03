# T011 — bounded one-shot system-wide CPU and GPU counter probe

**Status: PARTIAL — diagnostic and portable aggregation tests implemented; hosted Windows execution and WindowsEnv validation pending. Keep T011 unchecked until results are reviewed.**

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
- GitHub hosted Windows runner one-shot CPU and GPU outputs: **PENDING** until the pushed workflow executes.
- WindowsEnv 25H2 CPU/GPU availability, per-call wall time and fallback: **SKIPPED** until non-disruptive command execution is available.
- Physical GPU Task Manager parity and threshold-60 acceptance: **SKIPPED**, production integration belongs to later tasks.

**Current verdict:** CPU counter semantics and GPU aggregation method are deterministic-test verified, but real GPU readings are not production-approved. Do not mark overall GPU load check as implemented.
