# Genera crates/lumen-wasm/web/embedded_examples.js desde examples/*.nv
# Fallback offline del playground (F4.1): funciona en file:// y GitHub Pages
# sin servidor. Regenerar tras añadir/modificar ejemplos:
#   pwsh scripts/gen-embedded-examples.ps1
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Resolve-Path (Join-Path $scriptDir "..")
$examplesDir = Join-Path $root "examples"
$outPath = Join-Path $root "crates\lumen-wasm\web\embedded_examples.js"

$files = Get-ChildItem $examplesDir -Filter "*.nv" | Sort-Object Name
$parts = @()
foreach ($f in $files) {
    $name = $f.BaseName
    $code = [System.IO.File]::ReadAllText($f.FullName)
    $code = $code.Replace("\", "\\").Replace("`r", "").Replace("`n", "\n").Replace('"', '\"').Replace("`t", "\t")
    $desc = ""
    foreach ($line in [System.IO.File]::ReadAllLines($f.FullName)) {
        $t = $line.TrimStart()
        if ($t.StartsWith("//")) { $desc = $t.Substring(2).Trim(); break }
        if ($t -ne "") { break }
    }
    $desc = $desc.Replace("\", "\\").Replace('"', '\"')
    $parts += "  { name: `"$name`", file: `"$($f.Name)`", description: `"$desc`", code: `"$code`" }"
}
$content = @"
// Generado por scripts/gen-embedded-examples.ps1 — NO editar a mano.
// ${parts.Count} ejemplos embebidos de examples/ (fallback offline del playground).
window.LUMEN_EMBEDDED_EXAMPLES = [
$($parts -join ",`n")
];
"@
[System.IO.File]::WriteAllText($outPath, $content, (New-Object System.Text.UTF8Encoding $false))
Write-Host "embedded_examples.js generado: $((Get-Item $outPath).Length) bytes ($($parts.Count) ejemplos)"