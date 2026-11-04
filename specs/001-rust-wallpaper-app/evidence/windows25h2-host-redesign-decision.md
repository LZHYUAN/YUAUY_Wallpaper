# Windows 11 25H2 — 桌面宿主／DefView 合成重新設計決策

**日期：** 2026-11-04，Asia/Taipei。**分支：** `rust_build`。
**狀態：** **新驗收程式已實作與測試，實機圖示後方像素仍未證實可見；Foundation Gate 保持 BLOCKED**。本文件不宣稱已解除 T037 或 Phase 2 的技術阻礙。

## 1. 最小可重現問題與根因邊界

在 WindowsEnv 25H2（`10.0.26200.9711`）的先前 T009/T019 實機測試中，Rust 建立的 `Progman` 子視窗擁有正確的父子關係、Z-order 確實在 `SHELLDLL_DefView` 下方，而且 D3D11 WARP/DirectComposition 成功完成 **303 次 Present**（HRESULT `0x00000000`），最後自有 HWND 銷毀成功。然而 220×190 的兩張實際執行中 PNG 與銷毀後 PNG **SHA-256 相同**，無法看見渲染內容，不能批准桌布宿主。請參閱 [T009/T019 原生驗收](t009-t019-closure-2026-11-04.md) 和 [DirectComposition AB 實驗](directcomposition-defview-ab.md)。

這個症狀不能以延長繪製時間、提高 FPS、修改動畫公式、單純把 `SetWindowPos` 放到 `DefView` 後面，或把 API `S_OK` 當作視覺成功來修復。

Microsoft 官方 `IDCompositionDevice::CreateTargetForHwnd` 文件說明，DirectComposition target **必須使用呼叫者自己擁有的 HWND**，且內容仍受視窗可見區域裁剪。不能合法地直接把我們的 visual tree 綁到 Explorer 擁有的 `DefView` HWND。[2]

Lively 社群 Issue #2074 公開引述 Microsoft 開發者：自 Build 26100 起的 raised desktop 預設使用 layered `DefView`，但 `DefView` 在 HDR 或系統輪播動畫等情境以外，並不保證背景透明合成；這是**經社群引述的工程線索，不是公開 Win32 API 保證**。[1]

## 2. 本次 WindowsEnv 唯讀驗證（22:24 +08）

透過 Windows PowerShell Win32 P/Invoke，僅呼叫 `GetShellWindow`、`FindWindowExW`、`GetWindowLongW`、`GetLayeredWindowAttributes` 和 `GetWindowThreadProcessId`，**沒有任何修改 Explorer／桌布設定的呼叫**。

| 欄位 | WindowsEnv 實際讀回 | 解釋 |
| --- | --- | --- |
| `Progman` | HWND `65788`，Explorer PID `6864` | 原有桌面宿主存在 |
| `Progman GWL_EXSTYLE` | `0x00200080` | 含 `WS_EX_NOREDIRECTIONBITMAP`，符合 raised desktop |
| `SHELLDLL_DefView` | HWND `65794`，同 PID | 圖示宿主仍由 Explorer 擁有 |
| `DefView GWL_EXSTYLE` | `0x00080000` | 含 `WS_EX_LAYERED` |
| `GetLayeredWindowAttributes` | `TRUE`、alpha=`255`、flags=`LWA_ALPHA` | **只說明全域 alpha 設定，不能當作每個像素不透明或透明的證據** |
| 直接子 `WorkerW` | **不存在** | 沒有現成、已驗證的全螢幕 wallpaper WorkerW 子宿主 |

Microsoft 官方 `GetLayeredWindowAttributes` 僅回報全域 alpha／color key，而且有使用模式的限制，不會回報 `DefView` 的逐像素 Alpha。[3] 因此，**實際有沒有讓下層像素透出**仍以同步畫面擷取為準。使用者當時開啟的 Notepad、Task Manager 與 Explorer 保留，臨時 PowerShell 視窗已關閉。

## 3. 重新設計候選與不可跨越的約束

| 路徑 | 具體作法 | 本機驗證 / 風險 | 決策 |
| --- | --- | --- | --- |
| A. 自有 `Progman` layered 子視窗 + WARP/DComp | `WS_EX_LAYERED`、alpha=255、自有 HWND 置於 `DefView` 下方 | 既有視窗生命週期可還原，但下層像素不可見 | **已否決直接投入正式產品** |
| B. 自有頂層 HWND 強制在 `Progman` 後方 | `SetWindowPos(..., Progman)` 或 `HWND_BOTTOM` | WindowsEnv 兩種都讀回位置在 `Progman` **上方**，安全探針未畫出任何影格 | **已否決** |
| C. 觸發 Explorer raised-desktop 動畫／WorkerW 狀態 | 社群使用未公開的 `Progman` 訊息 `0x052C`，有範例採 `wParam=0xD,lParam=1`，再建立/掛載自有 layered HWND | 可能讓 DefView 背景暫時透明；**直接改變 Explorer 內部狀態並可能產生 WorkerW**。未經 WindowsEnv 實測，且 Microsoft 沒有保證此私有訊息的 ABI／恢復語義 | **條件候選；目前禁止自動或實機執行** |
| D. 變更 Windows HDR、桌布輪播／主題 | 令 shell 進入不同透明／桌布過場狀態 | 會改變使用者全域桌面設定，亦非可靠的正常桌布宿主 | **不採用** |
| E. 前景透明視窗或像素摳洞模擬 | 在圖示上方繪製、以透明區域放行點擊 | 即使看起來接近桌布，也不是「真正位於桌面圖示後方」 | **規格禁止** |

社群參考：[Lively 24H2 討論][1]、[HextopLite][4] 與 [WebView2 WorkerW 宿主紀錄][5]。其 `0x052C` 用法**沒有正式文件支持**。部分範例將 `wParam=0xD,lParam=0` 當恢復，其他舊資料卻也使用 `0` 作初始化序列；在本機未驗證前，**不能承諾「傳回 0 即可完整復原」**。沒有可重現快照/回復驗證、或沒有使用者對 Explorer 狀態變更的明確例外授權，就不應觸發此訊息。

## 4. 已落地的安全驗收閘門

新增純 Rust 模組：

- `rust/src/desktop_host_gate.rs`：`CompositorEvidence`、`HostParent`、`GateBlocker`、`decide()`，獨立於 D3D11、WorkerW 及原 C# 動畫數學；
- `rust/tests/desktop_visibility_gate.rs`：先出現找不到模組的 RED，再以小步 RED/GREEN 加入可驗證的封鎖條件。

決策只考慮 **已在 WindowsEnv 實際取得**的觀測：由我們驗證的 `Progman` 或 `WorkerW` 作為合法宿主、位於圖示下層、至少一次成功 D3D11 Present、**至少兩次執行中的標記像素觀測**、可見區域至少 10% 呈現標記、圖示視覺及互動仍正常、自有 HWND 已銷毀，而且銷毀後標記顏色完全消失。任何缺值或不符均回傳 `Blocked(...)`。

這只是 **HostAcceptance**；即使未來 `decide()` 回傳 `HostAccepted`，完整 Foundation Gate 還另需 T006 WindowsEnv 工具盤點、T010 實機 VSync/FPS 上限及專案既定檢查。因此 `HostAccepted` **不能直接解除完整 Phase 2**。

新測試刻意涵蓋：303 次成功 Present 但 0 可見像素 → `InvisibleUnderlay`；畫面在圖示上方 → `WrongLayer`；只有一個匹配色點／單張截圖 → 拒絕；缺乏圖示完整性證據 → 拒絕；銷毀後仍有標記 → 拒絕；有可驗證的 WorkerW 親代時並不被程式錯誤拒絕。**單元測試使用合成觀測值，不能取代真正的 WindowsEnv 畫面驗收。**

本次在 Hermes Linux 執行完整 `cargo test --locked --offline --target x86_64-unknown-linux-gnu`：**70 passed / 0 failed**；Windows MSVC target `cargo clippy --locked --offline --all-targets --target x86_64-pc-windows-msvc -- -D warnings` **PASS**；`cargo fmt --all -- --check` **PASS**。私有 log 保存在忽略 Git 的 `rust/target/private-evidence/redesign-*.log`。這不是 WindowsEnv 原生編譯或圖示後方可見性成功證據。

## 5. 必要的下一階段與禁止事項

若專案擁有者**明確批准短暫改變 Explorer 內部桌面狀態**，才可能在可回復的 WindowsEnv 快照中執行路徑 C；在仍維持「不修改 Explorer 視窗／桌面狀態」原則時，不能默許這個例外。

後續有條件實驗須具備以下安全門檻：

1. 驗證 Progman/DefView 同屬 Explorer、保存完整 HWND parent、style、Z-order、WorkerW 子視窗與桌面截圖基準，**並確認有可信的 VM 快照或回復途徑**。
2. 僅在專門的有限測試視窗，以 bounded `SendMessageTimeoutW` 而非無上限 `SendMessage`；不能推定未公開訊息會成功，也不能重複盲發。
3. 只建立或修改**本程式擁有**的 layered HWND，使用 D3D11/WARP 的已驗證 DirectComposition 視覺路徑，固定幾秒後關閉；不直接變更 Explorer 自有 HWND 屬性或 Z-order。
4. 在**執行期間**取得至少兩張完整截圖及可見標記／圖示區像素，實際檢查圖示點擊而非只看 `WS_EX_TRANSPARENT`，並驗證結束後 HWND 銷毀、無殘留 WorkerW／程序與原桌面狀態。
5. 若任何狀態無法確認或回復，立即判為 **BLOCKED**，不把「自稱可逆」當作已驗證復原。

**最終判定：** 沒有證據證明 Windows 11 25H2 的下層 D3D11 畫面已可見；現有非修改 Explorer 的合法宿主途徑仍失敗，故不解除 Foundation Gate、不勾 T037、不更動 `master` 或 `directx_build`。下一個決策是是否批准例外測試 `0x052C`，而不是把同一段已成功的 D3D11 Present 程式再重跑一次。

## Sources

[1] https://github.com/lively-community/lively/issues/2074 — Lively Windows 11 24H2 shell composition discussion
[2] https://learn.microsoft.com/en-us/windows/win32/api/dcomp/nf-dcomp-idcompositiondevice-createtargetforhwnd — IDCompositionDevice::CreateTargetForHwnd
[3] https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getlayeredwindowattributes — GetLayeredWindowAttributes
[4] https://github.com/xfractalino/HextopLite — HextopLite shell proof of concept
[5] https://github.com/bbabcock1990/tool-animated-wallpapers/blob/main/README.md — Windows 11 raised desktop WorkerW
