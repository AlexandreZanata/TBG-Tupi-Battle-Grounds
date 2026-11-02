# Select the reviewed MSVC x64 environment; record the actual hosted runner tools.
$ErrorActionPreference = 'Stop'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$installation) { throw 'MSVC x64 toolset is required' }
# Launch-VsDevShell.ps1 does not forward DevCmdArguments on this runner.
# The underlying Microsoft cmdlet owns that parameter and selects the SDK.
$module = Join-Path $installation 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll'
Import-Module -Name $module -ErrorAction Stop
$enter = Get-Command Enter-VsDevShell -CommandType Cmdlet -ErrorAction Stop
if (!$enter.Parameters.ContainsKey('DevCmdArguments')) { throw 'Visual Studio DevShell does not support explicit SDK selection' }
& $enter -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64 -winsdk=10.0.26100.0'
if ($env:WindowsSDKVersion.TrimEnd('\') -ne '10.0.26100.0') { throw 'Pinned Windows SDK 10.0.26100.0 was not selected' }
if ($env:VSCMD_ARG_TGT_ARCH -ne 'x64') { throw 'MSVC target architecture must be x64' }
if (!(Get-Command rc.exe -ErrorAction SilentlyContinue)) { throw 'Windows SDK resource compiler missing' }
# DevShell can prepend a separate stable Rust installation to PATH.
. (Join-Path $PSScriptRoot 'windows-rust.ps1')
if (!$env:TBG_INNO_COMPILER) { throw 'Run scripts/setup-windows-inno.ps1 before selecting the build environment' }
$inno = Get-Content (Join-Path (Split-Path $env:TBG_INNO_COMPILER) 'tbg-toolchain.json') -Raw | ConvertFrom-Json
[ordered]@{
    os = [Environment]::OSVersion.VersionString; visualStudio = $installation
    toolset = $env:VCToolsVersion; sdk = $env:WindowsSDKVersion; rust = $rust
    inno = $inno; innoCompiler = $env:TBG_INNO_COMPILER
} | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $env:RUNNER_TEMP 'windows-build-tools.json')
