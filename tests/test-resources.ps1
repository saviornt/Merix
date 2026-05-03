# tests/test-resources.ps1
# Integration + unit test runner for the resources crate (hardware detection + CUDA awareness)

Write-Host "=== merix-resources: Running unit tests ===" -ForegroundColor Cyan
cargo test --package merix-resources -- --quiet

Write-Host "`n=== merix-resources: Running integration tests ===" -ForegroundColor Yellow
cargo test --test integration --package merix-resources -- --quiet

Write-Host "`n✅ merix-resources test suite completed successfully`n" -ForegroundColor Green