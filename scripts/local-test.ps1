# Local testing script for Pulse
# Cleans up existing data and starts server + clients

param(
    [switch]$ServerOnly,
    [switch]$SkipCleanup
)

$ErrorActionPreference = "Stop"

Write-Host "=== Pulse Local Testing ===" -ForegroundColor Cyan

# Cleanup
if (-not $SkipCleanup) {
    Write-Host "`nCleaning up existing data..." -ForegroundColor Yellow

    # Remove app data
    $appData = "$env:APPDATA\pulse"
    if (Test-Path $appData) {
        Remove-Item -Recurse -Force $appData
        Write-Host "  Removed: $appData" -ForegroundColor Gray
    }

    # Remove test data directories
    $testData2 = ".\test-data-2"
    if (Test-Path $testData2) {
        Remove-Item -Recurse -Force $testData2
        Write-Host "  Removed: $testData2" -ForegroundColor Gray
    }

    Write-Host "Cleanup complete." -ForegroundColor Green
}

# Start server
Write-Host "`nStarting local server on port 9001..." -ForegroundColor Yellow
$serverPath = Join-Path $PSScriptRoot "..\pulse-server"

Start-Process -FilePath "cmd" -ArgumentList "/k", "cd /d `"$serverPath`" && cargo run" -WindowStyle Normal
Write-Host "Server started in new window." -ForegroundColor Green

# Wait for server to be ready
Write-Host "Waiting for server to start..." -ForegroundColor Gray
Start-Sleep -Seconds 3

if ($ServerOnly) {
    Write-Host "`nServer-only mode. Run clients manually:" -ForegroundColor Cyan
    Write-Host "  Client 1: cd src-tauri && cargo tauri dev" -ForegroundColor White
    Write-Host "  Client 2: cd src-tauri && cargo tauri dev -- -- --data-dir ./test-data-2" -ForegroundColor White
    exit 0
}

# Start first client
Write-Host "`nStarting Client 1..." -ForegroundColor Yellow
$tauriPath = Join-Path $PSScriptRoot "..\src-tauri"
Start-Process -FilePath "cmd" -ArgumentList "/k", "cd /d `"$tauriPath`" && cargo tauri dev" -WindowStyle Normal
Write-Host "Client 1 started." -ForegroundColor Green

# Wait a bit before starting second client
Start-Sleep -Seconds 5

# Start second client with separate data dir
Write-Host "`nStarting Client 2 (separate data directory)..." -ForegroundColor Yellow
Start-Process -FilePath "cmd" -ArgumentList "/k", "cd /d `"$tauriPath`" && cargo tauri dev -- -- --data-dir ./test-data-2" -WindowStyle Normal
Write-Host "Client 2 started." -ForegroundColor Green

Write-Host "`n=== All components started ===" -ForegroundColor Cyan
Write-Host "- Server: ws://localhost:9001" -ForegroundColor White
Write-Host "- Client 1: Default data directory" -ForegroundColor White
Write-Host "- Client 2: ./test-data-2" -ForegroundColor White
Write-Host "`nTo test offline delivery:" -ForegroundColor Yellow
Write-Host "  1. Register different users in each client" -ForegroundColor Gray
Write-Host "  2. Close Client 2" -ForegroundColor Gray
Write-Host "  3. Send messages from Client 1 to Client 2's user" -ForegroundColor Gray
Write-Host "  4. Reopen Client 2 and verify messages arrive" -ForegroundColor Gray
