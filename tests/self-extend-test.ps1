# tests\self-extend-test.ps1
$env:RUST_LOG = "info,merix=debug"

Write-Host "Running self-extension test..." -ForegroundColor Cyan
cargo run --bin merix-cli -- self-extend --session-id "auto"