Write-Host "Running merix-cli tests..." -ForegroundColor Cyan
cargo test --package merix-cli -- --nocapture
