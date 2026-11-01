# Inspect the exact MSVC executable without relying on a graphical launch or an installed VC runtime.
param([Parameter(Mandatory=$true)][string]$Binary)
$ErrorActionPreference = 'Stop'
$bytes = [IO.File]::ReadAllBytes((Resolve-Path $Binary))
$pe = [BitConverter]::ToInt32($bytes, 0x3c)
if ([BitConverter]::ToUInt32($bytes, $pe) -ne 0x00004550) { throw 'Not a PE executable' }
if ([BitConverter]::ToUInt16($bytes, $pe + 4) -ne 0x8664) { throw 'PE architecture must be x64' }
if ([BitConverter]::ToUInt16($bytes, $pe + 24 + 68) -ne 2) { throw 'Release must use the Windows GUI subsystem' }
$info = (Get-Item $Binary).VersionInfo
if ($info.ProductName -ne 'TBG' -or !$info.ProductVersion) { throw 'Missing embedded product/version resources' }
$imports = (& dumpbin.exe /nologo /dependents $Binary | Out-String)
if ($LASTEXITCODE) { throw 'PE import inspection failed' }
if ($imports -match '(?i)(VCRUNTIME\d|MSVCP\d|ucrtbased|libgcc|libstdc\+\+|libwinpthread)') { throw 'Unexpected developer/runtime DLL dependency in the portable candidate' }
$imports | Set-Content -Encoding utf8 (Join-Path $env:RUNNER_TEMP 'windows-imports.txt')
Write-Output "Verified PE x64, GUI subsystem, resources, static CRT policy: $($info.ProductVersion)"
