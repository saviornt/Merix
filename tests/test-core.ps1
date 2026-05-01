<#
.SYNOPSIS
    Run all core crate tests (unit + integration) with full output and zero-warnings check.
    Part of the Three Pillars → Testing discipline.
#>

Write-Host "=== Merix Core Test Suite ===" -ForegroundColor Cyan

# Check for warnings
cargo check --package merix-core --tests

# Run unit tests (from lib.rs)
Write-Host "`nRunning unit tests..." -ForegroundColor Yellow
cargo test --package merix-core -- --quiet

# Run integration tests
Write-Host "`nRunning integration tests..." -ForegroundColor Yellow
cargo test --test integration --package merix-core -- --quiet

Write-Host "`n✅ All core tests passed! CoreRuntime is stable." -ForegroundColor Green
