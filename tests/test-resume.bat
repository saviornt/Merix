# test-resume.ps1
$env:RUST_LOG="info"
cargo run --bin merix-cli -- task "Create a resumable task that will be interrupted - Ctrl+C to stop"
# Simulate crash by killing (manual for now)
cargo run --bin merix-cli -- resume