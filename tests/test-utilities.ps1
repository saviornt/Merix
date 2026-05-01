Write-Host "Running merix-utilities tests..." -ForegroundColor Cyan
cargo test --package merix-utilities -- --nocapture
