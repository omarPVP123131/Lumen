#!/usr/bin/env pwsh
# install-hooks.ps1 — Configura git para usar scripts/git-hooks/ como hooks
# Uso: ./scripts/install-hooks.ps1

$RepoRoot = git rev-parse --show-toplevel 2>$null
if (-not $RepoRoot) {
    Write-Error "No estás en un repositorio git"
    exit 1
}

$HooksPath = Join-Path $RepoRoot "scripts\git-hooks"
git config core.hooksPath $HooksPath

if ($LASTEXITCODE -eq 0) {
    Write-Host "[install-hooks] hooksPath → $HooksPath"
    Get-ChildItem $HooksPath | ForEach-Object { Write-Host "  ✓ $($_.Name)" }
}
