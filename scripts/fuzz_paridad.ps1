# fuzz_paridad.ps1 — Paridad VM vs nativo con tolerancia a no-determinismo (v3.3.7)
# Uso: pwsh scripts/fuzz_paridad.ps1 [archivos.nv...]
param([string[]]$Files)
if (-not $Files -or $Files.Count -eq 0) { $Files = Get-ChildItem "$PSScriptRoot\..\fuzz\*.nv" | ForEach-Object { $_.FullName } }
$reglas = @(
    @{ re = 'coro_\d+';                 sub = 'coro_N' },      # ids de corutina
    @{ re = '\d{10,}';                  sub = '<EPOCH>' },     # timestamps unix
    @{ re = '\d+\.\d+(ms|s|µs)';        sub = '<DUR>' },       # duraciones
    @{ re = '(?m)^pid=\d+$';            sub = 'pid=N' },       # pids FFI
    @{ re = '0x[0-9a-fA-F]{8}';         sub = '<PTR>' }        # punteros ffi
)
function Normaliza($s) {
    foreach ($r in $reglas) { $s = $s -replace $r.re, $r.sub }
    return ($s -split "`r?`n" | Where-Object { $_.Trim() -ne "" })
}
$par = 0; $dif = 0; $falla = 0
foreach ($f in $Files) {
    $vmOut   = & cargo run --quiet --bin lumen -- run $f 2>&1
    $dir  = [IO.Path]::GetDirectoryName($f)
    $base = Join-Path $dir ([IO.Path]::GetFileNameWithoutExtension($f))
    cargo run --quiet --bin lumen -- build --native $f 2>&1
    # v3.5.41: en Linux `build --native` produce el binario SIN extensión .exe
    $tmpExe = if (Test-Path "$base.exe") { "$base.exe" } elseif (Test-Path $base) { $base } else { $null }
    if (-not $tmpExe) { Write-Host "FALLA-COMPILA $f"; $falla++; continue }
    $cOut = & $tmpExe 2>&1
    $v = @(Normaliza ($vmOut -join "`n"))
    $n = @(Normaliza ($cOut -join "`n"))
    $diff = Compare-Object $v $n
    if (-not $diff) { Write-Host "PAR  $(Split-Path -Leaf $f)"; $par++ }
    else { Write-Host "DIF  $(Split-Path -Leaf $f)"; $diff | ForEach-Object { "   {0} {1}" -f $_.SideIndicator, $_.InputObject }; $dif++ }
}
Write-Host "── RESUMEN: PAR=$par DIF=$dif FALLA=$falla ──"
