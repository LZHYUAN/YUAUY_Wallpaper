# User Configuration Contract (YAML)

**Status**: Proposed schema for `$speckit-plan`; not an implemented file format. [Sample](config.example.yaml).

| Key | Type | Required / validation | Semantics |
| --- | --- | --- | --- |
| `image_folders` | sequence of UTF-8 path strings | Required; unique paths; empty array is permitted and produces `NoImages` | Shared folders for all monitors and intro; default scan non-recursive as original C# picker. No photo data sent outside machine. |
| `slideshow_interval_seconds` | positive integer | Required; must be > 0; upper bound and minimum safe value to be selected during implementation tests | Time between normal transition opportunities, and between empty-state re-scans or load skips. No background GPU polling. |
| `load_guard.enabled` | boolean | Required | Disable/enable GPU gate for *normal* slideshow transitions only; never applies to opening composition. |
| `load_guard.metric` | enum `gpu_total`, `cpu_total` | Required | `gpu_total` default for real hardware; `cpu_total` for WindowsEnv/diagnostics without GPU; automatic fallback behavior to be validated, not silently invented. |
| `load_guard.pause_at_percent` | number | Required; 0 through 100 inclusive | Transition due with measured use >= high threshold is skipped, keeping static image. |
| `load_guard.resume_at_percent` | number | Required; >=0, < pause threshold | If a load skip has occurred, transitions resume when the due-time measurement <= recovery threshold. |

**Other constraints**: duplicate/unknown keys should be rejected clearly; do not leak image paths into routine logs. Parse YAML completely before switching live config; `ReloadConfig` on bad YAML reports an error and preserves the last valid settings. Open YAML via system default editor. If folder is inaccessible, mark as source warning and skip it safely; when none contain usable images use `NoImages`.

**No config entries**: `gpu_poll_interval`, `directory_poll_interval`, `background_monitor`, `monitor_images`, or settings for altering the original slideshow math. These would contradict the owner requirements or constitute an unapproved product extension.

**Default proposals (not approved)**: 30 seconds normal slideshow; 80% pause and 60% resume. This sample is for testing/feedback, not a claim of production defaults.

**Security**: only parse local config files; do not execute expressions, shell commands, scripts or arbitrary YAML tags. Validate numeric limits and reject invalid paths without destructive side effects. Image file operations should be read-only.
