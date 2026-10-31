# System Tray and Application Command Contract

**Status**: Proposed user-facing Win32 tray behavior; no code exists yet.

| Command / status | Visible behavior | Internal action |
| --- | --- | --- |
| Status: `Ready` | `YUAUY Wallpaper — 執行中` | Normal interval active; no continuous rendering between animations. |
| Status: `NoImages` | `YUAUY Wallpaper — 沒有圖片` | Display status, keep controls and normal due timer; zero images => zero image draws and zero GPU reads. |
| `開啟設定檔` | Open user's YAML in default editor | Does not apply edits until reload. |
| `重新載入設定` | Show confirmation/error status | Validate YAML atomically, rebuild image source and reschedule safely; when previously `NoImages`, perform one immediate scan without waiting for interval. |
| `暫停輪播` | Tray shows `已暫停` | Retain current composited wallpaper; do not start new normal transitions. |
| `繼續輪播` | Tray shows `執行中` or `沒有圖片` | Resume from current state; proposed policy schedules a fresh normal interval (not an immediate catch-up burst). |
| `結束` | Icon disappears, no dangling overlay windows | Graceful renderer/window release; retain normal Windows desktop operation. |
| YAML parse / IO error | Status reports setting error without repeated popups | Preserve last valid configuration and scene, leave tray available. |
| GPU sample unavailable | `負載無法測量` (non-spamming status/log) | Proposed policy: skip that normal due-time animation; next attempt at next full interval, not treat unavailable as zero. |

**Empty-source auto recovery**: at each normal slideshow deadline (not a separate watcher) perform one folder scan. If usable photos are found, clear `沒有圖片` and resume display. Do not issue a balloon notification on each empty tick. Only user-initiated reload can cause a scan outside an automatic scheduled deadline.

**Notification framework**: Win32 `Shell_NotifyIcon` and a hidden message window for menu callbacks; no large always-open settings UI is required. Explorer restart recovery and tray re-registration are acceptance tests.

**Accessibility / language**: initial UI is Traditional Chinese; prefer descriptive menu items and state tooltip instead of relying on icon color alone. Never make an error require a modal dialog to quit or edit settings.
