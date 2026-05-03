# test-core.ps1
# Unit + Integration tests for merix-core crate
# Run from MSVC Developer Command Prompt in the project root
# sccache-friendly: clean is OFF by default for fast iteration

param(
    [switch]$Clean          # Use -Clean only when you want a completely fresh build
)

$ErrorActionPreference = "Stop"
Write-Host "=== Merix Core Test Runner (Unit + Integration) ===" -ForegroundColor Cyan

if ($Clean) {
    Write-Host "`n1. Full clean (requested)..." -ForegroundColor Yellow
    cargo clean -p merix-core
} else {
    Write-Host "`n1. Skipping clean (sccache preserved for speed)..." -ForegroundColor DarkGray
}

# 2. Check
Write-Host "`n2. Checking merix-core..." -ForegroundColor Yellow
cargo check -p merix-core
if ($LASTEXITCODE -ne 0) { Write-Host "❌ Check failed!" -ForegroundColor Red; exit 1 }

# 3. Unit tests (single-threaded)
Write-Host "`n3. Running unit tests (single-threaded)..." -ForegroundColor Yellow
cargo test -p merix-core -- --test-threads=1 --nocapture
if ($LASTEXITCODE -ne 0) { Write-Host "❌ Unit tests failed!" -ForegroundColor Red; exit 1 }

# 4. Integration tests (single-threaded + real model)
Write-Host "`n4. Running integration tests (single-threaded)..." -ForegroundColor Yellow
cargo test -p merix-core --test integration -- --test-threads=1 --nocapture
if ($LASTEXITCODE -ne 0) { Write-Host "❌ Integration tests failed!" -ForegroundColor Red; exit 1 }

Write-Host "`n✅ All merix-core tests passed successfully!" -ForegroundColor Green
Write-Host "CoreRuntime is stable, resumable, crash-safe, and CUDA-ready." -ForegroundColor Green
Write-Host "Next step: update PLAN.md checkbox and commit." -ForegroundColor Green