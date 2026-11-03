# Read-only, on-demand Windows acceptance inventory.
# Never edits registry, changes display modes, spawns WorkerW or attaches to
# Explorer. Native API probes run only under an explicit opt-in switch.
[CmdletBinding()]
param(
    [string]$ProbeDirectory = '',
    [switch]$RunReadOnlyProbes
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue) {
    $PSNativeCommandUseErrorActionPreference = $false
}

function Has-Command([string] $Name) {
    return [bool](Get-Command -Name $Name -ErrorAction SilentlyContinue)
}

function Run-SafeProbe([string] $Binary, [string[]] $Arguments) {
    $Executable = Join-Path -Path $ProbeDirectory -ChildPath $Binary
    if (!(Test-Path -LiteralPath $Executable -PathType Leaf)) {
        Write-Output ("SKIPPED probe={0} reason=binary_unavailable" -f $Binary)
        return
    }
    # Binary invocation is synchronous, with bounded work coded into the
    # probe; callers control when this runs. No WorkerW attachment is requested.
    & $Executable @Arguments
    $Exit = $LASTEXITCODE
    if ($Exit -eq 0) {
        Write-Output ("PROBE probe={0} status=PASS api_only=true" -f $Binary)
    } elseif ($Exit -eq 2) {
        Write-Output ("PROBE probe={0} status=SKIPPED api_only=true" -f $Binary)
    } else {
        throw ("Probe {0} failed with exit code {1}" -f $Binary, $Exit)
    }
}

$Version = [Environment]::OSVersion.Version
$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$IsWin = [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform(
    [System.Runtime.InteropServices.OSPlatform]::Windows
)

Write-Output ("BASELINE os_windows={0} version={1} architecture={2} source=local_process" -f $IsWin, $Version.ToString(), $Arch)
Write-Output ("TOOL rustc={0} cargo={1} cl_in_path={2} link_in_path={3}" -f (Has-Command 'rustc'), (Has-Command 'cargo'), (Has-Command 'cl.exe'), (Has-Command 'link.exe'))

$VsWhere = ''
if (${env:ProgramFiles(x86)}) {
    $Candidate = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (Test-Path -LiteralPath $Candidate -PathType Leaf) {
        $VsWhere = $Candidate
    }
}
$HasVsCxx = $false
if ($VsWhere) {
    $InstallVersion = & $VsWhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationVersion
    $HasVsCxx = [bool]$InstallVersion
}
Write-Output ("TOOL visual_studio_cpp_detected={0} sdk_registry_key_present={1}" -f $HasVsCxx, (Test-Path 'HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots'))

if (!$IsWin) {
    Write-Output 'SKIPPED reason=not_windows'
    exit 2
}

if (!$RunReadOnlyProbes) {
    Write-Output 'SKIPPED probes=not_requested use=-RunReadOnlyProbes'
    exit 0
}

if (![string]::IsNullOrWhiteSpace($ProbeDirectory)) {
    $ProbeDirectory = [System.IO.Path]::GetFullPath($ProbeDirectory)
} else {
    $ProbeDirectory = Split-Path -Parent $PSCommandPath
}

Run-SafeProbe 'probe_d3d11.exe' @('--warp')
Run-SafeProbe 'probe_workerw.exe' @('--inspect')
Run-SafeProbe 'probe_usage.exe' @('--metric', 'cpu_total', '--budget-ms', '400')
Write-Output 'VERDICT diagnostics_only=true desktop_layering=UNVERIFIED production_wallpaper=UNVERIFIED'
exit 0
