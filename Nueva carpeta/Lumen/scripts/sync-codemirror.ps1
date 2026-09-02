# sync-codemirror.ps1 — Regenera el vendor CM6 local (F2.1) — reproducible
# Uso: pwsh scripts/sync-codemirror.ps1  (requiere npm + red)
$ErrorActionPreference = 'Stop'
$scriptDir = $PSScriptRoot
$root = Split-Path $scriptDir -Parent
$web = Join-Path $root 'crates\lumen-wasm\web'
$vendor = Join-Path $web 'vendor'
$tmp = Join-Path $web 'vendor\.npm-tmp'

Write-Host "→ Instalando paquetes CodeMirror 6 en $tmp"
New-Item -ItemType Directory -Path $tmp -Force | Out-Null
Push-Location $tmp
try {
    npm install --no-save @codemirror/state@6 @codemirror/view@6 @codemirror/language@6 @codemirror/commands@6
    if ($LASTEXITCODE -ne 0) { throw "npm install falló ($LASTEXITCODE)" }
} finally { Pop-Location }

$nm = Join-Path $tmp 'node_modules'
$dest = Join-Path $vendor 'cm'
New-Item -ItemType Directory -Path $dest -Force | Out-Null

function Copy-Pkg($from, $to) {
    Copy-Item (Join-Path $nm $from) (Join-Path $dest $to) -Force
    Write-Host "  -> $to"
}

Copy-Pkg '@codemirror\commands\dist\index.js' 'cm-commands.js'
Copy-Pkg '@codemirror\language\dist\index.js'  'cm-language.js'
Copy-Pkg '@codemirror\state\dist\index.js'     'cm-state.js'
Copy-Pkg '@codemirror\view\dist\index.js'      'cm-view.js'
Copy-Pkg '@lezer\common\dist\index.js'         'lezer-common.js'
Copy-Pkg '@lezer\highlight\dist\index.js'      'lezer-highlight.js'
Copy-Pkg '@lezer\lr\dist\index.js'             'lezer-lr.js'
Copy-Pkg '@marijn\find-cluster-break\src\index.js' 'marijn-find-cluster-break.js'
Copy-Pkg 'style-mod\src\style-mod.js'          'style-mod.js'
Copy-Pkg 'w3c-keyname\index.js'                'w3c-keyname.js'
Copy-Pkg 'crelt\index.js'                      'crelt.js'

Remove-Item $tmp -Recurse -Force
Write-Host "✓ vendor/codemirror sincronizado -> $dest"
Write-Host "→ Regenerando modo LUMEN"
pwsh "$scriptDir\gen-lumen-mode.ps1"