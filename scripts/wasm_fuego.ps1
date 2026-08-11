# wasm_fuego.ps1 — Prueba de fuego WASM: corre los ejemplos en el runtime WASM
# (wasm-pack) y los compara contra la VM Rust nativa (lumen.exe).
# Uso: pwsh scripts/wasm_fuego.ps1 [-SoloListar] [-Timeout seg] [-MaxEjemplos n]
param(
    [int]$Timeout = 15,
    [int]$MaxEjemplos = 0
)
$root = "C:\Users\Omar\Documents\Documentos WEb\LumenRust"
$exe = "$root\target\release\lumen.exe"
$node = "node"
$runner = "$root\scripts\wasm-ejemplos-test.mjs"
$tmpOut = "$root\stdlib\compiler\fuego_wasm_out.txt"
$tmpErr = "$root\stdlib\compiler\fuego_wasm_err.txt"

$skip = @('debug_parser3.nv', 'graficos_completo.nv', 'gui_ventana.nv', 'test_connect_direct.nv', 'tilemap_demo.nv')
$lista = @(Get-ChildItem "$root\examples\*.nv" | ForEach-Object { $_.Name } | Sort-Object | Where-Object { $_ -notin $skip })
if ($MaxEjemplos -gt 0) { $lista = $lista | Select-Object -First $MaxEjemplos }

function Ejecutar-Con-Timeout($exePath, $argsList) {
    Remove-Item $tmpOut, $tmpErr -ErrorAction SilentlyContinue
    $argStr = ($argsList | ForEach-Object {
        if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ }
    }) -join ' '
    $p = Start-Process -FilePath $exePath -ArgumentList $argStr -NoNewWindow `
        -RedirectStandardOutput $tmpOut -RedirectStandardError $tmpErr -PassThru
    if (-not $p.WaitForExit($Timeout * 1000)) {
        try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
        return "[[TIMEOUT]]"
    }
    $o = ""; if (Test-Path $tmpOut) { $o = [string](Get-Content $tmpOut -Raw -ErrorAction SilentlyContinue) }
    $e = ""; if (Test-Path $tmpErr) { $e = [string](Get-Content $tmpErr -Raw -ErrorAction SilentlyContinue) }
    return ([string]($o + $e)).Trim().Replace("`r`n", "`n")
}

$resultados = @()
$i = 0
foreach ($ej in $lista) {
    $i++
    $entrada = "$root\examples\$ej"
    $outWasm = Ejecutar-Con-Timeout $node @($runner, $entrada)
    $outRust = Ejecutar-Con-Timeout $exe @("run", $entrada)
    if ($outWasm -eq "[[TIMEOUT]]") {
        $res = "WASM-TIMEOUT"
    } elseif ($outWasm -like "W-ERROR:*" -or $outWasm -like "W-TRAP:*") {
        $res = "WASM-FALLO"
    } elseif ($outRust -eq "[[TIMEOUT]]") {
        $res = "RUST-TIMEOUT"
    } else {
        $res = if ($outWasm -eq $outRust) { "CORRECTO" } else { "DIFF" }
    }
    $detalle = ""
    if ($res -eq "DIFF") {
        $detalle = "wasm: [$($outWasm)] rust: [$($outRust)]"
    } elseif ($res -eq "WASM-FALLO") {
        $detalle = $outWasm
    }
    $resultados += [PSCustomObject]@{ Ejemplo = $ej; Resultado = $res; Detalle = $detalle }
    Write-Output ("{0,3}. {1,-32} {2}" -f $i, $ej, $res)
}

Write-Output ""
Write-Output "=== RESUMEN WASM ($($resultados.Count) ejemplos) ==="
$ok = ($resultados | Where-Object { $_.Resultado -eq "CORRECTO" }).Count
$diff = ($resultados | Where-Object { $_.Resultado -eq "DIFF" }).Count
$fallo = ($resultados | Where-Object { $_.Resultado -eq "WASM-FALLO" }).Count
$to = ($resultados | Where-Object { $_.Resultado -like "*TIMEOUT*" }).Count
Write-Output "Correctos: $ok | DIFF: $diff | WASM-FALLO: $fallo | Timeouts: $to"
Write-Output ""
Write-Output "--- Detalle ---"
$resultados | Where-Object { $_.Resultado -ne "CORRECTO" } | Format-Table -AutoSize | Out-String -Width 400