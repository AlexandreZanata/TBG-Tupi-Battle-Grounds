# Select the reviewed MSVC x64 environment; record the actual hosted runner tools.
$ErrorActionPreference = 'Stop'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$installation) { throw 'MSVC x64 toolset is required' }
& (Join-Path $installation 'Common7\Tools\Launch-VsDevShell.ps1') -Arch amd64 -HostArch amd64 -SkipAutomaticLocation -DevCmdArguments "-winsdk=10.0.26100.0"
if (!(Get-Command rc.exe -ErrorAction SilentlyContinue)) { throw 'Windows SDK resource compiler missing' }
[ordered]@{ os = [Environment]::OSVersion.VersionString; visualStudio = $installation; toolset = $env:VCToolsVersion; sdk = $env:WindowsSDKVersion; rust = (& rustc -V); inno = (Get-Item 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe').VersionInfo.ProductVersion } | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $env:RUNNER_TEMP 'windows-build-tools.json')
