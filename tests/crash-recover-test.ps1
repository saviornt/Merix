# tests\test-resume.ps1
$env:RUST_LOG = "info,merix=debug"

<<<<<<< HEAD
Write-Host "=== Merix Resume / Crash Recovery Test ===" -ForegroundColor Cyan
=======
Write-Host "=== Merix PHASE 1 - Real Resume / Crash Recovery Test ===" -ForegroundColor Cyan
>>>>>>> 2f6ba3de6402cb75d225655dbbeb6b50574685da

# Start a task in the background
Write-Host "Starting long-running task in background..." -ForegroundColor Yellow
$taskProcess = Start-Process -FilePath "target\debug\merix-cli.exe" `
                             -ArgumentList 'task "Create a resumable task that will be interrupted"' `
                             -PassThru -NoNewWindow

# Give it time to create checkpoints (adjust if needed)
<<<<<<< HEAD
Start-Sleep -Seconds 5
=======
Start-Sleep -Milliseconds 800
>>>>>>> 2f6ba3de6402cb75d225655dbbeb6b50574685da

# Simulate crash by killing the process
Write-Host "Simulating crash (killing task process)..." -ForegroundColor Red
Stop-Process -Id $taskProcess.Id -Force -ErrorAction SilentlyContinue

# Now resume
Write-Host "Running resume command..." -ForegroundColor Green
cargo run --bin merix-cli -- resume

Write-Host "=== Resume Test Complete ===" -ForegroundColor Cyan