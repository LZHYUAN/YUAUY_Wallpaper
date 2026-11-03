# WindowsEnv 25H2 — 有上限 DWM/GPU/DirectComposition ETW 擷取嘗試

**日期：** 2026-11-04，Asia/Taipei（UTC+08）

**判定：權限阻擋（BLOCKED），不是合成分析成功。** 已**實際嘗試** WPR 系統 GPU + DesktopComposition 記憶體錄製，並另試僅含 DirectComposition provider 的獨立 `logman` ETW session。兩種途徑皆由 Windows 拒絕；**沒有任何 ETL**，受控 WARP 渲染**沒有開始**，不能主張已取得 DWM/DXGI/DirectComposition 事件或證明特定裁剪機制。T009、T019 與 Phase 2 Foundation Gate 仍未通過。

## 1. 實測前身分與環境

- 專案分支 `rust_build`，實測前 Git HEAD `57c319d99608bd8868a715880f9798f3f60bbae7`。
- WindowsEnv Windows 11 25H2，build `10.0.26200.9711`，1920×1080 互動桌面；CUA health `0.37.0`、UI Automation、D3D11 screenshot capability 正常。
- Windows 目前登入帳號：`PRIV_ADMIN=False`，未提權或變更使用者權限原則。
- 預先存在的 Explorer PID `6864`、`Program Manager` HWND `65788`；原有 Task Manager PID `4568`、Notepad PID `6404`。
- 受控 WARP 診斷：`probe_desktop_dcomp.exe`，來自先前已驗證的 Windows MSVC CI artifact，預期與 WindowsEnv 再次核對 SHA-256 `00b93e755a904df761e334a9690f26f84e3b9093262d94373bdc7c06916cd336`；兩次控制腳本的 hash 皆匹配。
- `wpr -profiles` 已確認包含 `GPU`、`DesktopComposition` 兩個 Microsoft 內建 profile；`wpr -status` 起始為 `WPR is not recording`。官方 WPR 可透過 `-start` 開啟指定 profile 並於 `-stop` 儲存 ETL；預設使用記憶體模式，`-filemode` 才是寫入可能持續增長檔案的模式。[1]

## 2. WPR 嘗試 — 確切權限錯誤

採用 PowerShell `Start-Process -Wait` 分離 stdout/stderr，避免 PowerShell `$ErrorActionPreference='Stop'` 在 native stderr 中斷腳本，並獨立讀回退出碼。原本安排最長 4500 ms 的 `DefView` 下方和上方 WARP 案例、每案在約 t=1.8、3.4 秒擷取 220×190 ROI 截圖；僅在 WPR 成功開始並查明 session 存在後才執行。錄製預設採**記憶體模式**，在 try/finally 中停止並保存，若失敗則取消。

- `2026-11-04T06:15:39.779+08:00`：控制器啟動；WARP SHA 核對通過；WPR 原始狀態未錄製。
- `2026-11-04T06:15:49.897+08:00`：呼叫 `wpr.exe -start GPU -start DesktopComposition`。
- **`WPR_START_EXIT=-984068079`**（十六進位 `0xC5585011`）。
- 原生 stderr：

```text
Failed to enable the policy to profile system performance.
Profile Id: GPU.Verbose.Memory, DesktopComposition.Verbose.Memory
Error code: 0xc5585011
Failed to enable the policy to profile system performance.
```

- 控制器回報 `RUN_ERROR=WPR_START_DENIED`；`2026-11-04T06:15:51.010+08:00` 的最終 `wpr -status` 仍為 `WPR is not recording`。
- `capture-v2-gpu-desktop.etl` **不存在**；`probe_desktop_dcomp.exe` **沒有啟動**。

最初有一版控制腳本把 WPR stderr 視為 PowerShell 終止錯誤，未取得退出碼；那次也沒有開始 WPR 或 WARP，因此不當作有效追蹤。以上**第二版原生 stderr／exit code**才是可採信的原因。

Microsoft 將 `Profile system performance` 指為 `SeSystemProfilePrivilege`，官方建議限於 Administrators；Microsoft Q&A 有相同 `0xC5585011` 錯誤與以管理員身分執行 WPR 才能開始的案例。[3][4] 這提供了權限診斷背景，但本次未修改權限設定。

## 3. logman user-provider ETW 替代嘗試

為排除 WPR 內建 profile 需要 system profiling 權限，而單一 user-mode manifest provider 或可記錄的情況，另嘗試直接建立**獨立且限額**的 ETW trace：

```powershell
logman create trace YUAUY_DComp_251104 -ets `
  -o <TEMP>\YUAUY-Win25-Probes\etw-capture-57c319d\logman-directcomposition.etl `
  -f bincirc -max 32 `
  -p {c44219d0-f344-11df-a5e2-b307dfd72085}
```

- `-ets` 對即時 ETW session 發命令、不儲存排程設定，`-f bincirc -max 32` 規劃 32 MB 循環 ETL 上限。[2]
- `2026-11-04T06:17:16.162+08:00`：執行前 `logman query -ets` 未列出本測試 session。
- 延遲 9 秒以允許測試主控視窗最小化後呼叫。
- **`LOGMAN_CREATE_DCOMP_EXIT=-2147024891`**（`0x80070005`：Access Denied），命令列回應「存取被拒。請嘗試以系統管理員身分執行此命令。」
- `RUN_ERROR=CREATE_TRACE_DENIED`，`FINAL_SESSION_STILL_LISTED=False`；`logman-directcomposition.etl` **不存在**。
- 因第一個 provider session 根本未建立，未執行後續增加 Dwm-Core/DXGI provider 或 WARP 過場。**不能聲稱三個 provider 的擷取能力已被實際驗證**。

## 4. 可見性與清理驗證

兩條路徑均在建立 ETW session 之前失敗，故這次**沒有**具備同步 ETW 的 WARP `Present`、也沒有合成事件時間線或效能指標。期間另由 Windows CUA 擷取幾張 1920×1080 桌面畫面；左上 x=12..231/y=12..201 的 ROI 均未出現洋紅色標記（與「沒有執行 WARP」一致，不是渲染失敗的證明）。

`2026-11-04T06:18:49+08:00` 實際清理讀回：

```text
OWN_SHELL_6836_CLOSED
OWN_SHELL_12152_CLOSED
OWN_SHELL_10820_CLOSED
WPR_EXIT=0 ... WPR is not recording
LOGMAN_QUERY_EXIT=0 OUR_SESSION_EXISTS=False
ETL_capture-v2-gpu-desktop.etl_EXISTS=False
ETL_logman-directcomposition.etl_EXISTS=False
PROBE_RUNNING=False
PROBE_INSPECT=INSPECT_ONLY raised=true same_icon_owner=true matches_Explorer=true attach_eligible=true composition=NOT_RUN visual_wallpaper=UNVERIFIED
```

CUA 再次列出 **沒有殘留 PowerShell/CMD 測試視窗**；Explorer 原 `Progman` HWND `65788`／PID `6864` 和原有 Task Manager PID `4568` 仍在。僅關閉本次新增的 Notepad 測試分頁，原本分頁保留且 Notepad 回復最小化。

最終 1920×1080 桌面 screenshot 與事前基準比較，**測試矩形、左側 x=0..466/y=0..1039 圖示區及右上 x=467..1919/y=0..354 區域逐像素完全相同**。這是指定區域的還原驗證，**不是**所有 Explorer／DWM 內部狀態都未變的絕對證明。

私有資訊（兩組 PowerShell stderr/exit code、清理狀態、原生截圖）仍在 WindowsEnv 的 `%TEMP%\YUAUY-Win25-Probes\etw-capture-57c319d\`；Hermes 的 Git-ignored 證據在 `rust/target/private-evidence/windowsenv-25h2/etw-capture-57c319d/`。未將桌面截圖、私人 ETL（實際不存在）或使用者日誌上傳至 GitHub。

本次兩張對照截圖 SHA-256：

| 證據 | SHA-256 |
| --- | --- |
| 基準桌面 | `7dfabe8767ae186340d86febd7053def991d30d2ea2db10aa511e0d6c36a2e08` |
| 清理後桌面 | `e7e0827899ed6047ea928c8eb0a20904fd3b5ad2df488ca24e6fbf2f8751c007` |

## 5. 下一步的權限條件與驗收門檻

**真正的 ETW 合成追蹤尚未完成。** 要繼續，需有在 WindowsEnv 互動桌面上授權執行 GPU／DesktopComposition profile 的管理員提升權限情境，並保留一套自動停止／原位復原流程；未取得該情境前不要重複以標準帳號啟動，也不要修改 `SeSystemProfilePrivilege`、UAC、ETW provider ACL 或 Explorer HWND 去規避拒絕。

權限具備後，沿用既有最長 4.5 秒的 WARP Below/Above DefView 正負控制組，記錄 ETW 啟動／停止、`Present` 計數、HRESULT、同步原生截圖時間及像素。只有觀察到可用的 ETL、事件/工作階段資料與截圖時間線，才能聲稱獲得 ETW 合成資料；即使 trace 產生，**仍須分析事件與 HWND/surface 的對應，不能自動判定 DWM 裁剪根因**。

此輪沒有修改 `master`、`directx_build`、C# 動畫數學、原有 `tasks.md` 未提交變更；T009/T019 保持未勾選。

## Sources

[1] https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wpr-command-line-options — WPR Command-Line Options
[2] https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/logman-create-trace — logman create trace
[3] https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/profile-system-performance — Profile system performance
[4] https://learn.microsoft.com/en-us/answers/questions/1339327/high-cpu-usage-without-applications-running-on-win — Error 0xc5585011 Microsoft Q&A
