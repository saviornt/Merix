# Merix Test Runner — Auto-discovers and runs ALL other *.ps1 files in tests/
# Dynamically excludes itself (works even if renamed to run-all-tests.ps1, etc.)
# Fail-fast on first error for easy debugging

$testDir = $PSScriptRoot
if (-not $testDir) { $testDir = Get-Location }

# Get the full path of the currently running script so we never run ourselves
$selfPath = $MyInvocation.MyCommand.Path

Write-Host "🚀 Merix Test Runner Starting..." -ForegroundColor Cyan
Write-Host "Scanning $testDir for test scripts..." -ForegroundColor Yellow

$testScripts = Get-ChildItem -Path $testDir -Filter "*.ps1" |
    Where-Object { $_.FullName -ne $selfPath } |
    Sort-Object Name

if ($testScripts.Count -eq 0) {
    Write-Host "No other test scripts found!" -ForegroundColor Red
    exit 1
}

Write-Host "Found $($testScripts.Count) test script(s) to run:`n" -ForegroundColor Green
$testScripts | ForEach-Object { Write-Host "   • $($_.Name)" -ForegroundColor Gray }

foreach ($script in $testScripts) {
    Write-Host "`n──────────────────────────────────────────────────────────────" -ForegroundColor DarkGray
    Write-Host "Running: $($script.Name)" -ForegroundColor Yellow

    & $script.FullName

    if ($LASTEXITCODE -ne 0) {
        Write-Host "`n❌ Test failed: $($script.Name) (exit code $LASTEXITCODE)" -ForegroundColor Red
        Write-Host "Stopping test run for debugging." -ForegroundColor Red
        exit $LASTEXITCODE
    }

    Write-Host "✅ $($script.Name) passed" -ForegroundColor Green
}

Write-Host "`n══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "✅ ALL TESTS PASSED SUCCESSFULLY!" -ForegroundColor Green
Write-Host "══════════════════════════════════════════════════════════════" -ForegroundColor Green
