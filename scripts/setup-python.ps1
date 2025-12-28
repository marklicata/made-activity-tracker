#!/usr/bin/env pwsh
# Fast Python venv setup - only creates if missing

$ErrorActionPreference = "Stop"
$venvPath = "$PSScriptRoot\..\src-tauri\amplifier-tools\.venv"

Push-Location "$PSScriptRoot\..\src-tauri\amplifier-tools"

if (-not (Test-Path $venvPath)) {
    Write-Host "Creating Python venv..." -ForegroundColor Yellow
    uv venv | Out-Null
} else {
    Write-Host "✓ Python venv exists" -ForegroundColor Green
}

Write-Host "Installing dependencies..." -ForegroundColor Yellow
uv pip install -e . --quiet 2>&1 | Out-Null

Pop-Location
Write-Host "✓ Python setup complete" -ForegroundColor Green
