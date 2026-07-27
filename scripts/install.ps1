#!/usr/bin/env pwsh
# install.ps1 — Instalador de LÚMEN para Windows
# Uso: irm https://raw.githubusercontent.com/omarPVP123131/Lumen/main/scripts/install.ps1 | pwsh
# O:   ./scripts/install.ps1

$Version = "1.6.0"
$Repo = "omarPVP123131/Lumen"
$BinDir = "$env:LOCALAPPDATA\lumen\bin"

# Detectar arquitectura
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$Target = "x86_64-pc-windows-msvc"

Write-Host "LÚMEN v$Version Installer (Windows)" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# Crear directorio bin
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    Write-Host "  ✓ Creado: $BinDir"
}

# Descargar release
$Url = "https://github.com/$Repo/releases/download/v$Version/lumen-$Target.zip"
$ZipPath = "$env:TEMP\lumen-$Version.zip"

Write-Host "  Descargando $Url ..."
try {
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
    Write-Host "  ✓ Descargado"
}
catch {
    Write-Host "  ! No se encontró release para v$Version" -ForegroundColor Yellow
    Write-Host "  Construyendo desde fuente..."
    
    # Verificar Rust
    if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
        Write-Host "  ✗ Necesitas Rust: https://rustup.rs" -ForegroundColor Red
        exit 1
    }
    
    # Clonar y compilar
    $SrcDir = "$env:TEMP\lumen-build"
    if (Test-Path $SrcDir) { Remove-Item -Recurse -Force $SrcDir }
    
    git clone "https://github.com/$Repo.git" --branch "v$Version" $SrcDir 2>$null
    if (-not $?) {
        git clone "https://github.com/$Repo.git" $SrcDir
    }
    
    Push-Location $SrcDir
    cargo build --release
    if (-not $?) { Pop-Location; exit 1 }
    
    Copy-Item "target\release\lumen.exe" "$BinDir\lumen.exe"
    Pop-Location
    Remove-Item -Recurse -Force $SrcDir
    
    Write-Host "  ✓ Compilado desde fuente"
}

# Agregar al PATH si no está
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    $NewPath = "$BinDir;$UserPath"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = "$BinDir;$env:Path"
    Write-Host "  ✓ Agregado al PATH de usuario"
}

Write-Host ""
Write-Host "  LÚMEN instalado en: $BinDir\lumen.exe" -ForegroundColor Green
Write-Host "  Prueba: lumen run ejemplos/hello.nv" -ForegroundColor Green
