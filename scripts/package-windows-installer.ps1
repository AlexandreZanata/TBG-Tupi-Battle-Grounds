# Compile a per-user Inno Setup installer from the already verified portable package.
param(
    [Parameter(Mandatory=$true)][string]$Package,
    [Parameter(Mandatory=$true)][string]$Version,
    [string]$Output = 'target/native',
    [string]$Compiler = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
)
$ErrorActionPreference = 'Stop'
if ((Get-Item $Compiler).VersionInfo.ProductVersion -notmatch '^6\.7\.1(?:\.|$)') { throw 'Reviewed Inno Setup 6.7.1 is required; update the tool review explicitly' }
$root = (Resolve-Path $Package).Path
$destination = (Resolve-Path $Output).Path
& $Compiler "/DPackageDir=$root" "/DBuildVersion=$Version" "/DOutputDir=$destination" deploy/native/windows/tbg.iss
if ($LASTEXITCODE) { throw 'Installer compilation failed' }
$installer = Join-Path $destination "tupi-battle-grounds-$Version-windows-x86_64-setup.exe"
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $installer).Hash.ToLowerInvariant()
"$hash  $([IO.Path]::GetFileName($installer))" | Set-Content -Encoding ascii "$installer.sha256"
