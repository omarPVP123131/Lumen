# ═══════════════════════════════════════════════════════════════
# Build LÚMEN WASM for WASI target (wasm32-wasip1) — PowerShell
# ═══════════════════════════════════════════════════════════════
param(
    [switch]$Release = $true,
    [switch]$Check = $false
)

$ErrorActionPreference = "Stop"
Push-Location -LiteralPath "$PSScriptRoot\..\.."

Write-Host "╔════════════════════════════════════╗"
Write-Host "║   LÚMEN WASI Build (Windows)      ║"
Write-Host "╚════════════════════════════════════╝"
Write-Host ""
Write-Host "📦 Project: $PWD"
Write-Host "🎯 Target:  wasm32-wasip1"
Write-Host "📂 Crate:   lumen-wasm"
Write-Host ""

# ── Step 1: Check for WASI target ──────────────────────────
Write-Host "1️⃣  Verificando target wasm32-wasip1..."
$installed = rustup target list --installed | Select-String "wasm32-wasip1"
if (-not $installed) {
    Write-Host "   ⚠️  Target no instalado. Instalando..."
    rustup target add wasm32-wasip1
} else {
    Write-Host "   ✅ Target wasm32-wasip1 instalado"
}

# ── Step 2: Build ─────────────────────────────────────────
Write-Host ""
Write-Host "2️⃣  Compilando lumen-wasm para WASI..."

if ($Release) {
    $profileFlag = "--release"
} else {
    $profileFlag = ""
}

cargo build -p lumen-wasm `
    --target wasm32-wasip1 `
    --no-default-features `
    --features wasi `
    $profileFlag

if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ Error de compilación"
    Pop-Location
    exit 1
}

# ── Step 3: Check artifact ────────────────────────────────
Write-Host ""
Write-Host "3️⃣  Verificando artefacto..."

$profileDir = if ($Release) { "release" } else { "debug" }
$wasmFile = "target\wasm32-wasip1\$profileDir\lumen_wasm.wasm"

if (Test-Path -LiteralPath $wasmFile) {
    $size = (Get-Item -LiteralPath $wasmFile).Length
    $sizeKB = [math]::Round($size / 1024, 1)
    Write-Host "   ✅ WASM generado: $wasmFile"
    Write-Host "   📏 Tamaño: ${sizeKB} KB ($size bytes)"
} else {
    Write-Host "   ❌ ERROR: No se encontró: $wasmFile"
    Pop-Location
    exit 1
}

# ── Step 4: cargo check ───────────────────────────────────
Write-Host ""
Write-Host "4️⃣  Verificando con cargo check..."
cargo check -p lumen-wasm `
    --target wasm32-wasip1 `
    --no-default-features `
    --features wasi

Write-Host ""
Write-Host "╔════════════════════════════════════╗"
Write-Host "║   WASI Build Completado ✅          ║"
Write-Host "╚════════════════════════════════════╝"
Write-Host ""
Write-Host "Para ejecutar con un runtime WASI:"
Write-Host "  wasmtime target\wasm32-wasip1\release\lumen_wasm.wasm"
Write-Host "  wasmer run target\wasm32-wasip1\release\lumen_wasm.wasm"

Pop-Location
