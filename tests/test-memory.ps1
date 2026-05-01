<#
.SYNOPSIS
    Dedicated test runner for Merix MemoryLayer (Persistent + Ethereal)
    Follows Merix V2.5 instruction set — no editing of .rs files.

.PARAMETER Filter
    "persistent", "ethereal", or "all" (default)

.PARAMETER Coverage
    Run with code coverage (requires cargo-llvm-cov installed)
#>

param(
    [ValidateSet("all", "persistent", "ethereal")]
    [string]$Filter = "all",

    [switch]$Coverage
)

Write-Host "🚀 Running Merix MemoryLayer tests..." -ForegroundColor Cyan

$testArgs = @("--package", "merix-memory")

if ($Filter -eq "all") {
    $testArgs += "--test", "integration"
} elseif ($Filter -eq "persistent") {
    $testArgs += "test_persistent_memory_basic"
} elseif ($Filter -eq "ethereal") {
    $testArgs += "test_ethereal_memory_basic"
}

if ($Coverage) {
    Write-Host "📊 Running with coverage..." -ForegroundColor Magenta
    cargo llvm-cov -- $testArgs
} else {
    cargo test @testArgs -- --nocapture
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ All MemoryLayer tests passed!" -ForegroundColor Green
} else {
    Write-Host "❌ MemoryLayer tests failed for filter: $Filter" -ForegroundColor Red
    Write-Host "   (See Cargo output above for details)" -ForegroundColor Yellow
}


