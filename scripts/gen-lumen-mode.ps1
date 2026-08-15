# gen-lumen-mode.ps1 — Genera el modo CodeMirror 6 de LÚMEN (F2.1)
# Fuente única: crates/lumen-lexer/src/token.rs (added: 13 Ago 2026)
# Uso: pwsh scripts/gen-lumen-mode.ps1  →  rewrite crates/lumen-wasm/web/vendor/lumen-mode.js
$ErrorActionPreference = 'Stop'
$scriptDir = $PSScriptRoot
$root = Split-Path $scriptDir -Parent
$tokenRs = Join-Path $root 'crates\lumen-lexer\src\token.rs'
$out = Join-Path $root 'crates\lumen-wasm\web\vendor\lumen-mode.js'

$src = Get-Content $tokenRs -Raw
# Keywords: líneas `"xxx" => Some(TokenKind::X),` del match de is_keyword
$kw = [System.Collections.Generic.List[string]]::new()
foreach ($m in [regex]::Matches($src, '"([a-z_]+)"\s*=>\s*Some\(TokenKind::')) {
    if (-not $kw.Contains($m.Groups[1].Value)) { $kw.Add($m.Groups[1].Value) }
}
$kwList = ($kw | Sort-Object | ForEach-Object { "'$_'" }) -join ', '

@"
// LUMEN CodeMirror 6 mode - GENERADO por scripts/gen-lumen-mode.ps1 (no editar a mano)
// Keywords desde crates/lumen-lexer/src/token.rs → regenerar al añadir keywords
import { StreamLanguage, LanguageSupport, HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

const KEYWORDS = new Set([$kwList]);

const strLit = /(?:[^\\"\n]|\\.)*/;
const token = (stream, state) => {
  const ch = stream.next();
  // Strings "..." con escapes \\n \\t \\\" \\\\
  if (ch === '"') {
    stream.eatWhile((c) => c !== '"' && c !== '\n');
    if (!stream.eol() && stream.peek() === '"') stream.next();
    return "string";
  }
  // Comentarios
  if (ch === '/') {
    if (stream.peek() === '/') { stream.skipToEnd(); return "lineComment"; }
    if (stream.peek() === '*') { stream.skipTo('*/') || stream.skipToEnd(); if (stream.peek() === '*') { stream.next(); stream.next(); } return "blockComment"; }
    return "operator";
  }
  if (ch === '0' && (stream.peek() === 'x' || stream.peek() === 'X')) {
    stream.next(); stream.eatWhile(/[0-9a-fA-F]/); return "number";
  }
  if (/\d/.test(ch)) {
    let sawDot = false;
    while (true) {
      const p = stream.peek();
      if (/\d/.test(p || '')) { stream.next(); }
      else if (p === '.' && !sawDot && stream.peek(1) !== '.') { sawDot = true; stream.next(); }
      else break;
    }
    return "number";
  }
  if (/\w/.test(ch) || ch === '_') {
    stream.eatWhile(/\w/);
    const w = stream.current().slice(0);
    return KEYWORDS.has(w) ? "keyword" : "variableName";
  }
  return "operator";
};

// Espacios: defino via indentation para que StreamLanguage no los consuma mal
const lumen = StreamLanguage.define({
  name: "lumen",
  token,
  startState: () => ({}),
  blankLine: () => {},
  tokenTable: {},
});

// Resaltado Catppuccin Mocha (paleta del playground; classification por @lezer/highlight)
const lumenHighlighting = HighlightStyle.define([
  { tag: [t.keyword, t.modifier], color: "#cba6f7" },
  { tag: [t.string, t.special(t.string)], color: "#a6e3a1" },
  { tag: [t.number, t.bool, t.atom], color: "#fab387" },
  { tag: t.comment, color: "#6c7086", fontStyle: "italic" },
  { tag: [t.variableName, t.function(t.variableName)], color: "#cdd6f4" },
  { tag: t.operator, color: "#89b4fa" },
  { tag: t.bracket, color: "#94e2d5" },
]);

// LanguageSupport: StreamLanguage devuelve un Language, no una extension directa;
// el HighlightStyle tampoco lo es — se envuelve con syntaxHighlighting()
const lumenSupport = new LanguageSupport(lumen, [syntaxHighlighting(lumenHighlighting)]);

export { lumenSupport as lumenLanguage, lumenHighlighting };
"@ | Set-Content -Path $out -Encoding utf8 -NoNewline

Write-Host "lumen-mode.js generado: $($kw.Count) keywords -> $out"