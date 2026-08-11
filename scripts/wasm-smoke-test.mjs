// Smoke test del runtime WASM de LÚMEN — Node puro (sin librerías Rust).
// Importa el pkg generado por wasm-pack y verifica run/check/compile con
// la stdlib embebida (loader virtual).
// Uso: node scripts/wasm-smoke-test.mjs
import { readFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const pkgDir = path.join(__dirname, '..', 'crates', 'lumen-wasm', 'pkg');
const wasmBytes = readFileSync(path.join(pkgDir, 'lumen_wasm_bg.wasm'));

const pkgUrl = pathToFileURL(path.join(pkgDir, 'lumen_wasm.js')).href;
const init = (await import(pkgUrl)).default;
const { LumenRuntime } = await import(pkgUrl);
await init(wasmBytes);

let passed = 0;
let failed = 0;

function test(name, actual, expected) {
  const ok = actual === expected;
  if (ok) { passed++; console.log(`  ✓ ${name}`); }
  else { failed++; console.log(`  ✗ ${name}\n      esperado: ${JSON.stringify(expected)}\n      actual:   ${JSON.stringify(actual)}`); }
}

function testContains(name, actual, fragment) {
  const ok = actual.includes(fragment);
  if (ok) { passed++; console.log(`  ✓ ${name}`); }
  else { failed++; console.log(`  ✗ ${name}\n      esperaba contener: ${JSON.stringify(fragment)}\n      actual: ${JSON.stringify(actual)}`); }
}

const rt = new LumenRuntime();
console.log(`LÚMEN WASM v${LumenRuntime.version()} — smoke test`);

// ── Core ────────────────────────────────────────────────────────────
test('hola mundo', rt.run('imprimir("Hola, LÚMEN!");'), 'Hola, LÚMEN!');
test('aritmética', rt.run('imprimir(40 + 2, " ", 6 * 7);'), '42\n \n42');
test('funciones', rt.run('funcion entero f(entero x) { retornar x * 3; }\nimprimir(f(14));'), '42');

// ── Stdlib embebida (loader virtual) ─────────────────────────────────
test('importar texto.nv', rt.run('importar "texto.nv";\nimprimir(texto_mayusculas("hola"));'), 'HOLA');
test('importar coleccion.nv', rt.run('importar "coleccion.nv";\nlista<entero> l = [3, 1, 2, 2];\nimprimir(coleccion_contar(l, 2));'), '2');
test('importar matematicas.nv', rt.run('importar "matematicas.nv";\nimprimir(matematicas_potencia(2, 10));'), '1024');

// ── Builtins de filesystem: no deben panickear ───────────────────────
const fsOut = rt.run('imprimir(__existe_archivo("nada.nv"));');
testContains('filesystem builtin no panickea', fsOut.toLowerCase(), 'false');

// ── check ────────────────────────────────────────────────────────────
test('check válido', String(rt.check('imprimir("ok");')), 'undefined');
testContains('check inválido', String(rt.check('imprimir(')), 'Error');

// ── compile_to_bytes (F9.1): .nvc válido ─────────────────────────────
const bytes = rt.compile_to_bytes('imprimir(42);');
const magic = bytes.slice(0, 4).join(',');
test('compile_to_bytes magic LUMN', magic, '76,85,77,78');
test('compile_to_bytes no vacío', bytes.length > 8, true);

// ── Errores controlados ──────────────────────────────────────────────
testContains('error sintáctico reportado', rt.run('imprimir(1 +'), 'Error');

console.log(`\n${passed} pasaron, ${failed} fallaron`);
process.exit(failed > 0 ? 1 : 0);
