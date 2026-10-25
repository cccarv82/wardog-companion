param(
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\dist'),
    [string]$ISCC = ''
)
$ErrorActionPreference = 'Stop'
$sourceRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$version = [regex]::Match((Get-Content (Join-Path $sourceRoot 'Cargo.toml') -Raw), '(?m)^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"').Groups[1].Value
if (!$version) { throw 'Missing package version' }
$exe = Join-Path $sourceRoot 'target\release\wardog-companion.exe'
if (!(Test-Path -LiteralPath $exe)) { throw 'Build first: cargo build --release --locked' }
if ((Get-Item $exe).VersionInfo.ProductVersion -ne $version) { throw 'Executable version does not match Cargo.toml; rebuild first' }
$outputRoot = [IO.Path]::GetFullPath($OutputDirectory)
$payload = Join-Path $outputRoot "WARDOG-Companion-$version-windows-x64"
if (Test-Path -LiteralPath $payload) { throw "Payload already exists. Use a fresh output directory: $payload" }
New-Item -ItemType Directory -Force $payload,(Join-Path $payload 'Data'),(Join-Path $payload 'About'),(Join-Path $payload 'assets'),(Join-Path $payload 'docs\images') | Out-Null
Copy-Item $exe (Join-Path $payload 'wardog-companion.exe')
Copy-Item (Join-Path $sourceRoot 'data\catalog.json') (Join-Path $payload 'Data\catalog.json')
foreach ($name in @('README.md','START_HERE.txt','LICENSE','THIRD_PARTY_NOTICES.md','CONTRIBUTING.md','CODE_OF_CONDUCT.md','SECURITY.md','MODELING_LIMITS.md','SOURCES.md','VALIDATION.md','UPDATE_GUIDE.md','RELEASE_NOTES.md')) {
    Copy-Item (Join-Path $sourceRoot $name) $payload
}
foreach ($name in @('MODELING_LIMITS.md','SOURCES.md','VALIDATION.md','UPDATE_GUIDE.md','RELEASE_NOTES.md')) {
    Copy-Item (Join-Path $sourceRoot $name) (Join-Path $payload 'About')
}
Copy-Item (Join-Path $sourceRoot 'assets\wardog.svg') (Join-Path $payload 'assets')
Copy-Item (Join-Path $sourceRoot 'docs\*.md') (Join-Path $payload 'docs')
Copy-Item (Join-Path $sourceRoot 'docs\images\*.png') (Join-Path $payload 'docs\images')
Copy-Item (Join-Path $sourceRoot 'data\WARDOGS-DATA-LICENSE.txt') (Join-Path $payload 'About')
Copy-Item (Join-Path $sourceRoot 'research\wardogs-data\MANIFEST.json') (Join-Path $payload 'About\source-manifest.json')
Copy-Item (Join-Path $sourceRoot 'research\wardogs-data\csv-weapon-damage.csv') (Join-Path $payload 'About\weapon-damage.csv')
Copy-Item (Join-Path $sourceRoot 'research\muzzle-compatibility-all.json') (Join-Path $payload 'About')
Copy-Item (Join-Path $sourceRoot 'data\update-report.json') (Join-Path $payload 'About')
$audit = Start-Process -FilePath $exe -ArgumentList '--audit-output',('"' + (Join-Path $payload 'About\ranking-audit.json') + '"') -WindowStyle Hidden -Wait -PassThru
if ($audit.ExitCode -ne 0) { throw 'Model audit failed' }
Get-ChildItem $payload -Recurse -File | Sort-Object FullName | ForEach-Object {
    '{0}  {1}' -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(),$_.FullName.Substring($payload.Length+1).Replace('\','/')
} | Set-Content (Join-Path $payload 'About\SHA256.txt') -Encoding utf8
$zip = Join-Path $outputRoot "WARDOG-Companion-$version-windows-x64.zip"
Compress-Archive -LiteralPath $payload -DestinationPath $zip
if ($ISCC) {
    & $ISCC "/DAppVersion=$version" "/DPayloadDir=$payload" "/DReleaseDir=$outputRoot" (Join-Path $sourceRoot 'installer\wardog.iss')
    if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }
}
Get-ChildItem $outputRoot -File | Where-Object { $_.Extension -in @('.zip','.exe') } | Sort-Object Name | ForEach-Object {
    '{0}  {1}' -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(),$_.Name
} | Set-Content (Join-Path $outputRoot 'SHA256SUMS.txt') -Encoding utf8
Write-Output "Release $version packaged in $outputRoot"
