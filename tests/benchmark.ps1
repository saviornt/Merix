# Merix Benchmark Test — CLI startup + common commands (Pillar 3)

Write-Host "=== Merix PHASE 1 Benchmark ===" -ForegroundColor Cyan

$binary = "target\debug\merix.exe"

if (-not (Test-Path $binary)) {
    Write-Host "Building debug binary first..." -ForegroundColor Yellow
    cargo build --quiet
}

function Measure-MerixCommand {
    param([string]$Description, [string]$Arguments)

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    Start-Process -FilePath $binary -ArgumentList $Arguments -NoNewWindow -PassThru -Wait | Out-Null
    $timer.Stop()

    $timeMs = $timer.Elapsed.TotalMilliseconds
    Write-Host "✓ $Description completed in $($timeMs.ToString("F2")) ms" -ForegroundColor Green
    return $timeMs
}

Write-Host "`nBenchmarking Merix CLI..." -ForegroundColor Yellow

$times = @{}

$times["Help"]        = Measure-MerixCommand "merix --help" "--help"
$times["Version"]     = Measure-MerixCommand "merix future version" "future version"
$times["Debug Log"]   = Measure-MerixCommand "merix --log-level debug" "--log-level debug"

$totalTime = ($times.Values | Measure-Object -Sum).Sum

Write-Host "`n=== Benchmark Summary ===" -ForegroundColor Cyan
Write-Host "Total time      : $($totalTime.ToString("F2")) ms" -ForegroundColor White
Write-Host "Typical Phase 1 range: 200-4000 ms (depends on machine)" -ForegroundColor Gray
Write-Host "Note: Peak memory measurement for fast CLI is unreliable in PowerShell." -ForegroundColor DarkGray
Write-Host "      We can add advanced profiling later when we have longer tasks." -ForegroundColor DarkGray

Write-Host "`n✅ Benchmark completed successfully" -ForegroundColor Green
exit 0
