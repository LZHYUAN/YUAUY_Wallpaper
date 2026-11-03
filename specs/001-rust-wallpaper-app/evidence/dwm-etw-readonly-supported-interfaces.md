# WindowsEnv 25H2 — 唯讀 DWM／ETW 調查與 Microsoft 官方桌面介面稽核

**日期：** 2026-11-04，Asia/Taipei（UTC+08）

**判定：** **READ-ONLY ENVIRONMENT AUDIT PASS；實際 DWM compositing trace NOT CAPTURED；正式圖示後方動態桌布仍 BLOCKED。**

> 本文件是對既有 [DefView 上／下層](directcomposition-defview-ab.md)、[Layered 透明度](directcomposition-layered-windowsenv.md)、[自有頂層替代宿主](desktop-compositor-host-audit.md) 實測的延伸，不是新的圖示後方渲染成功證據。

## 1. 實測主機與操作界線

- WindowsEnv 25H2：`10.0.26200.9711`（Windows 環境的 `Environment.OSVersion` 簡化為 `10.0.26200.0`）；主畫面 1920×1080。Windows CUA health check：Windows 0.37.0、UI Automation、D3D11 擷取能力 PASS。
- 開發 Git 分支 `rust_build`，調查開始時 HEAD `f306a239e903a89982447c1559f3448221b268fe`。
- 查詢帳號：`PRIV_ADMIN=False`；系統程序讀回 `explorer:6864`、`dwm:1440` 與 `dwm:3460`（PID 僅反映查詢當下，並不是畫面合成分析）。
- 完整操作僅查詢現有 ETW 工作階段、provider manifest、事件通道啟用狀態、既有日誌、WPR 錄製狀態及官方 COM 介面是否可由 PowerShell interop 讀取。**未使用 `wpr -start`、`logman start`、`wevtutil sl`、`Enable`、`SetWallpaper`、`SetWindowLong`、`SetParent`、Explorer restart 或任何提權。**
- 程式只將讀取結果寫入測試者的 `%TEMP%\YUAUY-Win25-Probes\etw-readonly-f306a23\`。未改動任何現有 ETW session、事件通道、桌布、視窗或登錄設定。

## 2. ETW／DWM 實測結果

Microsoft 官方 `logman query -ets` 是對 ETW session 的查詢，不是建立記錄；`Get-WinEvent -ListProvider` 取得的是 provider metadata；`wpr -status` 是檢查 WPR 是否正在記錄。[1][2][3]

| 唯讀命令／檢查 | WindowsEnv 結果 | 可解釋的範圍 |
| --- | --- | --- |
| `logman.exe query -ets` | `LOGMAN_QUERY_EXIT=0`，表格沒有列出 session | **此非管理員查詢**未枚舉到使用中的 session；不代表絕對不存在任何不可見的核心／受限 session |
| `Get-WinEvent -ListProvider Microsoft-Windows-Dwm-Core` | provider GUID `9e9bba3c-2e38-40cb-99f4-9e8281425164`，**345 個事件定義**，log link 為 `Microsoft-Windows-Dwm-Core/Diagnostic` | 事件**定義數量**，不是已錄製 345 筆 |
| `Microsoft-Windows-DirectComposition` | GUID `c44219d0-f344-11df-a5e2-b307dfd72085`，**72 個事件定義** | Provider 存在，不代表 channel 正在記錄 |
| `Microsoft-Windows-DxgKrnl` | GUID `802ec45a-1e99-4b83-9920-87c98277ba9d`，**712 個事件定義** | 相同限制 |
| `Microsoft-Windows-Win32k` | GUID `8c416c79-d49b-4f01-a467-e56d3aa8234c`，**369 個事件定義** | 相同限制 |
| `Microsoft-Windows-DXGI` | GUID `ca11c036-0102-4a2d-a6ad-f03cfed5d3c9`，**212 個事件定義** | 相同限制 |
| `wevtutil el` + `wevtutil gl` | Dwm-Core／Dwm-Compositor／Dwm-API／Dwm-Dwm／Dwm-Redir／Dwm-Udwm、DirectComposition/Diagnostic 與 DXGI/Analytic／Logging 均回報 **`enabled: false`** | 現有 analytic/diagnostic 通道未啟用 |
| `Get-WinEvent -FilterHashtable @{ProviderName='Microsoft-Windows-Dwm-Core'; ...}` | `參數錯誤`，未讀取到有效事件 | **無法推定「事件數為零」**，只能判定這次查詢無可用資料 |
| `wpr.exe -status` | `WPR_STATUS_EXIT=0`、`WPR is not recording`（WPR 10.0.26100） | 明確確認本機 WPR **當時沒有進行錄製** |

**這次沒有取得任何可用的 DWM composition event payload、surface ID、clip rectangle 或 alpha 對應資料。** 因此不能將先前 HWND 可見性失敗直接歸咎於某個特定 DWM compositor event。可見性與 HWND Z-order 的事實仍以此前 WindowsEnv 原生截圖／像素驗收為準。

WPR 是 Microsoft 官方 ETW 記錄工具；要取得新的合成事件一般必須啟動一次有期限的 recording profile。這個操作會啟用系統追蹤、配置記錄資源與產生 ETL，**不再屬於本次要求的「唯讀」操作**，因此未執行。[3]

### 本次可重現的唯讀查詢

```powershell
# 只查詢，不建立/停止 session，也不啟用 analytic channel。
logman.exe query -ets
Get-WinEvent -ListProvider Microsoft-Windows-Dwm-Core
Get-WinEvent -ListProvider Microsoft-Windows-DirectComposition
Get-WinEvent -ListProvider Microsoft-Windows-DxgKrnl
Get-WinEvent -ListProvider Microsoft-Windows-Win32k
Get-WinEvent -ListProvider Microsoft-Windows-DXGI
wevtutil.exe el
wevtutil.exe gl Microsoft-Windows-Dwm-Core/Diagnostic
wevtutil.exe gl Microsoft-Windows-DirectComposition/Diagnostic
wevtutil.exe gl Microsoft-Windows-DXGI/Analytic
wpr.exe -status
```

執行這些查詢不需要變更 Explorer 或進行桌布繪製。注意列舉系統 provider/log 可能需要足夠權限；不足權限或 channel 停用時應誠實記錄，不應以管理員權限繞過。

## 3. Microsoft 官方桌面整合介面評估

| 官方介面 | 文件明確支援的用途 | 能否直接承載本專案的圖示後方 D3D11 動畫？ |
| --- | --- | --- |
| `IDesktopWallpaper` | 管理各螢幕桌布圖檔、位置、顏色、輪播與監視器資訊；`SetWallpaper` 接收**圖片路徑**。[4] | **未見即時畫面／D3D swapchain 提交通道**。可管理靜態圖片或系統輪播，不能直接證成每幀 Direct3D 動畫 |
| `IDCompositionDevice::CreateTargetForHwnd` | 在**呼叫程序自有 HWND**上建立 DirectComposition target，且內容受 window clipping 約束；以 Explorer 的 DefView/Progman 外部 HWND 直接建立 target 會因跨程序所有權遭拒。[5] | 自有視窗可以，但**必須先有符合圖示後方定位的合法 HWND**。先前實測仍未取得 |
| `Windows.UI.Composition.Desktop`／`ICompositorDesktopInterop` | 讓 Win32 應用的 HWND 成為其自身 visual layer target。[8] | 能做 Win32 視窗內容，不提供已證實的 Explorer 圖示後方特殊合成層 |
| `DwmRegisterThumbnail` | 將既有**頂層來源視窗**的縮圖呈現在呼叫者擁有的目的地視窗；API 不會改寫桌面合成。[6] | 是**讀取／轉顯既有視窗**的功能，不是將本程式的影格注入 Explorer 桌布層 |
| `Windows.Graphics.Capture` | 在允許情境下擷取視窗或顯示器影格，供錄影、快照等使用。[7] | **擷取** API，不是桌布畫面寫入 API；不能解決 DefView 下方宿主不可見 |

官方文件中的 `IDesktopWallpaper` 方法列舉沒有任何每幀 pixel buffer、DirectComposition target 或 swapchain setter；這是依文件功能範圍作的**負面適用性判斷**，不是宣稱微軟系統內部沒有其他未公開機制。[4]

### IDesktopWallpaper COM 唯讀試探的限制

WindowsEnv 的 PowerShell 僅嘗試建立 DesktopWallpaper COM 物件並轉為手動宣告的 `IDesktopWallpaper` 介面，以讀取 monitor count；該 PowerShell/.NET interop **轉型失敗**（`System.__ComObject` 無法轉型為宣告介面），因此**沒有成功讀取實際 monitor count 或桌布圖檔**。這不代表 Microsoft 官方 COM 介面本身不可用；僅代表本次手寫 interop 測試未驗證成功。未呼叫任何會修改桌布的 setter。

## 4. 與先前實測結論的交集

- 之前的自有 `Progman` 子視窗位於 `DefView` **下方**，WARP／DComp 提交成功，但實際畫面與桌面基準完全相同；`DefView` **上方**可見，卻遮住圖示。
- `WS_EX_LAYERED` + alpha 255/128 不會讓 **DefView 下方**的子視窗可見；上方的透明度能正常混合。
- 自有頂層 `WS_POPUP` 改以 `Progman` HWND 相對定位或 `HWND_BOTTOM` 嘗試放下方時，實際 Z-order 卻讀回到 `Progman` **上方**，程式在任何 `Present` 前安全拒絕。
- 本次只驗證「相關 ETW provider 有安裝、既有診斷通道未啟用、WPR 無錄製」及「官方 API 沒有直接提供可確認的圖示下層動畫 target」。**仍不能精確區分** `DefView` layered alpha、`SysListView32` 遮蔽與 DWM composition clip 的哪一個是最終原因。

## 5. 測試後清理與狀態

- 沒有啟動或停止任何 ETW session、沒有切換 channel；WPR 最終查詢回報未錄製。
- 專用 PowerShell 視窗已關閉，為讀取日誌開啟的 Notepad 測試分頁也已關閉，原有 Notepad 分頁保留並恢復最小化狀態。
- 最終 CUA `list_windows`：沒有殘留 PowerShell/CMD；Explorer `Program Manager` 仍為 PID `6864`／HWND `65788`。
- 最終 1920×1080 截圖，對照先前清理基準，`x=12..231, y=12..201` 測試矩形、`x=0..466, y=0..1039` 圖示區及右上未遮擋區域逐像素完全一致。**不延伸聲稱每個系統內部狀態都沒有變化。**
- 私人證據截圖：`rust/target/private-evidence/windowsenv-25h2/etw-readonly-f306a23/final_desktop.png`（Git 忽略，SHA-256 `45134635f875e7237181ee9f29310f460c6f8e2c5490081f1dcc1c589df02d0f`）；Windows TEMP 中有詳細文字檔 `read-only-inspection.txt` 及 `wpr-wallpaper-read.txt`。未上傳私人截圖或本機日誌。

## 6. 工程決策

**T009／T019 繼續未勾選，Phase 2 仍 BLOCKED。** 無符合需求的官方桌面動態畫面承載介面或已驗證可見的圖示後方 D3D11 渲染路徑。

若日後要繼續定位 DWM 內部原因，可提出**單獨、有時限的 ETW/WPR 記錄授權**：包含選用的 GPU/DWM profile、是否需要管理員、資料保存位置、清理、以及與原生截圖同步的測試窗口。但因開啟 trace 會改變追蹤狀態，本次沒有執行，也**不能冒稱取得 DWM/ETW 合成 trace**。

在驗收通過前，不修改 `master`／`directx_build`、原始 C# 動畫數學、Explorer 視窗屬性，不改用覆蓋圖示的前景透明視窗。

## Sources

[1] https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/logman-query — Logman query
[2] https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.diagnostics/get-winevent?view=powershell-7.5 — Get-WinEvent
[3] https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wpr-command-line-options — WPR command-line options
[4] https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-idesktopwallpaper — IDesktopWallpaper
[5] https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-idcompositiondevice-createtargetforhwnd — DirectComposition CreateTargetForHwnd
[6] https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmregisterthumbnail — DWM Register Thumbnail
[7] https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture — Windows Graphics Capture
[8] https://learn.microsoft.com/en-us/windows/uwp/composition/using-the-visual-layer-with-win32 — Win32 Visual Layer
