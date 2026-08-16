# ============================================================================
# LÚMEN — Ejecutor y Verificador Masivo de Ejemplos en Windows PowerShell
# Valida y prueba todos los ejemplos (.nv) en examples/ en un solo comando
# Uso: 
#   .\scripts\probar_todos_ejemplos.ps1         # Comprobación de tipos (check)
#   .\scripts\probar_todos_ejemplos.ps1 check   # Comprobación estática completa
#   .\scripts\probar_todos_ejemplos.ps1 run     # Ejecución en máquina virtual
# ============================================================================

param(
    [string]$Accion = "check"
)

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║        🚀 LÚMEN — VERIFICACIÓN MASIVA DE EJEMPLOS (.nv)              ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Determinar el binario de LÚMEN (release o debug)
$LumenBin = "target\release\lumen.exe"
if (-not (Test-Path $LumenBin)) {
    $LumenBin = "target\debug\lumen.exe"
}
if (-not (Test-Path $LumenBin)) {
    Write-Host "Compilando binario de LÚMEN primero..." -ForegroundColor Yellow
    cargo build --bin lumen
    $LumenBin = "target\debug\lumen.exe"
}

Write-Host "• Binario activo: $LumenBin" -ForegroundColor Gray
Write-Host "• Modo de prueba: $Accion" -ForegroundColor Yellow

# Si la acción es check global directo
if ($Accion -eq "check" -or $Accion -eq "comprobar") {
    Write-Host "• Ejecutando validación estática global recursiva..." -ForegroundColor White
    $StartTime = Get-Date
    & $LumenBin check -L stdlib -L stdlib/compiler examples
    $exitCode = $LASTEXITCODE
    $Elapsed = [math]::Round(((Get-Date) - $StartTime).TotalSeconds, 2)
    Write-Host "• Tiempo de comprobación: $Elapsed segundos" -ForegroundColor Gray
    exit $exitCode
}

# Si la acción es run individual
$Examples = Get-ChildItem -Path "examples" -Filter "*.nv" -Recurse | Sort-Object Name
$Total = $Examples.Count
$Passed = 0
$Failed = 0
$StartTime = Get-Date

Write-Host "• Total de ejemplos detectados: $Total" -ForegroundColor White
Write-Host "• Iniciando ejecución masiva..." -ForegroundColor White
Write-Host "══════════════════════════════════════════════════════════════════════" -ForegroundColor DarkGray

foreach ($file in $Examples) {
    $relPath = $file.FullName.Replace((Get-Location).Path + "\", "")
    
    $output = & $LumenBin run -L stdlib -L stdlib/compiler $file.FullName 2>&1
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        $Passed++
        Write-Host "  ✓ OK: $($file.Name)" -ForegroundColor Green
    } else {
        $Failed++
        Write-Host "  ✗ FALLÓ: $($file.Name)" -ForegroundColor Red
        $output | Select-Object -First 3 | ForEach-Object { Write-Host "      $_" -ForegroundColor DarkRed }
    }
}

$Elapsed = [math]::Round(((Get-Date) - $StartTime).TotalSeconds, 2)

Write-Host "══════════════════════════════════════════════════════════════════════" -ForegroundColor DarkGray
Write-Host ""
Write-Host "📊 RESUMEN FINAL DE EJECUCIÓN MASIVA:" -ForegroundColor Cyan
Write-Host "  • Total analizados : $Total" -ForegroundColor White
Write-Host "  • Pasaron con éxito: $Passed" -ForegroundColor Green
Write-Host "  • Fallaron         : $Failed" -ForegroundColor $(if ($Failed -eq 0) { "Green" } else { "Red" })
Write-Host "  • Tiempo total     : $Elapsed segundos" -ForegroundColor Gray
Write-Host ""

if ($Failed -eq 0) {
    Write-Host "🎉 ¡TODOS LOS EJEMPLOS EJECUTADOS Y VALIDADOS AL 100% CON ÉXITO!" -ForegroundColor Green
} else {
    Write-Host "⚠️  Se encontraron algunos errores en los ejemplos indicados arriba." -ForegroundColor Yellow
}
Write-Host ""
