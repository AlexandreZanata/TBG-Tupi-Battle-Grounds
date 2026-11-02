# Compile a per-user Inno Setup installer from the already verified portable package.
param(
    [Parameter(Mandatory=$true)][string]$Package,
    [Parameter(Mandatory=$true)][string]$Version,
    [string]$Output = 'target/native',
    [string]$Compiler = $env:TBG_INNO_COMPILER
)
$ErrorActionPreference = 'Stop'
if (!$Compiler) { throw 'Run scripts/setup-windows-inno.ps1 or pass the reviewed Compiler explicitly' }
Get-Item -LiteralPath $Compiler -ErrorAction Stop | Out-Null
# The installer script checks PREPROCVER before accepting any package inputs.
$root = (Resolve-Path $Package).Path
$destination = (Resolve-Path $Output).Path
& $Compiler "/DPackageDir=$root" "/DBuildVersion=$Version" "/DOutputDir=$destination" deploy/native/windows/tbg.iss
if ($LASTEXITCODE) { throw 'Installer compilation failed' }
$installer = Join-Path $destination "tupi-battle-grounds-$Version-windows-x86_64-setup.exe"
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $installer).Hash.ToLowerInvariant()
"$hash  $([IO.Path]::GetFileName($installer))" | Set-Content -Encoding ascii "$installer.sha256"
