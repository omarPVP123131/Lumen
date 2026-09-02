# ============================================================================
# LUMEN - Verificacion del fixpoint self-hosting (v3.5.11 - FIXPOINT, con progreso en vivo)  [PowerShell]
#
# Corre desde la RAIZ del repo Lumen (pwsh 7 recomendado):
#     pwsh scripts/verificar_fixpoint.ps1
#   (o:  powershell -ExecutionPolicy Bypass -File scripts\verificar_fixpoint.ps1)
#
# Que hace:
#   0. Construye el binario release si no existe (cargo build --release).
#   1. Genera compiler_v4.nvc (compilador self-hosted compilado por RUST).
#   2. STAGE 1: compiler_v4.nvc compila compiler_v4.nv -> v4_self.nvc   (~5 s con lexer nativo)
#   3. Verifica que v4_self.nvc NO tenga funciones duplicadas.
#   4. PROBE: v4_self.nvc compila fuzz/selfhost_probe.nv -> debe imprimir 42.
#   5. STAGE 2: v4_self.nvc recompila compiler_v4.nv -> v4_self2.nvc     (~5 s con lexer nativo)
#   6. Compara v4_self.nvc == v4_self2.nvc (byte-identical = FIXPOINT).
#
# Resultado: escribe reports\fixpoint_status.md e imprime un resumen.
# Duracion aprox: <1 min (dos autocompilaciones de ~150KB, lexer nativo).
# ============================================================================
$ErrorActionPreference = "Continue"   # no matar por stderr de comandos nativos
$ProgressPreference = "SilentlyContinue"
Set-Location -Path (Split-Path -Parent $PSScriptRoot)   # raiz del repo
$Root   = (Get-Location).Path
$Lumen  = Join-Path $Root "target\release\lumen.exe"
$Report = Join-Path $Root "reports\fixpoint_status.md"
$Tmp    = $env:TEMP
New-Item -ItemType Directory -Force -Path (Join-Path $Root "reports") | Out-Null

function Log($msg) {
    Write-Host $msg
    Add-Content -Path $Report -Value $msg -Encoding utf8
}
Set-Content -Path $Report -Value "" -Encoding utf8

Log "# Fixpoint self-hosting - $(Get-Date)"
Log "host: $([System.Environment]::OSVersion.VersionString)"

# 0) binario release
if (-not (Test-Path $Lumen)) {
    Log "## Binario release ausente - construyendo (cargo build --release)..."
    Push-Location $Root
    & cargo build --release --bin lumen *> (Join-Path $Tmp "fp_build.log")
    Get-Content (Join-Path $Tmp "fp_build.log") -ErrorAction SilentlyContinue | Add-Content -Path $Report -Encoding utf8
    Pop-Location
    if (-not (Test-Path $Lumen)) { Log "FALLO: cargo build --release"; exit 1 }
}
Log "binario: $Lumen"

$self1 = Join-Path $Tmp "v4_self.nvc"
$self2 = Join-Path $Tmp "v4_self2.nvc"
$self1f = $self1.Replace('\', '/')
$self2f = $self2.Replace('\', '/')

# 1) compiler_v4.nvc via RUST
Log "## Paso 1: compiler_v4.nvc (compilado por Rust)..."
& $Lumen build "stdlib\compiler\compiler_v4.nv" *> (Join-Path $Tmp "fp_step1.log")
Get-Content (Join-Path $Tmp "fp_step1.log") -ErrorAction SilentlyContinue | Add-Content -Path $Report -Encoding utf8

# 2) STAGE 1
Log "## Paso 2 (STAGE 1): autocompilar compiler_v4.nv (~5 s con lexer nativo)..."
Set-Content -Path "stdlib\compiler\target.txt" -Value "stdlib/compiler/compiler_v4.nv`n$self1f" -NoNewline -Encoding ascii
Remove-Item $self1 -ErrorAction SilentlyContinue
Remove-Item $self2 -ErrorAction SilentlyContinue
$sw = [Diagnostics.Stopwatch]::StartNew()
& $Lumen run "stdlib\compiler\compiler_v4.nvc" 2>&1 |
    Tee-Object -FilePath (Join-Path $Tmp "fp_stage.log") |
    ForEach-Object { Write-Host "  | $_" }
Get-Content (Join-Path $Tmp "fp_stage.log") -ErrorAction SilentlyContinue | Add-Content -Path $Report -Encoding utf8
$sw.Stop()
if (-not (Test-Path $self1)) { Log "FALLO: stage1 no genero v4_self.nvc"; exit 1 }
Log "v4_self.nvc: $((Get-Item $self1).Length) bytes  ($('{0:N1}' -f $sw.Elapsed.TotalMinutes) min)"

# 3) duplicados?
$dis = Join-Path $Tmp "dis_self.txt"
& $Lumen disasm $self1 > $dis 2>&1
$names = Select-String -Path $dis -Pattern 'name=([a-zA-Z_0-9]+)' -AllMatches |
    ForEach-Object { $_.Matches } | ForEach-Object { $_.Groups[1].Value }
$dups = $names | Group-Object | Where-Object { $_.Count -gt 1 } | ForEach-Object { $_.Name }
if ($dups) { Log "funciones duplicadas en v4_self: [$($dups -join ', ')]" }
else       { Log "funciones duplicadas en v4_self: [NINGUNA OK]" }
Log "total funciones: $(($names | Select-Object -Unique).Count)"

# 4) PROBE via self
Log "## Paso 3 (PROBE): v4_self.nvc compila selfhost_probe.nv (esperado: 42)..."
$fpc = Join-Path $Tmp "fpc"
New-Item -ItemType Directory -Force -Path (Join-Path $fpc "stdlib\compiler") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $fpc "fuzz") | Out-Null
Copy-Item "fuzz\selfhost_probe.nv" (Join-Path $fpc "fuzz\selfhost_probe.nv") -Force
Set-Content -Path (Join-Path $fpc "stdlib\compiler\target.txt") -Value "fuzz/selfhost_probe.nv`nfuzz/p.nvc" -NoNewline -Encoding ascii
Push-Location $fpc
& $Lumen run $self1 *> (Join-Path $Tmp "fp_probe_comp.log")
$probe = (& $Lumen run "fuzz\p.nvc" | Out-String).Trim()
Pop-Location
Log "probe via SELF-COMPILED = [$probe] (esperado 42)"

# 5) STAGE 2
Log "## Paso 4 (STAGE 2): v4_self.nvc recompila compiler_v4.nv (~5 s con lexer nativo)..."
Set-Content -Path "stdlib\compiler\target.txt" -Value "stdlib/compiler/compiler_v4.nv`n$self2f" -NoNewline -Encoding ascii
Remove-Item $self2 -ErrorAction SilentlyContinue
$sw2 = [Diagnostics.Stopwatch]::StartNew()
& $Lumen run $self1 2>&1 |
    Tee-Object -FilePath (Join-Path $Tmp "fp_stage2.log") |
    ForEach-Object { Write-Host "  | $_" }
Get-Content (Join-Path $Tmp "fp_stage2.log") -ErrorAction SilentlyContinue | Add-Content -Path $Report -Encoding utf8
$sw2.Stop()

# 6) comparar
if (Test-Path $self2) {
    $h1 = (Get-FileHash -Algorithm SHA256 $self1).Hash
    $h2 = (Get-FileHash -Algorithm SHA256 $self2).Hash
    if ($h1 -eq $h2) {
        Log "## FIXPOINT: v4_self.nvc y v4_self2.nvc BYTE-IDENTICAL ($((Get-Item $self1).Length) B)"
        Log "sha256: $h1"
    } else {
        Log "## FIXPOINT ROTO: self=$((Get-Item $self1).Length) B vs self2=$((Get-Item $self2).Length) B"
        Log "sha256 self : $h1"
        Log "sha256 self2: $h2"
    }
} else {
    Log "## FALLO: stage2 no genero v4_self2.nvc (revisar fin del log)"
}
Log "fin: $(Get-Date)"
Write-Host "---------------------------------------------"
Get-Content $Report
