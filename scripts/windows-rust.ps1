# Restore the reviewed rustup proxy/toolchain after DevShell changes PATH.
$ErrorActionPreference = 'Stop'
if ($env:RUST_TOOLCHAIN -notmatch '^\d+\.\d+\.\d+$') { throw 'RUST_TOOLCHAIN must name the reviewed exact Rust version' }
$env:RUSTUP_TOOLCHAIN = $env:RUST_TOOLCHAIN
$rustupBin = if ($env:CARGO_HOME) { Join-Path $env:CARGO_HOME 'bin' } else { Join-Path $env:USERPROFILE '.cargo/bin' }
if (!(Test-Path (Join-Path $rustupBin 'rustup.exe'))) { throw 'The selected rustup proxy directory is missing' }
$env:PATH = "$rustupBin;$env:PATH"
$rust = (& rustc -V | Out-String).Trim()
if ($LASTEXITCODE -or $rust -notmatch ('^rustc ' + [regex]::Escape($env:RUST_TOOLCHAIN) + ' ')) { throw "Rust toolchain mismatch: expected $env:RUST_TOOLCHAIN, got $rust" }
if ($env:GITHUB_ENV) { "RUSTUP_TOOLCHAIN=$env:RUST_TOOLCHAIN" >> $env:GITHUB_ENV }
