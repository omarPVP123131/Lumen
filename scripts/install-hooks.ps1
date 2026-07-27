#!/usr/bin/env pwsh
# install-hooks.ps1 — Instala post-commit hook para auto-tag
# Uso: ./scripts/install-hooks.ps1

$RepoRoot = git rev-parse --show-toplevel 2>$null
if (-not $RepoRoot) {
    Write-Error "No estás en un repositorio git"
    exit 1
}

$HooksDir = Join-Path $RepoRoot ".git\hooks"
$Source = Join-Path $RepoRoot "scripts\git-hooks\post-commit"
$Dest = Join-Path $HooksDir "post-commit"

Copy-Item -Path $Source -Destination $Dest -Force
Write-Host "[install-hooks] Instalado: $Dest"
Get-ChildItem $HooksDir | Where-Object { $_.Name -notlike "*.sample" } | ForEach-Object { Write-Host "  ✓ $($_.Name)" }
