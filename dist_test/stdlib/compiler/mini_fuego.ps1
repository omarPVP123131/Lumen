# mini_fuego.ps1 — prueba rápida de N ejemplos con el pipeline puro
param([string[]]$Ejemplos)
$root = "C:\Users\Omar\Documents\Documentos WEb\LumenRust"
$v4 = "$root\stdlib\compiler\compiler_v4.nvc"
$targetFile = "$root\stdlib\compiler\target.txt"
$salida = "$root\stdlib\compiler\fuego_out.nvc"
foreach ($ej in $Ejemplos) {
    Set-Content -Path $targetFile -Value "examples/$ej.nv`n$salida" -Encoding utf8 -NoNewline
    $c = & $root\target\release\lumen.exe run $v4 2>&1 | Out-String
    $nvc = & $root\target\release\lumen.exe run $salida 2>&1 | Out-String
    $rust = & $root\target\release\lumen.exe run "examples/$ej.nv" 2>&1 | Out-String
    $ok = if ($nvc -eq $rust) { "CORRECTO" } else { "INCOMPATIBLE" }
    Write-Output ("{0,-24} {1}  nvc=[{2}] rust=[{3}]" -f $ej, $ok, $nvc.Trim().Replace("`n","|"), $rust.Trim().Replace("`n","|"))
}
