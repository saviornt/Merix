# tests/test-llama.ps1
# Integration + unit test runner for the llama crate (llama-cpp-2 runtime)

Write-Host "=== merix-llama: Running unit tests ===" -ForegroundColor Cyan
cargo test --package merix-llama -- --quiet

Write-Host "`n=== merix-llama: Running integration tests ===" -ForegroundColor Yellow
cargo test --test integration --package merix-llama -- --quiet

Write-Host "`n✅ merix-llama test suite completed successfully`n" -ForegroundColor Green