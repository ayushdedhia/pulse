# Dev client launcher with optional fresh database
# Usage:
#   .\scripts\dev-client.ps1              # Normal launch
#   .\scripts\dev-client.ps1 -Fresh       # Clear DB first
#   .\scripts\dev-client.ps1 -Instance 2  # Second instance
#   .\scripts\dev-client.ps1 -Fresh -Instance 2

param(
    [switch]$Fresh,
    [int]$Instance = 1
)

$ErrorActionPreference = "Stop"

# Determine data directory
if ($Instance -eq 1) {
    $dataDir = "$env:APPDATA\pulse"
    $dataDirArg = ""
} else {
    $dataDir = ".\test-data-$Instance"
    $dataDirArg = "-- -- --data-dir $dataDir"
}

Write-Host "=== Pulse Dev Client (Instance $Instance) ===" -ForegroundColor Cyan

# Fresh start - delete database
if ($Fresh) {
    Write-Host "Clearing database at: $dataDir" -ForegroundColor Yellow
    if (Test-Path $dataDir) {
        Remove-Item -Recurse -Force $dataDir
        Write-Host "  Deleted." -ForegroundColor Green
    } else {
        Write-Host "  Already clean." -ForegroundColor Gray
    }
}

# Launch client
Write-Host "Starting client..." -ForegroundColor Yellow
Set-Location (Join-Path $PSScriptRoot "..")

if ($Instance -eq 1) {
    cargo tauri dev
} else {
    cargo tauri dev -- -- --data-dir $dataDir
}
