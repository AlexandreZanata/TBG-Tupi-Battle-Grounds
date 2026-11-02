# Check tool selection and fail-closed provisioning without compiling Rust or changing global tools.
$ErrorActionPreference = 'Stop'
$stage = Join-Path $env:RUNNER_TEMP ('tbg-tool-tests-' + [Guid]::NewGuid().ToString('N'))
$names = @('CARGO_HOME', 'RUST_TOOLCHAIN', 'RUSTUP_TOOLCHAIN', 'PATH', 'GITHUB_ENV')
$previous = @{}
foreach ($name in $names) { $previous[$name] = [Environment]::GetEnvironmentVariable($name, 'Process') }
try {
    New-Item -ItemType Directory -Force -Path (Join-Path $stage 'cargo/bin') | Out-Null
    Set-Content (Join-Path $stage 'cargo/bin/rustup.exe') 'test fixture, never executed'
    $env:CARGO_HOME = Join-Path $stage 'cargo'
    $env:RUST_TOOLCHAIN = '1.99.0'
    $env:RUSTUP_TOOLCHAIN = 'stable'
    $env:GITHUB_ENV = Join-Path $stage 'github-env'
    function rustc { $global:LASTEXITCODE = 0; 'rustc 1.99.0 (fixture)' }
    . (Join-Path $PSScriptRoot 'windows-rust.ps1')
    if ($env:RUSTUP_TOOLCHAIN -ne '1.99.0' -or $env:PATH.Split(';')[0] -ne (Join-Path $env:CARGO_HOME 'bin')) { throw 'Floating DevShell tools were not replaced' }
    if ((Get-Content $env:GITHUB_ENV) -notcontains 'RUSTUP_TOOLCHAIN=1.99.0') { throw 'Pinned Rust selection did not persist for cache restore' }
    Write-Output 'PASS: DevShell stable selection replaced with the pinned rustup proxy/version'
    function rustc { $global:LASTEXITCODE = 0; 'rustc 1.99.1 (wrong fixture)' }
    $rejected = $false
    try { . (Join-Path $PSScriptRoot 'windows-rust.ps1') } catch { if ($_.Exception.Message -notmatch 'Rust toolchain mismatch') { throw }; $rejected = $true }
    if (!$rejected) { throw 'Unexpected Rust compiler version was accepted' }
    $env:RUST_TOOLCHAIN = 'stable'
    $rejected = $false
    try { . (Join-Path $PSScriptRoot 'windows-rust.ps1') } catch { if ($_.Exception.Message -notmatch 'exact Rust version') { throw }; $rejected = $true }
    if (!$rejected) { throw 'Floating Rust toolchain was accepted' }
    Write-Output 'PASS: mismatched and floating Rust compilers rejected'
    $bad = Join-Path $stage 'modified-inno.exe'
    Set-Content $bad 'modified distribution, never executed'
    $rejected = $false
    try { & (Join-Path $PSScriptRoot 'setup-windows-inno.ps1') -ToolsRoot $stage -InstallerPath $bad } catch { if ($_.Exception.Message -notmatch 'SHA-256 mismatch') { throw }; $rejected = $true }
    if (!$rejected) { throw 'Modified Inno distribution was accepted' }
    Write-Output 'PASS: modified compiler distribution rejected before execution'
} finally {
    Remove-Item Function:rustc -ErrorAction SilentlyContinue
    foreach ($name in $names) { [Environment]::SetEnvironmentVariable($name, $previous[$name], 'Process') }
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
}
