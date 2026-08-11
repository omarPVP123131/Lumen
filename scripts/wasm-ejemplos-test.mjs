// Ejecuta UN archivo .nv en el runtime WASM de LÚMEN y devuelve la salida
// (o un prefijo W-ERROR/W-TRAP si falla). Usado por wasm_fuego.ps1.
// Uso: node scripts/wasm-ejemplos-test.mjs examples/foo.nv
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const file = process.argv[2];
if (!file) {
  console.log('W-ERROR: falta archivo');
  process.exit(1);
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const pkgDir = path.join(__dirname, '..', 'crates', 'lumen-wasm', 'pkg');
const wasmBytes = readFileSync(path.join(pkgDir, 'lumen_wasm_bg.wasm'));
const pkgUrl = pathToFileURL(path.join(pkgDir, 'lumen_wasm.js')).href;
const init = (await import(pkgUrl)).default;
const { LumenRuntime } = await import(pkgUrl);
await init({ module_or_path: wasmBytes });

// Inyecta los .nv del directorio del archivo como proyecto (multi-archivo),
// imitando la resolución relativa del CLI.
const repoRoot = path.resolve(__dirname, '..');
const fileAbs = path.resolve(file);
const dir = path.dirname(fileAbs);
const names = [];
const contents = [];
for (const entry of readdirSync(dir)) {
  if (entry.endsWith('.nv')) {
    names.push(entry);
    contents.push(readFileSync(path.join(dir, entry), 'utf8'));
  }
}

const rt = new LumenRuntime();
const source = readFileSync(fileAbs, 'utf8');
try {
  const out = names.length
    ? rt.run_with_files(source, names, contents)
    : rt.run(source);
  console.log(out.replace(/\n$/, ''));
} catch (e) {
  console.log('W-TRAP:', e && e.message ? e.message : String(e));
  process.exit(2);
}