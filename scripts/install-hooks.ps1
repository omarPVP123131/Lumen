# ============================================================================
# LÚMEN — Instalador de Git Hooks para Windows PowerShell (install-hooks.ps1)
# ============================================================================

$RepoRoot = git rev-parse --show-toplevel 2>$null
if (-not $RepoRoot) {
    Write-Error "No estás en un repositorio Git"
    exit 1
}

$HooksDir = Join-Path $RepoRoot ".git\hooks"
$SourceDir = Join-Path $RepoRoot "scripts\git-hooks"

if (-not (Test-Path $HooksDir)) {
    New-Item -ItemType Directory -Path $HooksDir -Force | Out-Null
}

Get-ChildItem $SourceDir | ForEach-Object {
    $Dest = Join-Path $HooksDir $_.Name
    Copy-Item -Path $_.FullName -Destination $Dest -Force
    Write-Host "  ✓ Hook instalado: $($_.Name)" -ForegroundColor Green
}

Write-Host "🎉 ¡Git hooks de LÚMEN (pre-commit, pre-push, post-commit) instalados con éxito!" -ForegroundColor Cyan
