param(
    [string[]]$Archivos
)
# Compara la VM Rust vs la VM en LUMEN (vm.nv) sobre varios .nv
$root = "C:\Users\Omar\Documents\Documentos WEb\LumenRust"
$lumen = "$root\target\release\lumen.exe"
$vm = "$root\stdlib\compiler\vm.nvc"
$entrada = "$root\stdlib\compiler\entrada_vm.txt"
$fallidos = 0
$ok = 0

foreach ($f in $Archivos) {
    if (-not (Test-Path "$root\examples\$f.nv")) { Write-Output "SKIP (no existe): $f"; continue }
    # Compilar con el compilador Rust
    $buildOut = & $lumen build "$root\examples\$f.nv" 2>&1 | Out-String
    if ($buildOut -match "error|Error") {
        Write-Output "COMPILA-FALLA: $f"
        continue
    }
    $rust = (& $lumen run "$root\examples\$f.nvc" 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { $rust = "RUNTIME-ERROR: $rust" }
    Set-Content -Path $entrada -Value "examples/$f.nvc" -NoNewline
    $lvm = (& $lumen run $vm 2>&1 | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { $lvm = "RUNTIME-ERROR: $lvm" }
    if ($rust -eq $lvm) {
        Write-Output "OK: $f"
        $ok++
    } else {
        Write-Output "DIFF: $f"
        Write-Output "  Rust : $rust"
        Write-Output "  Lumen: $lvm"
        $fallidos++
    }
}
Write-Output "---"
Write-Output "OK=$ok FALLAS=$fallidos"
