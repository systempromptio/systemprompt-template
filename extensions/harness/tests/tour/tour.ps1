# tour.ps1: the screenshot tour on Windows (S2-C1; core testing/05-evidence.md section 2), the twin of tour.sh.
#
# Installs the installer packaging/harness/windows/package.ps1 produced, silently, into a tour folder; starts the
# scripted instance (built from a core checkout at the rev extensions/harness/Cargo.toml pins); signs in with
# "harness login --stdin"; seeds sessions with "harness run" (seeds.txt); then for each state in states.txt reachable
# by launch or deep link: brings the window to it, waits, captures the window (user32 + System.Drawing) and checks the
# capture is not blank and matches the window's size. States no link reaches go to the manual-walk list. The user's
# data root is moved aside for the tour and put back after. Same outputs as tour.sh.
#
# Usage:
#   powershell -NoProfile -File extensions\harness\tests\tour\tour.ps1 -Out DIR [-Core DIR] [-Installer PATH]
#        [-Settle SECONDS] [-DryRun]
param(
    [Parameter(Mandatory = $true)][string]$Out,
    [string]$Core = "",
    [string]$Installer = "",
    [int]$Settle = 4,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version 3

$here = $PSScriptRoot
$root = (Resolve-Path (Join-Path $here "..\..\..\..")).Path
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path

$scheme = "sp-harness"
$displayName = "systemprompt Harness"
$localData = [Environment]::GetFolderPath("LocalApplicationData")
if (-not $localData) { $localData = Join-Path $HOME "AppData\Local" }
$dataRoot = Join-Path $localData "systemprompt\$displayName"
$cargoToml = Get-Content (Join-Path $root "extensions\harness\Cargo.toml")
$pinnedRev = ($cargoToml | Select-String '^systemprompt-harness = .*rev = "([0-9a-f]+)"').Matches[0].Groups[1].Value
$results = Join-Path $Out "tour.tsv"
$manual = Join-Path $Out "manual-walk.txt"
Set-Content -Path $results -Value "number|slug|theme|status|file|note" -Encoding utf8
Set-Content -Path $manual -Value $null -Encoding utf8
$script:failures = 0
$script:sessions = @{}
$script:signedIn = $false
$script:blocked = ""
$script:proc = $null
$installDir = Join-Path $localData "sp-harness-tour"
$binary = Join-Path $installDir "harness.exe"
$project = Join-Path $Out ".workspace"
$backup = ""
$instance = $null

function Write-Record($number, $slug, $theme, $status, $file, $note) {
    Add-Content -Path $results -Value ("{0}|{1}|{2}|{3}|{4}|{5}" -f $number, $slug, $theme, $status, $file, $note) -Encoding utf8
    if ($status -eq "failed" -or $status -eq "blocked") { $script:failures++ }
    $suffix = ""
    if ($note) { $suffix = " - $note" }
    Write-Host "  $number $slug ($theme): $status$suffix"
}

Add-Type -AssemblyName System.Drawing
if (-not $DryRun) {
    Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class TourWin {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}
"@
    [TourWin]::SetProcessDPIAware() | Out-Null
}

# Returns "" when the capture passes, else why not.
function Test-Capture($file, $wantW, $wantH) {
    if (-not (Test-Path $file)) { return "no capture written" }
    $bitmap = [System.Drawing.Bitmap]::FromFile($file)
    try {
        if ($bitmap.Width -ne $wantW -or $bitmap.Height -ne $wantH) {
            return ("capture is {0}x{1}, the window {2}x{3}" -f $bitmap.Width, $bitmap.Height, $wantW, $wantH)
        }
        $first = $bitmap.GetPixel(0, 0).ToArgb()
        for ($y = 0; $y -lt $bitmap.Height; $y += [Math]::Max(1, [int]($bitmap.Height / 40))) {
            for ($x = 0; $x -lt $bitmap.Width; $x += [Math]::Max(1, [int]($bitmap.Width / 40))) {
                if ($bitmap.GetPixel($x, $y).ToArgb() -ne $first) { return "" }
            }
        }
        return "capture looks blank (every sampled pixel is one colour)"
    } finally { $bitmap.Dispose() }
}

function Save-Capture($number, $slug, $theme) {
    $file = Join-Path $Out "$number-$slug-$theme.png"
    if ($DryRun) {
        Copy-Item (Join-Path $here "fixture\screenshot.png") $file
        $probe = [System.Drawing.Image]::FromFile($file)
        $w = $probe.Width; $h = $probe.Height
        $probe.Dispose()
    } else {
        $script:proc.Refresh()
        $handle = $script:proc.MainWindowHandle
        if ($handle -eq [IntPtr]::Zero) { Write-Record $number $slug $theme "failed" "" "no window found"; return }
        [TourWin]::SetForegroundWindow($handle) | Out-Null
        Start-Sleep -Seconds 1
        $rect = New-Object TourWin+RECT
        [TourWin]::GetWindowRect($handle, [ref]$rect) | Out-Null
        $w = $rect.Right - $rect.Left; $h = $rect.Bottom - $rect.Top
        $bitmap = New-Object System.Drawing.Bitmap $w, $h
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
        $bitmap.Save($file, [System.Drawing.Imaging.ImageFormat]::Png)
        $graphics.Dispose(); $bitmap.Dispose()
    }
    $problem = Test-Capture $file $w $h
    if ($problem) { Write-Record $number $slug $theme "failed" (Split-Path $file -Leaf) $problem }
    else { Write-Record $number $slug $theme "captured" (Split-Path $file -Leaf) "" }
}

function Set-Theme($theme) {
    New-Item -ItemType Directory -Force (Join-Path $dataRoot "state") | Out-Null
    Set-Content -Path (Join-Path $dataRoot "state\ui-settings.json") -Value ('{"scheme":"' + $theme + '"}') -Encoding ascii
}

function Start-App {
    $script:proc = Start-Process -FilePath $binary -PassThru
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Seconds 1
        $script:proc.Refresh()
        if ($script:proc.MainWindowHandle -ne [IntPtr]::Zero) { Start-Sleep -Seconds $Settle; return $true }
    }
    return $false
}

function Stop-App {
    Get-Process -Name harness -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $binary } | ForEach-Object {
        $_.CloseMainWindow() | Out-Null
        if (-not $_.WaitForExit(10000)) { $_.Kill() }
    }
}

function Open-Link($target) {
    if ($DryRun) { Write-Host "    would open $($scheme)://$target"; return }
    Start-Process "$($scheme)://$target"
    Start-Sleep -Seconds $Settle
}

function Expand-Target($target) {
    $target = $target.Replace("{project}", $project)
    foreach ($name in @($script:sessions.Keys)) { $target = $target.Replace("{session:$name}", $script:sessions[$name]) }
    return $target
}

function Invoke-SignIn {
    if (Test-Path $project) { Remove-Item -Recurse -Force $project }
    Copy-Item -Recurse (Join-Path $here "workspace") $project
    if ($DryRun) {
        Write-Host "  would start the scripted instance from core at $pinnedRev and sign in with login --stdin"
        $script:signedIn = $true
        return
    }
    if (-not (Select-String -Path $binary -Pattern "login --stdin" -SimpleMatch -Quiet)) {
        $script:blocked = "this harness binary does not dispatch login/run (core command module not at the pinned rev)"; return
    }
    $coreHead = ""
    if ($Core) { $coreHead = (& git -C $Core rev-parse HEAD 2>$null) }
    if ($coreHead -ne $pinnedRev) { $script:blocked = "no core checkout at $pinnedRev (-Core) to build the scripted instance from"; return }
    $target = Join-Path $Out ".instance-target"
    & cargo build --release --manifest-path (Join-Path $Core "crates\tests\Cargo.toml") -p systemprompt-scripted-instance --target-dir $target *> (Join-Path $Out "instance-build.log")
    if ($LASTEXITCODE -ne 0) { $script:blocked = "scripted instance did not build (instance-build.log)"; return }
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = Join-Path $target "release\systemprompt-scripted-instance.exe"
    $info.Arguments = '--script "' + (Join-Path $here "instance\turns.json") + '" --messages-model tour-model'
    $info.RedirectStandardOutput = $true
    $info.UseShellExecute = $false
    $script:instance = [System.Diagnostics.Process]::Start($info)
    $line = $script:instance.StandardOutput.ReadLine() | ConvertFrom-Json
    $login = $line.pat | & $binary login --stdin --instance $line.base_url 2>&1
    $login | Set-Content (Join-Path $Out "login.jsonl") -Encoding utf8
    if ($LASTEXITCODE -ne 0) { $script:blocked = "login --stdin failed (login.jsonl)"; return }
    $script:signedIn = $true
    foreach ($seed in (Get-Content (Join-Path $here "seeds.txt") | Where-Object { $_ -and -not $_.StartsWith("#") })) {
        $name, $mode, $prompt = $seed.Split("|", 3)
        $log = Join-Path $Out "seed-$name.jsonl"
        & $binary run --workspace $project --prompt $prompt --permission-mode $mode --timeout 120 *> $log
        $started = Get-Content $log | Where-Object { $_.StartsWith('{"type":"started"') } | Select-Object -First 1
        if ($started) { $script:sessions[$name] = ($started | ConvertFrom-Json).session_id }
    }
}

$states = Get-Content (Join-Path $here "states.txt") | Where-Object { $_ -and -not $_.StartsWith("#") } | ForEach-Object {
    $f = $_.Split("|")
    [pscustomobject]@{ Number = $f[0]; Slug = $f[1]; Title = $f[2]; Themes = $f[3].Split(","); Reach = $f[4]; Target = $f[5]; Needs = $f[6] }
}

$note = ""
if ($DryRun) { $note = ", dry run" }
Write-Host "tour: $($states.Count) states, out $Out$note"
try {
    if (-not $DryRun) {
        if (-not $Installer) {
            $Installer = Get-ChildItem (Join-Path $root "dist") -Filter "$scheme-*-windows-*-setup.exe" | Sort-Object LastWriteTime | Select-Object -Last 1 -ExpandProperty FullName
        }
        if (-not $Installer) { throw "no installer in dist (run packaging\harness\windows\package.ps1)" }
        Stop-App
        $run = Start-Process -FilePath $Installer -ArgumentList @("/S", "/D=$installDir") -Wait -PassThru
        if ($run.ExitCode -ne 0) { throw "silent install exited with $($run.ExitCode)" }
        if (Test-Path $dataRoot) {
            $backup = "$dataRoot.tour-backup-" + (Get-Date -Format "yyyyMMddHHmmss")
            Move-Item $dataRoot $backup
        }
    }

    foreach ($s in ($states | Where-Object Reach -eq "manual")) {
        Add-Content -Path $manual -Value ("{0} {1} [{2}]" -f $s.Number, $s.Title, ($s.Themes -join ",")) -Encoding utf8
        Write-Record $s.Number $s.Slug ($s.Themes -join ",") "manual" "" "no deep link or command reaches it"
    }

    foreach ($theme in @("light", "dark")) {
        foreach ($s in ($states | Where-Object { $_.Reach -eq "launch" -and $_.Themes -contains $theme })) {
            if ($DryRun) { Write-Host "    would launch $displayName signed out ($theme)" }
            else {
                Set-Theme $theme
                if (-not (Start-App)) { Write-Record $s.Number $s.Slug $theme "failed" "" "the app did not start"; continue }
            }
            Save-Capture $s.Number $s.Slug $theme
            if (-not $DryRun) { Stop-App }
        }
    }

    Invoke-SignIn
    foreach ($theme in @("light", "dark")) {
        $launched = $false
        foreach ($s in ($states | Where-Object { $_.Reach -eq "link" -and $_.Themes -contains $theme })) {
            if ($s.Needs -ne "-" -and -not $script:signedIn) { Write-Record $s.Number $s.Slug $theme "blocked" "" $script:blocked; continue }
            if ($s.Needs.StartsWith("seed:")) {
                $seed = $s.Needs.Substring(5)
                if ($DryRun) { $script:sessions[$seed] = "ses_dry_run_$seed" }
                if (-not $script:sessions.ContainsKey($seed)) { Write-Record $s.Number $s.Slug $theme "blocked" "" "seed $seed made no session (seed-$seed.jsonl)"; continue }
            }
            if (-not $launched -and -not $DryRun) {
                Set-Theme $theme
                if (-not (Start-App)) { Write-Record $s.Number $s.Slug $theme "failed" "" "the app did not start"; continue }
                $launched = $true
            }
            Open-Link (Expand-Target $s.Target)
            Save-Capture $s.Number $s.Slug $theme
        }
        if ($launched) { Stop-App }
    }
} finally {
    if (-not $DryRun) {
        Stop-App
        if ($script:instance) { $script:instance.Kill() }
        if ($backup -and (Test-Path $backup)) {
            if (Test-Path $dataRoot) { Remove-Item -Recurse -Force $dataRoot }
            Move-Item $backup $dataRoot
        }
    }
}

$captured = @(Get-Content $results | Where-Object { $_ -like "*|captured|*" }).Count
$walk = @(Get-Content $manual | Where-Object { $_ }).Count
Write-Host "tour: $captured captured, $($script:failures) failed or blocked, $walk on the manual walk"
if ($script:failures -gt 0) { exit 1 }
exit 0
