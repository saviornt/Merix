# tests/test-cargo.ps1 — Merix development loop (CLI → Logging → Testing)
Write-Host "=== Merix Cargo Test Loop ===" -ForegroundColor Cyan

cargo fmt --all
if ($LASTEXITCODE -ne 0) { exit 1 }

# CPU-only (fast dev loop)
cargo check --workspace
if ($LASTEXITCODE -ne 0) { exit 1 }

cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "`n✅ All clear — workspace is clean (CPU-only)!" -ForegroundColor Green