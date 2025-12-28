#!/usr/bin/env pwsh
# Fast Python venv setup - only creates if missing

$ErrorActionPreference = "Stop"
$venvPath = "$PSScriptRoot\..\src-tauri\amplifier-tools\.venv"
$installMarker = "$venvPath\.installed"

Push-Location "$PSScriptRoot\..\src-tauri\amplifier-tools"

$needInstall = $false

if (-not (Test-Path $venvPath)) {
    Write-Host "Creating Python venv..." -ForegroundColor Yellow
    python -m venv .venv
    $needInstall = $true
} else {
    Write-Host "✓ Python venv exists" -ForegroundColor Green
}

# Check if dependencies need installing
if ((-not (Test-Path $installMarker)) -or $needInstall) {
    Write-Host "Installing dependencies..." -ForegroundColor Yellow
    & .venv\Scripts\Activate.ps1
    pip install -e . --quiet 2>&1 | Out-Null
    New-Item -ItemType File -Path $installMarker -Force | Out-Null
    Write-Host "✓ Dependencies installed" -ForegroundColor Green
} else {
    Write-Host "✓ Dependencies already installed" -ForegroundColor Green
}

Pop-Location
Write-Host "✓ Python setup complete" -ForegroundColor Green
