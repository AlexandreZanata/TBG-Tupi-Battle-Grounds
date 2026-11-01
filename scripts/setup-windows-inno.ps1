# Provision the reviewed, digest-pinned compiler without changing the runner's global tools.
param([string]$ToolsRoot = $env:RUNNER_TEMP, [string]$InstallerPath)
$ErrorActionPreference = 'Stop'
if (!$ToolsRoot) { throw 'Set RUNNER_TEMP or pass a private ToolsRoot directory' }
$spec = Get-Content (Join-Path $PSScriptRoot '../deploy/native/windows/inno-toolchain.json') -Raw | ConvertFrom-Json
$stage = Join-Path $ToolsRoot ('tbg-inno-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $stage -Force | Out-Null
$setup = Join-Path $stage 'setup.exe'
if ($InstallerPath) {
    Copy-Item -LiteralPath $InstallerPath -Destination $setup
} else {
    Invoke-WebRequest -Uri $spec.url -OutFile $setup
}
$hash = (Get-FileHash -LiteralPath $setup -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hash -ne $spec.sha256) { throw "Inno distribution SHA-256 mismatch: expected $($spec.sha256), got $hash" }
$destination = Join-Path $stage 'compiler'
$log = Join-Path $stage 'setup.log'
$arguments = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP- /NOICONS /TASKS="" /CURRENTUSER /PORTABLE=1 /DIR="' + $destination + '" /LOG="' + $log + '"'
# Wait only for this installer; Start-Process -Wait can also wait for shell descendants.
$process = Start-Process -FilePath $setup -ArgumentList $arguments -PassThru
if (!$process.WaitForExit(60000)) {
    $process.Kill()
    throw "Pinned Inno portable setup timed out; inspect $log"
}
if ($process.ExitCode -ne 0) { throw "Pinned Inno portable setup exited $($process.ExitCode); inspect $log" }
$compiler = Join-Path $destination 'ISCC.exe'
$engine = Join-Path $destination 'ISCmplr.dll'
foreach ($file in @($compiler, $engine, (Join-Path $destination 'ISPP.dll'))) {
    if (!(Test-Path -LiteralPath $file -PathType Leaf)) { throw "Incomplete pinned Inno distribution: $file" }
}
[ordered]@{
    version = $spec.version; url = $spec.url; distributionSha256 = $hash
    compilerSha256 = (Get-FileHash -LiteralPath $compiler -Algorithm SHA256).Hash.ToLowerInvariant()
    engineSha256 = (Get-FileHash -LiteralPath $engine -Algorithm SHA256).Hash.ToLowerInvariant()
} | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $destination 'tbg-toolchain.json')
$env:TBG_INNO_COMPILER = $compiler
if ($env:GITHUB_ENV) { "TBG_INNO_COMPILER=$compiler" >> $env:GITHUB_ENV }
Write-Output "Provisioned reviewed Inno $($spec.version) from SHA-256 $hash at $compiler"
