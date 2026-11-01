# Compile the real installer script with a tiny fixture before an expensive Rust build.
param([string]$Compiler = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe')
$ErrorActionPreference = 'Stop'
$stage = Join-Path $env:RUNNER_TEMP ('tbg-installer-preflight-' + [Guid]::NewGuid().ToString('N'))
try {
    $package = Join-Path $stage 'package'
    $output = Join-Path $stage 'output'
    $branding = Join-Path $package 'assets/branding'
    New-Item -ItemType Directory -Force -Path $branding,$output | Out-Null
    Copy-Item crates/tbg-client/assets/branding/tbg.ico $branding
    # This fixture is compiled, never installed, executed, uploaded or published.
    'Installer compile fixture, not a game executable' | Set-Content (Join-Path $package 'tbg-client.exe')
    ./scripts/package-windows-installer.ps1 -Package $package -Version '0.0.0-preflight' -Output $output -Compiler $Compiler
    Write-Output 'PASS: reviewed Inno compiler, languages, constants, lock handling and installer script compile'
} finally {
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
}
