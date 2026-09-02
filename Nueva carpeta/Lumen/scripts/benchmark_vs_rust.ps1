# benchmark_vs_rust.ps1 — Benchmarks: pipeline RUST (compilador + VM) vs LÚMEN self-hosted
#  - Compilar:  `lumen build` (Rust)  vs  compiler_v4.nvc (100% LÚMEN)
#  - Ejecutar:  VM Rust                vs  vm.nvc (VM LÚMEN interpretando el mismo .nvc)
# Uso: ./scripts/benchmark_vs_rust.ps1  (pwsh 7)
$r = "C:\Users\Omar\Documents\Documentos WEb\LumenRust"
$lumen = "$r\target\release\lumen.exe"
$v4 = "$r\stdlib\compiler\compiler_v4.nvc"
$vm = "$r\stdlib\compiler\vm.nvc"
$target = "$r\stdlib\compiler\target.txt"
$entrada = "$r\stdlib\compiler\entrada_vm.txt"
$work = "$r\stdlib\compiler\bench_out.nvc"

$cargas = @("test_math_simple", "stress_test", "demo_completo", "genericos", "utils", "test_stdlib_mini", "test_arr", "bench_fib")

# bench_fib vive en stdlib/compiler; mapear rutas especiales
function Get-SrcPath {
    param([string]$c)
    if ($c -eq "bench_fib") { return "$r\stdlib\compiler\bench_fib.nv" }
    return "$r\examples\$c.nv"
}

function Tiempo {
    param([scriptblock]$s)
    # warm-up + 3 mediciones, tomar la mejor (mínima)
    $best = [double]::MaxValue
    foreach ($n in 1..4) {
        $t = Measure-Command { & $s | Out-Null }
        if ($t.TotalSeconds -lt $best) { $best = $t.TotalSeconds }
    }
    return $best
}

Write-Output ("{0,-22} {1,10} {2,12} {3,10} {4,12} {5,8} {6,8}" -f "carga","RustCompile","LumenCompile","RustRun","LumenRun","xComp","xRun")
Write-Output ("-" * 96)

$rows = @()
foreach ($c in $cargas) {
    $src = Get-SrcPath $c
    $nvcRef = if ($c -eq "bench_fib") { "$r\stdlib\compiler\bench_fib.nvc" } else { "$r\examples\$c.nvc" }
    $relSrc = if ($c -eq "bench_fib") { "stdlib/compiler/bench_fib.nv" } else { "examples/$c.nv" }
    $relNvc = if ($c -eq "bench_fib") { "stdlib/compiler/bench_fib.nvc" } else { "examples/$c.nvc" }
    if (-not (Test-Path $src)) { continue }

    # --- Compilar (Rust) ---
    $tRc = Tiempo { & $lumen build $src }

    # --- Compilar (LÚMEN self-hosted) ---
    Set-Content -Path $target -Value ("$relSrc`n$work") -Encoding utf8 -NoNewline
    $tLc = Tiempo { & $lumen run $v4 }

    # --- Ejecutar (VM Rust) ---
    $tRr = Tiempo { & $lumen run $nvcRef }

    # --- Ejecutar (VM LÚMEN sobre el mismo .nvc Rust-built) ---
    Set-Content -Path $entrada -Value ($relNvc) -Encoding utf8 -NoNewline
    $tLr = Tiempo { & $lumen run $vm }

    $xcomp = if ($tRc -gt 0) { [math]::Round($tLc / $tRc, 1) } else { 0 }
    $xrun  = if ($tRr -gt 0) { [math]::Round($tLr / $tRr, 1) } else { 0 }
    Write-Output ("{0,-22} {1,10:N3} {2,12:N3} {3,10:N3} {4,12:N3} {5,8} {6,8}" -f $c, $tRc, $tLc, $tRr, $tLr, $xcomp, $xrun)
    $rows += [PSCustomObject]@{ carga=$c; rc=$tRc; lc=$tLc; rr=$tRr; lr=$tLr; xc=$xcomp; xr=$xrun }
}

Write-Output ""
Write-Output "=== Promedios ==="
if ($rows.Count) {
    $trc = ($rows | Measure-Object rc -Average).Average
    $tlc = ($rows | Measure-Object lc -Average).Average
    $trr = ($rows | Measure-Object rr -Average).Average
    $tlr = ($rows | Measure-Object lr -Average).Average
    Write-Output ("Rust compile avg: {0:N3}s   Lumen compile avg: {1:N3}s   (x{2:N1})" -f $trc, $tlc, ($tlc/$trc))
    Write-Output ("Rust run    avg: {0:N3}s   Lumen run    avg: {1:N3}s   (x{2:N1})" -f $trr, $tlr, ($tlr/$trr))
}
