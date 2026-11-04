# Run on the actual Hyper-V HOST, never inside the WindowsEnv guest.
# Read-only inspection. A successful exit establishes a checkpoint was listed,
# NOT that the checkpoint can be restored safely. Does not send 0x052C.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$VMName,

    [Parameter(Mandatory = $true)]
    [ValidateNotNullOrEmpty()]
    [string]$CheckpointName
)
$ErrorActionPreference = 'Stop'

foreach ($name in @('Get-VM', 'Get-VMSnapshot', 'Restore-VMSnapshot')) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        throw "HOST_PREFLIGHT_BLOCKED: Hyper-V management command not available: $name"
    }
}

$vm = Get-VM -Name $VMName -ErrorAction Stop
if ($vm.Name -cne $VMName) {
    throw 'HOST_PREFLIGHT_BLOCKED: VM identity mismatch'
}

$matches = @(Get-VMSnapshot -VMName $VMName -ErrorAction Stop |
    Where-Object { $_.Name -ceq $CheckpointName })
if ($matches.Count -ne 1) {
    throw "HOST_PREFLIGHT_BLOCKED: expected exactly one named checkpoint, found $($matches.Count)"
}

$snapshot = $matches[0]
[pscustomobject]@{
    Gate = 'CHECKPOINT_LISTED_ONLY'
    VerifiedOn = [DateTimeOffset]::Now.ToString('o')
    Host = $env:COMPUTERNAME
    VMName = $vm.Name
    VMId = $vm.Id.ToString()
    VMState = $vm.State.ToString()
    CheckpointName = $snapshot.Name
    CheckpointId = $snapshot.Id.ToString()
    CheckpointCreated = $snapshot.CreationTime.ToString('o')
    CheckpointType = $snapshot.SnapshotType.ToString()
    RestoreCommandInstalled = $true
    RestoreDemonstrated = $false
    ExplorerMessageSent = $false
    SafeToRun052C = $false
    Reason = 'A listed checkpoint is insufficient: separately verify restore permission and recovery on an isolated VM before the guest-side experiment.'
} | ConvertTo-Json -Depth 3
