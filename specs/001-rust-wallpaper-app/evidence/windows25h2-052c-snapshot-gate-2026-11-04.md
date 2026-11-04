# Windows 11 25H2 — `Progman 0x052C` VM snapshot safety gate

**日期：** 2026-11-04，Asia/Taipei

**分支：** `rust_build`
**結果：BLOCKED。** 未確認可還原的 VM checkpoint，故 **未發送** `Progman 0x052C`，也未執行 WARP 掛載實驗。此結果不是圖示後方動畫可見性測試的 PASS 或 FAIL。

## 明確授權與前提

專案擁有者授權在**具備可還原 VM 快照的 WindowsEnv**，短暫執行未公開 `Progman 0x052C`，驗證 WorkerW/DefView 合成、實際畫面、圖示互動及清理。授權不包括缺少 VM 快照保護的 Explorer 內部狀態變更，也不表示未公開訊息的回復語義已被驗證。

## 實際前置調查

WindowsEnv CUA 0.37.0 互動桌面與擷取能力通過，作業系統 Build 26200.9711。唯讀 PowerShell `Get-CimInstance Win32_ComputerSystem` 和 `Get-Command` 回傳：

| 項目 | 實際結果 |
| --- | --- |
| Manufacturer / Model | `Microsoft Corporation` / `Virtual Machine` |
| HypervisorPresent | `True` |
| Guest 中的 `Get-VM` / `Checkpoint-VM` | 未提供 |
| Guest 中的 `Get-VMSnapshot` / `Restore-VMSnapshot` | 未提供 |
| Guest 中的 `VBoxManage` / `virsh` / `qm` / `govc` | 未提供 |
| Hermes Linux 的 `virsh`, `VBoxManage`, `qm`, `govc`, `multipass`, `az`, `gcloud`, `aws` | 無可呼叫的管理 CLI |
| 當前允許的 Hermes / Windows CUA tools | 未找到經授權的 VM checkpoint 建立、列舉、還原操作 |
| Snapshot ID、對應 VM ID、建立時間、可呼叫的 restore endpoint | **尚未取得** |

這只能證明**目前會話無法驗證與操作 VM checkpoint**，不能斷言實際 hypervisor 宿主沒有快照。Windows 的 System Restore Point 不是 VM checkpoint。Microsoft 官方文件說明 `Checkpoint-VM` 建立 VM checkpoint、`Get-VMSnapshot` 查詢、`Restore-VMSnapshot` 套用已有 checkpoint。[1][2][3]

## 可執行試驗的必要檢查點

1. 從**宿主機管理端**核實對應這台 WindowsEnv 的 VM ID，並建立專用試驗前 checkpoint，讀回 checkpoint ID、狀態、建立完成時間。
2. 驗證管理工具／連線具備有效的 VM restore 權限與實際回復路徑；如需演練還原，先在試驗 VM 或複本進行，避免不可逆地覆蓋使用者桌面工作。
3. 在任何 `0x052C` 前記錄 Explorer／Progman／DefView／WorkerW 及截圖基準；僅使用有時間上限的 `SendMessageTimeoutW`，不更動 Explorer-owned HWND 屬性。
4. 僅使用自有短命 HWND 配合 D3D11 WARP／DirectComposition；同步擷取至少兩張執行中畫面，驗證圖示可見、可點擊、標記確實位於圖示後方。
5. 銷毀自有視窗、讀回 Explorer 與桌面狀態；若出現無法復原的差異，透過已驗證的 VM checkpoint 恢復。任何先決條件缺失即 **fail closed**。

## 收尾與任務狀態

本次沒有呼叫 `0x052C`、`SetParent`、`SetWindowLong`、`SetWindowPos` 或修改登錄／桌布。先前已驗證的 T009、T019 診斷任務仍完成；T037 與真正圖示後方動畫仍未驗收，**Phase 2 Foundation Gate 保持 BLOCKED**。原有 `.specify/`、`.agents/`、`rust/README.md`、`tasks.md` 未提交修改應保留。

## Sources

[1] https://learn.microsoft.com/en-us/powershell/module/hyper-v/checkpoint-vm?view=windowsserver2025-ps — Checkpoint-VM
[2] https://learn.microsoft.com/en-us/powershell/module/hyper-v/get-vmsnapshot?view=windowsserver2025-ps — Get-VMSnapshot
[3] https://learn.microsoft.com/en-us/powershell/module/hyper-v/restore-vmsnapshot?view=windowsserver2025-ps — Restore-VMSnapshot
