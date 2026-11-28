# run.ps1: the evidence command on Windows (S2-C2; core testing/05-evidence.md section 3), the twin of run (bash).
#
# Runs the steps in steps.txt on this machine and writes the same local, untracked folder
# evidence\<host>-<timestamp>\ (index.html, summary.tsv, meta.txt, logs\, screenshots\, package\). Nothing is
# uploaded. Exits 1 if any step failed, and names them. Steps whose command is bash need Git for Windows' bash on PATH.
#
# Usage:
#   powershell -NoProfile -File scripts\harness\evidence\run.ps1 [-Core DIR] [-OutRoot DIR] [-DryRun [-Fixture DIR]]
param(
    [string]$Core = "",
    [string]$OutRoot = "",
    [switch]$DryRun,
    [string]$Fixture = ""
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version 3

$here = $PSScriptRoot
$root = (Resolve-Path (Join-Path $here "..\..\..")).Path
if (-not $OutRoot) { $OutRoot = Join-Path $root "evidence" }
if (-not $Fixture) { $Fixture = Join-Path $here "fixture\pass" }
$pwshPath = (Get-Process -Id $PID).Path
$cargoToml = Get-Content (Join-Path $root "extensions\harness\Cargo.toml")
$pinnedRev = ($cargoToml | Select-String '^systemprompt-harness = .*rev = "([0-9a-f]+)"').Matches[0].Groups[1].Value

function Test-AtRev($dir) {
    if (-not $dir -or -not (Test-Path $dir)) { return $false }
    return ((& git -C $dir rev-parse HEAD 2>$null) -eq $pinnedRev)
}
if (-not $Core) {
    foreach ($candidate in @((Join-Path $root "..\systemprompt-core"), (Join-Path $root "..\core-future"))) {
        if (Test-AtRev $candidate) { $Core = (Resolve-Path $candidate).Path; break }
    }
}
$coreOk = Test-AtRev $Core

$machine = $env:COMPUTERNAME
$started = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ")
$out = Join-Path $OutRoot "$machine-$started"
foreach ($sub in @("logs", "screenshots", "package")) { New-Item -ItemType Directory -Force (Join-Path $out $sub) | Out-Null }
$summary = Join-Path $out "summary.tsv"
Set-Content -Path $summary -Value "step|status|exit|seconds" -Encoding utf8
$failed = @()

$commit = (& git -C $root rev-parse HEAD)
& git -C $root diff --quiet HEAD 2>$null
if ($LASTEXITCODE -ne 0) { $commit += " (working tree differs)" }
$coreLine = "core: none at the pinned rev (core steps skipped)"
if ($coreOk) { $coreLine = "core: $Core at $pinnedRev" }
Push-Location (Join-Path $root "extensions\harness")
try { $toolchain = (& rustc --version 2>$null) } catch { $toolchain = "unavailable" }
Pop-Location
$mode = "real"
if ($DryRun) { $mode = "dry run, fixture $Fixture" }
$meta = @(
    "host: $machine",
    "system: $([Environment]::OSVersion.VersionString) $env:PROCESSOR_ARCHITECTURE",
    "commit: $commit",
    "pinned core rev: $pinnedRev",
    $coreLine,
    "toolchain: $toolchain",
    "inference spend: none; no provider credential is used by any step",
    "mode: $mode",
    "started: $started"
)
Set-Content -Path (Join-Path $out "meta.txt") -Value $meta -Encoding utf8

function Expand-Command($command) {
    $dry = ""
    if ($DryRun) { $dry = "-DryRun" }
    return $command.Replace("{root}", $root).Replace("{core}", $Core).Replace("{out}", $out).Replace("{dry}", $dry).Replace("{pwsh}", '"' + $pwshPath + '"')
}

foreach ($line in (Get-Content (Join-Path $here "steps.txt"))) {
    if (-not $line -or $line.StartsWith("#")) { continue }
    $id, $needs, $cwd, $bashCommand, $psCommand = $line.Split("|", 5)
    $log = Join-Path $out "logs\$id.log"
    if ($needs.StartsWith("core:") -and (-not $coreOk -or -not (Test-Path (Join-Path $Core $needs.Substring(5))))) {
        Set-Content -Path $log -Value "skipped: needs a core checkout at $pinnedRev with $($needs.Substring(5))" -Encoding utf8
        Add-Content -Path $summary -Value "$id|skipped||0" -Encoding utf8
        Write-Host "== $($id): skipped"
        continue
    }
    $command = Expand-Command $psCommand
    $dir = $root
    if ($cwd -eq "core") { $dir = $Core }
    Write-Host "== $($id): $command"
    $clock = [System.Diagnostics.Stopwatch]::StartNew()
    if ($DryRun -and -not $psCommand.Contains("{dry}")) {
        $code = 0
        $exitFile = Join-Path $Fixture "$id.exit"
        if (Test-Path $exitFile) { $code = [int](Get-Content $exitFile -Raw).Trim() }
        $text = @("dry run: would run in $($dir): $command")
        $fixtureLog = Join-Path $Fixture "$id.log"
        if (Test-Path $fixtureLog) { $text += Get-Content $fixtureLog }
        Set-Content -Path $log -Value $text -Encoding utf8
    } else {
        Push-Location $dir
        try {
            $global:LASTEXITCODE = 0
            $ErrorActionPreference = "Continue"
            Invoke-Expression $command *> $log
            $code = $LASTEXITCODE
            if ($null -eq $code) { $code = 0 }
        } catch {
            Add-Content -Path $log -Value $_.ToString()
            $code = 1
        } finally {
            $ErrorActionPreference = "Stop"
            Pop-Location
        }
    }
    $seconds = [int]$clock.Elapsed.TotalSeconds
    if ($code -eq 0) { $status = "passed" } else { $status = "failed"; $failed += $id }
    Add-Content -Path $summary -Value "$id|$status|$code|$seconds" -Encoding utf8
    Write-Host "   $status (exit $code, $($seconds)s)"
}

$sums = Join-Path $root "dist\SHA256SUMS"
if ((Test-Path $sums) -and -not $DryRun) {
    Copy-Item $sums (Join-Path $out "package")
    Get-ChildItem (Join-Path $root "dist") | Format-Table Name, Length, LastWriteTime | Out-File (Join-Path $out "package\files.txt")
}
Add-Content -Path (Join-Path $out "meta.txt") -Value "finished: $((Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ'))" -Encoding utf8

function ConvertTo-Html([string]$text) { return [System.Net.WebUtility]::HtmlEncode($text) }
$html = New-Object System.Collections.Generic.List[string]
$html.Add('<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><title>Harness evidence</title>')
$html.Add('<style>body{font:14px system-ui,sans-serif;margin:2em;max-width:72em}table{border-collapse:collapse}td,th{border:1px solid #bbb;padding:.3em .6em;text-align:left;vertical-align:top}.passed,.captured{color:#176f2c}.failed,.blocked{color:#b3261e;font-weight:bold}.skipped,.manual{color:#666}img{max-width:22em;border:1px solid #ccc}</style></head><body>')
$html.Add("<h1>Harness evidence: $(ConvertTo-Html (Split-Path $out -Leaf))</h1>")
$html.Add("<h2>Machine</h2><pre>$(ConvertTo-Html ((Get-Content (Join-Path $out 'meta.txt')) -join [Environment]::NewLine))</pre>")
$html.Add('<h2>Steps</h2><table><tr><th>Step</th><th>Status</th><th>Exit</th><th>Seconds</th><th>Log</th></tr>')
foreach ($row in (Get-Content $summary | Select-Object -Skip 1)) {
    $step, $status, $code, $seconds = $row.Split("|")
    $html.Add("<tr><td>$(ConvertTo-Html $step)</td><td class=""$status"">$status</td><td>$code</td><td>$seconds</td><td><a href=""logs/$step.log"">logs/$step.log</a></td></tr>")
}
$html.Add('</table>')
$tour = Join-Path $out "screenshots\tour.tsv"
if (Test-Path $tour) {
    $rows = @(Get-Content $tour | Select-Object -Skip 1)
    $count = { param($s) @($rows | Where-Object { $_.Split("|")[3] -in $s }).Count }
    $html.Add("<h2>Screenshot tour</h2><p>$(& $count @('captured')) captured, $(& $count @('failed','blocked')) failed or blocked, $(& $count @('manual')) on the manual walk. <a href=""screenshots/tour.tsv"">tour.tsv</a>, <a href=""screenshots/manual-walk.txt"">manual-walk.txt</a></p>")
    $html.Add('<table><tr><th>#</th><th>State</th><th>Theme</th><th>Status</th><th>Capture or note</th></tr>')
    foreach ($row in $rows) {
        $number, $slug, $theme, $status, $file, $note = $row.Split("|", 6)
        $cell = ConvertTo-Html $note
        if ($file) { $cell = "<a href=""screenshots/$file""><img src=""screenshots/$file"" alt=""$slug $theme""></a><br>$cell" }
        $html.Add("<tr><td>$number</td><td>$(ConvertTo-Html $slug)</td><td>$theme</td><td class=""$status"">$status</td><td>$cell</td></tr>")
    }
    $html.Add('</table>')
}
$packageSums = Join-Path $out "package\SHA256SUMS"
if (Test-Path $packageSums) { $html.Add("<h2>Package</h2><pre>$(ConvertTo-Html (Get-Content $packageSums -Raw))</pre>") }
$html.Add('<p>Local evidence; nothing in this folder was uploaded.</p></body></html>')
Set-Content -Path (Join-Path $out "index.html") -Value $html -Encoding utf8

Write-Host ""
Write-Host "evidence: $(Join-Path $out 'index.html')"
if ($failed.Count -gt 0) {
    Write-Host "failed: $($failed -join ' ')"
    exit 1
}
Write-Host "every step passed or was skipped"
exit 0
