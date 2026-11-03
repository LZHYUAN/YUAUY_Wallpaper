# System Tray and Application Command Contract

**Status**: User-approved tray commands and immediate Windows sign-in autostart behavior; no executable or OS registry writes have occurred yet.

| Command / status | Visible behavior | Internal action |
| --- | --- | --- |
| Status: `Ready` | `YUAUY Wallpaper — 執行中` | Normal slideshow interval (default 30 seconds); no continuous rendering between animations. |
| Status: `NoImages` | `YUAUY Wallpaper — 沒有圖片` | Keep tray and timer; zero image draws/CPU-GPU reads. Once per normal deadline scan source and auto-resume on success. |
| `開啟設定檔` | Open user's YAML in default editor | Edits apply only after reload. |
| `重新載入設定` | Confirmation or readable error | Parse YAML atomically; apply folder, interval, guard and adjustable `max_fps` (default 120) settings; on `NoImages` immediately scan once; do not affect Windows sign-in autostart. |
| `暫停輪播` | Checkmarked `已暫停` state | Retain current wallpaper and suppress ordinary transitions; no GPU/background polling. |
| `繼續輪播` | `執行中` or `沒有圖片` | Resume from state; schedule next normal interval without catch-up burst. |
| `登入 Windows 後自動啟動` (checkbox) | Checkmark equals OS read-back state, not local requested boolean | ON immediately create app-owned HKCU Run entry with the current exe path; OFF immediately delete that value; read back and update checked state only if successful. |
| Auto-start write fails | Error appears and checkbox reflects actual Windows state | Read back; keep error visible; never claim success, delay change to exit, or modify unrelated startup entries. |
| `結束` | Exit and remove notification icon | Clean resources; do **not** silently disable the user's Windows sign-in autostart preference. |
| YAML parse/IO error | Clearly reported without repeated modal alerts | Preserve last valid YAML state and compositor. |
| GPU sample unavailable | Non-spamming status/log | Proposed policy: skip that due transition, retry at next full interval; never treat unavailable as 0. |

**Startup semantics**: "登入 Windows 後自動啟動" means current-user **sign-in**, not power-on before login, and not a system service. The app must not require elevated privileges. Use a single app-owned HKCU Run value, quote executable paths containing spaces, verify write/delete with a read-back, and recheck OS state each time the tray menu is opened. Windows may defer launch; if an external tool disables autostart, never represent a mere registration as a guaranteed process start. See [autostart.md](autostart.md).

**Empty-source auto recovery**: The existing slideshow deadline triggers one folder scan. If usable photos are found, clear `沒有圖片` and resume display. Do not issue a notification balloon on every empty tick. Only explicit tray reload performs an unscheduled directory scan.

**Notification framework**: Win32 `Shell_NotifyIcon` and hidden message window for menu callbacks; no permanent settings window required. Explorer restart/tray re-registration are acceptance tests.

**Accessibility/language**: Traditional Chinese menu and status text; distinguish checked state from disabled/unavailable, and surface error text independent of icon color. Do not require a modal dialog to quit or edit settings.
