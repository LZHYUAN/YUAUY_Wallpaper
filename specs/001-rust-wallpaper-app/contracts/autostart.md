# Windows Current-User Sign-In Autostart Contract

**Status**: Approved behavior for future implementation. This design does not write registry keys on WindowsEnv right now.

## Registration scope and identity

- Registry: `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.
- App-owned value name: `YUAUY_Wallpaper` (stable and restricted to this app).
- Value data: the quoted path to the **installed** YUAUY Wallpaper executable and only necessary, validated arguments. Never use a test harness path as a production startup entry.
- Scope: current logged-in Windows user. Does not require administrator rights; does not affect other users or write to HKLM.
- OS truth: the current Windows Run registration and any applicable external startup restrictions, *not* a duplicate YAML setting.

## Immediate checkbox transaction

1. When tray menu opens, query the app-owned registry entry; reflect actual registration status (or explicit read error).
2. When the user checks the box, **synchronously initiate the Windows API write** of the app-owned value pointing to the active, appropriately quoted executable. On success read back exact command/path and then mark checked.
3. When the user unchecks, delete only the app-owned value; verify by reading back that it is absent, then uncheck.
4. On failed write, delete or verification: do not display success; reflect actual read-back if available, or show unknown/error state with the previous displayed checkmark until it can be verified. Explain the operation failed.
5. Repeated clicks are idempotent. No additional user confirmation, YAML reload, app restart or application exit is required to apply the Windows registration.

## Safety and packaging

- Do not modify other Run values, use RunOnce, or install a background service or scheduled task as a substitute.
- Quote paths with spaces correctly. Follow Windows documented command line restrictions for Run values (including maximum length) and report an actionable error rather than silently register an invalid entry.
- On relaunch, detect an already-running instance and avoid duplicate wallpaper surfaces (implementation design/gate).
- Windows **may delay** startup applications after user sign-in; the checkbox promises registration and immediate mutation, not exact scheduling at boot.
- Uninstall or update behavior must remove or update only the app-owned Run value. On ordinary "結束", leave the user's autostart choice intact.
- Integrate tests with a **disposable Windows user profile**: registry add/delete/read-back, path with spaces, access-denied/read-back mismatch, external settings change, repeated clicks, restart, cleanup of only the test-owned value. Do not change the current user's real startup registry during documentation/SDD work.

## Sources

- https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys
- https://support.microsoft.com/en-gb/windows/experience/startup-boot/configure-startup-applications-in-windows
