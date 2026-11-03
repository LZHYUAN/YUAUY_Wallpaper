# User Configuration Contract (YAML)

**Status**: Approved defaults and semantics as of 2026-11-03; Rust parser implementation is not written. [Sample](config.example.yaml).

| Key | Type | Default / validation | Semantics |
| --- | --- | --- | --- |
| `image_folders` | sequence of Windows folder paths | Required; unique paths; empty array allowed and produces `NoImages` | Global source for all monitors. Nonrecursive directory scan unless user requests a future change. |
| `slideshow_interval_seconds` | positive integer | **30** if omitted; must be greater than zero | Delay between normal transition opportunities, high-GPU skips, and empty-source rescans; changing it on reload changes future deadlines. |
| `load_guard.enabled` | boolean | `true` if omitted | Controls load gating for normal slideshow only, never startup composition. |
| `load_guard.metric` | `gpu_total` or `cpu_total` | `gpu_total` if omitted | Production uses best-effort system-wide GPU; GPU-less WindowsEnv testing uses system-wide CPU. |
| `load_guard.threshold_percent` | number | **60** if omitted; range 0 to 100 inclusive | ONE threshold: usage `>=` this value means skip normal transition; below means allow, with no hysteresis or separate recovery value. |

**Removed keys**: `load_guard.pause_at_percent` and `load_guard.resume_at_percent` are not accepted. There is no `gpu_poll_interval`, `directory_poll_interval`, `background_monitor`, `autostart`, `refresh_rate_cap`, `target_fps` or setting to alter the original slideshow math. The VSync target follows each monitor's effective display refresh rate only while animation runs.

**Parsing and state**: Validate the entire YAML before atomically applying. Unknown/old keys and invalid numeric values produce an understandable status in the tray while keeping the last valid configuration and static wallpaper. A missing image folder is not a crash; no valid photos means `NoImages` with the normal-timer rescan rule. Use case-insensitive supported JPG/JPEG/JFIF/PNG extensions and require successful decode before rendering.

**Load measurement**: One brief, bounded best-effort measurement per normal transition due-time; on real machines total GPU usage, WindowsEnv CPU usage. No history, expensive precision calibration, periodic monitor, or restart-only toggle. Invalid measurement is an explicit unknown/error rather than zero; failure policy and latency verified in Windows integration testing.

**Sign-in startup**: Controlled by a tray checkbox that *immediately* reads/writes the **current-user Windows Run registration**, not this YAML file. A registry write must be verified by read-back before UI checkmark changes.

**Security**: Only local photo files; use typed YAML data without executable tags; validate Windows paths and intervals, do not write private paths to routine logs or upload user photos. Read-only photo access.
