# tests\test-resume.ps1
$env:RUST_LOG = "info,merix=debug"

Write-Host "=== Merix Resume / Crash Recovery Test ===" -ForegroundColor Cyan

# Start a task in the background
Write-Host "Starting long-running task in background..." -ForegroundColor Yellow
$taskProcess = Start-Process -FilePath "target\debug\merix-cli.exe" `
                             -ArgumentList 'task "Create a resumable task that will be interrupted"' `
                             -PassThru -NoNewWindow

# Give it time to create checkpoints (adjust if needed)
Start-Sleep -Seconds 5

# Simulate crash by killing the process
Write-Host "Simulating crash (killing task process)..." -ForegroundColor Red
Stop-Process -Id $taskProcess.Id -Force -ErrorAction SilentlyContinue

# Now resume
Write-Host "Running resume command..." -ForegroundColor Green
cargo run --bin merix-cli -- resume

Write-Host "=== Resume Test Complete ===" -ForegroundColor Cyan