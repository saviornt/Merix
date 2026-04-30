# tests\benchmark.ps1
$env:RUST_LOG = "info"

Write-Host "=== Merix PHASE 1 Benchmark ===" -ForegroundColor Cyan

# Run task and measure time + peak memory
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$process = Start-Process -FilePath "target\debug\merix-cli.exe" `
                         -ArgumentList 'task "Benchmark task"' `
                         -PassThru -NoNewWindow

$process.WaitForExit()
$sw.Stop()

Write-Host "Task completed in $($sw.Elapsed.TotalSeconds) seconds" -ForegroundColor Green
Write-Host "Peak memory usage (approx): $($process.PeakWorkingSet64 / 1MB) MB" -ForegroundColor Green