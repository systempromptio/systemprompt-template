# package.ps1: builds and packs the per-user Windows installer of one (brand, channel) build of the harness.
#
#   dist/<scheme>-<version>-windows-x64-setup.exe   the installer (packaging/harness/windows/harness.nsi)
#   dist/SHA256SUMS                                 its SHA-256, in sha256sum format
#
# The identity table below is the packaging side of extensions/harness/brand/src/identity.rs (sp-brand): display name,
# vendor, app id and URL scheme per brand and channel. The two change together; the version is read from sp-brand
# itself. The cargo build turns on the matching sp-brand features, so the binary and its installer always carry the
# same identity.
#
# NSIS: -Nsis <dir>, else makensis on PATH, else NSIS's default install folder, else a pinned portable NSIS that is
# downloaded, checked against its SHA-256 and kept in extensions\harness\target\tools (never installed system-wide).
# Nothing is signed.
#
# -TestInstall installs the result silently with /FILESONLY into a fresh temporary directory, compares the payload
# with the build, uninstalls the same way and checks that only the uninstaller is left.
#
# Usage:
#   pwsh -File packaging\harness\windows\package.ps1 [-Brand systemprompt] [-Channel internal|canary|dev]
#        [-Version X.Y.Z] [-SkipBuild] [-Nsis DIR] [-TestInstall]
param(
    [ValidateSet("systemprompt")]
    [string]$Brand = "systemprompt",
    [ValidateSet("internal", "canary", "dev")]
    [string]$Channel = "internal",
    [string]$Version = "",
    [switch]$SkipBuild,
    [string]$Nsis = "",
    [switch]$TestInstall
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version 3

$root = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$workspace = Join-Path $root "extensions\harness"
$dist = Join-Path $root "dist"
$release = Join-Path $workspace "target\release"
$nsiScript = Join-Path $PSScriptRoot "harness.nsi"
$iconDir = Join-Path $workspace "shell\assets\icons"
$brandSource = Join-Path $workspace "brand\src\lib.rs"
$exe = "harness.exe"

$brands = @{
    systemprompt = @{ Product = "systemprompt Harness"; Vendor = "systemprompt"; AppId = "io.systemprompt.harness"; Scheme = "sp-harness"; Feature = $null }
}
$channels = @{
    internal = @{ Display = ""; AppId = ""; Scheme = ""; Feature = $null }
    canary   = @{ Display = " Canary"; AppId = ".canary"; Scheme = "-canary"; Feature = "channel-canary" }
    dev      = @{ Display = " Dev"; AppId = ".dev"; Scheme = "-dev"; Feature = "channel-dev" }
}

$b = $brands[$Brand]
$c = $channels[$Channel]
$product = $b.Product + $c.Display
$appId = $b.AppId + $c.AppId
$scheme = $b.Scheme + $c.Scheme

if (-not $Version) {
    $source = Get-Content $brandSource -Raw
    if ($source -notmatch '(?m)^const VERSION: &str = "([^"]+)";') { throw "no 'const VERSION' in $brandSource" }
    $Version = $Matches[1]
}
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "version '$Version' is not MAJOR.MINOR.PATCH" }

$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x64" }
    default { throw "only Windows x64 is packed; this host is $env:PROCESSOR_ARCHITECTURE" }
}

# Portable NSIS 3.13 from SourceForge.
$nsisVersion = "3.13"
$nsisSha256 = "ba63dffc4410ee89193e1cb5a41989991bd77c61068da17e3156d136b7b0b3d8"
$nsisUrl = "https://downloads.sourceforge.net/project/nsis/NSIS%203/$nsisVersion/nsis-$nsisVersion.zip"

function Get-Makensis {
    if ($Nsis) {
        $candidate = Join-Path $Nsis "makensis.exe"
        if (-not (Test-Path $candidate)) { throw "no makensis.exe in $Nsis" }
        return $candidate
    }
    $command = Get-Command makensis -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    foreach ($folder in @(${env:ProgramFiles(x86)}, $env:ProgramFiles)) {
        if (-not $folder) { continue }
        $candidate = Join-Path $folder "NSIS\makensis.exe"
        if (Test-Path $candidate) { return $candidate }
    }

    $tools = Join-Path $workspace "target\tools"
    $portable = Join-Path $tools "nsis-$nsisVersion"
    $candidate = Join-Path $portable "makensis.exe"
    if (Test-Path $candidate) { return $candidate }

    $scratch = Join-Path $tools "nsis-download"
    if (Test-Path $scratch) { Remove-Item -Recurse -Force $scratch }
    New-Item -ItemType Directory -Force $scratch | Out-Null
    $zip = Join-Path $scratch "nsis.zip"
    Write-Host "fetching NSIS $nsisVersion"
    $ProgressPreference = "SilentlyContinue"
    # Why: SourceForge serves its HTML mirror page to browser user agents; a plain agent gets the archive.
    Invoke-WebRequest -UseBasicParsing -UserAgent "Wget" -Uri $nsisUrl -OutFile $zip
    $actual = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower()
    if ($actual -ne $nsisSha256) { throw "NSIS archive SHA-256 is $actual, expected $nsisSha256" }
    Expand-Archive -Path $zip -DestinationPath $scratch
    # Why: a build cache can restore this folder without makensis.exe; moving onto an existing folder nests the new one inside it.
    if (Test-Path $portable) { Remove-Item -Recurse -Force $portable }
    Move-Item (Join-Path $scratch "nsis-$nsisVersion") $portable
    Remove-Item -Recurse -Force $scratch
    if (-not (Test-Path $candidate)) { throw "the NSIS archive holds no makensis.exe" }
    return $candidate
}

function Write-Utf8NoBom([string]$Path, [string]$Text) {
    [System.IO.File]::WriteAllText($Path, $Text, (New-Object System.Text.UTF8Encoding $false))
}

if (-not $SkipBuild) {
    $features = @($b.Feature, $c.Feature) | Where-Object { $_ } | ForEach-Object { "sp-brand/$_" }
    $cargoArgs = @("build", "--release", "-p", "harness-desktop")
    if ($features) { $cargoArgs += @("--features", ($features -join ",")) }
    Write-Host "cargo $($cargoArgs -join ' ')"
    Push-Location $workspace
    try {
        & cargo @cargoArgs
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }
    } finally {
        Pop-Location
    }
}

$built = Join-Path $release $exe
if (-not (Test-Path $built)) { throw "missing $built; build first or drop -SkipBuild" }

$icon = Join-Path $iconDir "icon-$Brand-$Channel.ico"
if (-not (Test-Path $icon)) { $icon = Join-Path $iconDir "icon.ico" }
$hasIcon = Test-Path $icon
if (-not $hasIcon) { Write-Warning "no icon in $iconDir; the installer keeps the NSIS icon and installs no app.ico" }

$stage = Join-Path $workspace "target\installer-stage\$Brand-$Channel"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item $built (Join-Path $stage $exe)
if ($hasIcon) { Copy-Item $icon (Join-Path $stage "app.ico") }

New-Item -ItemType Directory -Force $dist | Out-Null
$installerName = "$scheme-$Version-windows-$arch-setup.exe"
$installer = Join-Path $dist $installerName
if (Test-Path $installer) { Remove-Item -Force $installer }

$makensis = Get-Makensis
$defines = @(
    "/DSTAGE=$stage",
    "/DOUTFILE=$installer",
    "/DVERSION=$Version",
    "/DPRODUCT=$product",
    "/DVENDOR=$($b.Vendor)",
    "/DAPP_ID=$appId",
    "/DSCHEME=$scheme",
    "/DINSTALL_NAME=$product"
)
if ($hasIcon) { $defines += "/DAPP_ICON=app.ico" }
& $makensis /V2 /INPUTCHARSET UTF8 @defines $nsiScript
if ($LASTEXITCODE -ne 0) { throw "makensis failed with exit code $LASTEXITCODE" }

if ($TestInstall) {
    $target = Join-Path ([System.IO.Path]::GetTempPath()) ("harness-install-test-" + [System.Guid]::NewGuid().ToString("N"))
    try {
        # /D= must be the last argument and is taken verbatim, spaces included.
        $run = Start-Process -FilePath $installer -ArgumentList @("/S", "/FILESONLY", "/D=$target") -Wait -PassThru
        if ($run.ExitCode -ne 0) { throw "silent install exited with $($run.ExitCode)" }
        $present = @(Get-ChildItem $target | ForEach-Object Name | Sort-Object) -join ","
        $expected = @($exe, "uninstall.exe")
        if ($hasIcon) { $expected += "app.ico" }
        $expected = $expected | Sort-Object
        if ($present -ne ($expected -join ",")) { throw "installed files: $present" }
        if ((Get-FileHash (Join-Path $target $exe)).Hash -ne (Get-FileHash $built).Hash) { throw "$exe differs from the build" }
        # Why: without _?= the NSIS uninstaller copies itself to %TEMP% and returns before it has finished.
        $run = Start-Process -FilePath (Join-Path $target "uninstall.exe") -ArgumentList @("/S", "/FILESONLY", "_?=$target") -Wait -PassThru
        if ($run.ExitCode -ne 0) { throw "silent uninstall exited with $($run.ExitCode)" }
        $left = @(Get-ChildItem $target | ForEach-Object Name) -join ","
        if ($left -ne "uninstall.exe") { throw "left after uninstall: $left" }
        Write-Host "test install: ok"
    } finally {
        if (Test-Path $target) { Remove-Item -Recurse -Force $target }
    }
}

$hash = (Get-FileHash -Algorithm SHA256 $installer).Hash.ToLower()
Write-Utf8NoBom (Join-Path $dist "SHA256SUMS") ("{0}  {1}`n" -f $hash, $installerName)
"{0}  {1:N1} MB  sha256 {2}" -f $installerName, ((Get-Item $installer).Length / 1MB), $hash
