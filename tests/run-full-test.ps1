# tests\run-full-test.ps1
$env:RUST_LOG = "info,merix=debug"

Write-Host "=== Merix PHASE 1 Full Test Suite ===" -ForegroundColor Cyan

# Clean previous data
Remove-Item -Recurse -Force "data" -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "data" | Out-Null

# 1. Status
Write-Host "`n1. System Status" -ForegroundColor Yellow
cargo run --bin merix-cli -- status

# 2. Run a normal task
Write-Host "`n2. Running normal task" -ForegroundColor Yellow
cargo run --bin merix-cli -- task "Full test task - verify persistence and memory"

# 3. Tools & Skills
Write-Host "`n3. Tools and Skills" -ForegroundColor Yellow
cargo run --bin merix-cli -- tool-list
cargo run --bin merix-cli -- skill-list

# 4. Self-extension
Write-Host "`n4. Self-extension test" -ForegroundColor Yellow
cargo run --bin merix-cli -- self-extend --session-id "auto"

# 5. Resume / Crash Recovery
Write-Host "`n5. Resume / Crash Recovery test" -ForegroundColor Yellow
.\crash-recover-test.ps1

Write-Host "`n=== PHASE 1 TEST SUITE COMPLETE ===" -ForegroundColor Green