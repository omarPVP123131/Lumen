#!/usr/bin/env pwsh
# autotag.ps1 — Bump de versión, tag y notas de release.
# Uso: scripts/autotag.ps1 [patch|minor|major|2.4.1] [--no-push]
#
# Flujo:
#   1. Lee la versión actual de VERSION (fuente única de verdad).
#   2. Si el argumento es patch/minor/major, incrementa semver.
#   3. Si el argumento es un semver completo (ej: 2.4.1), lo usa tal cual.
#   4. Verifica que el tag v<ver> no exista ya.
#   5. Verifica que git esté limpio y en master/main.
#   6. Crea el commit de versionado, el tag anotado y (por defecto) lo empuja.
#
# Los checks de calidad (fmt/clippy/test) corren en CI: el tag solo se
# publica automáticamente cuando TODO pasa (ver .github/workflows/ci.yml).
# Este script SOLO prepara el versionado. --no-push deja el commit+tag
# local para que CI los valide antes del push manual.

param(
    [string]$Bump = "patch",
    [switch]$NoPush
)

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

# ── 1. Versión actual ──────────────────────────────
$versionFile = Join-Path (Get-Location) "VERSION"
if (-not (Test-Path $versionFile)) {
    Write-Host "[autotag] ERROR: no existe VERSION. Créalo con '2.4.1'." -ForegroundColor Red
    exit 1
}
$current = (Get-Content $versionFile | Select-Object -First 1).Trim()

# ── 2. Calcular nueva versión ──────────────────────
$new = $current
if ($Bump -match '^\d+\.\d+\.\d+$') {
    $new = $Bump
} else {
    $parts = $current.Split(".")
    $maj = [int]$parts[0]; $min = [int]$parts[1]; $pat = [int]$parts[2]
    switch ($Bump.ToLower()) {
        "major" { $maj++; $min = 0; $pat = 0 }
        "minor" { $min++; $pat = 0 }
        "major-minor" { $min++ }   # ej: 2.4 → 2.5 (sin tocar patch)
        default { $pat++ }        # patch
    }
    $new = "$maj.$min.$pat"
}
if ($new -eq $current) {
    Write-Host "[autotag] La versión ya es $current. Usa patch/minor/major o un semver." -ForegroundColor Yellow
    exit 1
}

$tag = "v$new"

# ── 3. Pre-condiciones ─────────────────────────────
$gitStatus = git status --porcelain
if ($gitStatus) {
    Write-Host "[autotag] ERROR: working tree sucio. Commit o stash antes de versionar." -ForegroundColor Red
    git status --short
    exit 1
}
$branch = git branch --show-current
if ($branch -notin @("master", "main")) {
    Write-Host "[autotag] ERROR: debes estar en master/main (estás en $branch)." -ForegroundColor Red
    exit 1
}
if (git rev-parse $tag 2>$null) {
    Write-Host "[autotag] ERROR: el tag $tag ya existe." -ForegroundColor Red
    exit 1
}

# ── 4. Escribir nueva versión ──────────────────────
Set-Content -Path $versionFile -Value $new -NoNewline
git add VERSION
git commit -m "chore(version): bump $current -> $new"

# ── 5. Tag anotado ─────────────────────────────────
git tag -a $tag -m "Release $tag — LÚMEN $new

Autogenerado por scripts/autotag.ps1. Los checks de CI (fmt, clippy
-D warnings, tests 3 OS, builds) deben pasar ANTES de publicar el tag:
el job 'autotag' de .github/workflows/ci.yml lo crea automáticamente
cuando el push a master pasa todas las pruebas."

if ($NoPush) {
    Write-Host "[autotag] Commit + tag $tag locales. CI validará y publicará al hacer push." -ForegroundColor Green
} else {
    git push origin $branch
    git push origin $tag
    Write-Host "[autotag] v$current -> v$new pusheado con tag $tag." -ForegroundColor Green
}
Write-Host "[autotag] VERSION ahora contiene: $new" -ForegroundColor Green