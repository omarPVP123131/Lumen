# fuego.ps1 — Prueba de fuego COMPLETA: compila los 115 ejemplos con el pipeline puro
# Nivel 1: compilación (compiler_v4.nvc)   Nivel 2: ejecución del .nvc + comparación vs Rust
# Uso: ./fuego.ps1 [-ArchivoLista archivo] [-SoloCompilar] [-Timeout seg]
param(
    [string]$ArchivoLista = "",
    [switch]$SoloCompilar,
    [int]$Timeout = 10
)
$root = "C:\Users\Omar\Documents\Documentos WEb\LumenRust"
$v4 = "$root\stdlib\compiler\compiler_v4.nvc"
$targetFile = "$root\stdlib\compiler\target.txt"
$salida = "$root\stdlib\compiler\fuego_out.nvc"
$tmpOut = "$root\stdlib\compiler\fuego_tmp_out.txt"
$tmpErr = "$root\stdlib\compiler\fuego_tmp_err.txt"

if ($ArchivoLista -ne "") {
    $lista = Get-Content $ArchivoLista | Where-Object { $_ -ne "" -and $_ -notlike "#*" }
} else {
    $lista = @(Get-ChildItem "$root\examples\*.nv" | ForEach-Object { $_.Name } | Sort-Object)
}

function Ejecutar-Con-Timeout($exe, $arg) {
    Remove-Item $tmpOut, $tmpErr -ErrorAction SilentlyContinue
    $p = Start-Process -FilePath $exe -ArgumentList @("run", ('"' + $arg + '"')) -NoNewWindow -RedirectStandardOutput $tmpOut -RedirectStandardError $tmpErr -PassThru
    if (-not $p.WaitForExit($Timeout * 1000)) {
        try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
        return "[[TIMEOUT]]"
    }
    $o = ""
    if (Test-Path $tmpOut) { $o = Get-Content $tmpOut -Raw }
    $e = ""
    if (Test-Path $tmpErr) { $e = Get-Content $tmpErr -Raw }
    return $o + $e
}

$resultados = @()
$i = 0
foreach ($ej in $lista) {
    $i++
    $entrada = "examples/$ej"
    Set-Content -Path $targetFile -Value "$entrada`n$salida" -Encoding utf8 -NoNewline
    $out = & $root\target\release\lumen.exe run $v4 2>&1 | Out-String
    $lineas = $out -split "`r?`n" | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" }
    if (($lineas | Where-Object { $_ -like "OK*" }).Count -gt 0) {
        $res = "OK"
    } elseif (($lineas | Where-Object { $_ -like "FALLO*" }).Count -gt 0) {
        $res = "FALLO"
    } else {
        $res = "?ERROR?"
    }
    $detalle = ""
    if ($res -eq "OK" -and -not $SoloCompilar) {
        $outNvc = Ejecutar-Con-Timeout $root\target\release\lumen.exe $salida
        if ($outNvc -eq "[[TIMEOUT]]") {
            $res = "OK+TIMEOUT"
            $detalle = "ejecución excede $Timeout s (¿loop infinito o GUI?)"
        } else {
            $outRust = Ejecutar-Con-Timeout $root\target\release\lumen.exe $entrada
            if ($outNvc -eq $outRust) {
                $res = "OK+CORRECTO"
            } elseif ($outRust -eq "[[TIMEOUT]]") {
                $res = "OK+TIMEOUT-RUST"
                $detalle = "rust también excede $Timeout s"
            } else {
                $res = "OK+INCOMPATIBLE"
                $detalle = "nvc: [$($outNvc.Trim())] rust: [$($outRust.Trim())]"
            }
        }
    } else {
        $detalle = ($lineas | Select-Object -Last 3) -join " | "
    }
    $resultados += [PSCustomObject]@{ Ejemplo = $ej; Resultado = $res; Detalle = $detalle }
    Write-Output ("{0,3}. {1,-28} {2}" -f $i, $ej, $res)
}

Write-Output ""
Write-Output "=== RESUMEN ($($resultados.Count) ejemplos) ==="
$ok = ($resultados | Where-Object { $_.Resultado -like "OK*" }).Count
$correcto = ($resultados | Where-Object { $_.Resultado -eq "OK+CORRECTO" }).Count
$incomp = ($resultados | Where-Object { $_.Resultado -eq "OK+INCOMPATIBLE" }).Count
$timeout = ($resultados | Where-Object { $_.Resultado -like "*TIMEOUT*" }).Count
$fallo = ($resultados | Where-Object { $_.Resultado -notlike "OK*" }).Count
Write-Output "Compilan: $ok/$($resultados.Count) | Correctos: $correcto | Incompatibles: $incomp | Timeouts: $timeout | Fallos: $fallo"
Write-Output ""
Write-Output "--- Detalle de no-correctos ---"
$resultados | Where-Object { $_.Resultado -ne "OK+CORRECTO" } | Format-Table -AutoSize | Out-String -Width 250
