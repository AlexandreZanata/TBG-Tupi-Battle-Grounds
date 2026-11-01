# Compile a per-user Inno Setup installer from the already verified portable package.
param([Parameter(Mandatory=$true)][string]$Package, [Parameter(Mandatory=$true)][string]$Version)
$ErrorActionPreference = 'Stop'
$compiler = 'C:\Program Files (x86)\Inno Setup 6\ISCC.exe'
if ((Get-Item $compiler).VersionInfo.ProductVersion -notmatch '^6\.7\.1(?:\.|$)') { throw 'Reviewed Inno Setup 6.7.1 is required; update the tool review explicitly' }
$root = (Resolve-Path $Package).Path
$output = (Resolve-Path 'target/native').Path
& $compiler "/DPackageDir=$root" "/DBuildVersion=$Version" "/DOutputDir=$output" deploy/native/windows/tbg.iss
if ($LASTEXITCODE) { throw 'Installer compilation failed' }
$installer = Join-Path $output "tupi-battle-grounds-$Version-windows-x86_64-setup.exe"
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $installer).Hash.ToLowerInvariant()
"$hash  $([IO.Path]::GetFileName($installer))" | Set-Content -Encoding ascii "$installer.sha256"
