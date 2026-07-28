# LÚMEN Engineer Skill

You are a LÚMEN language engineer. Follow these rules when working on the LÚMEN project.

## Language Rules
- ALL stdlib code must have dual ES/EN syntax (Spanish primary, English alias)
- Function pattern: `funcion Tipo nombre_es(params) { ... }` then `funcion Tipo name_en(params) { nombre_es(params); }`
- Module prefix is the filename without extension (e.g., `texto.nv` → `texto_` prefix)
- Use `entero` for integers, `decimal` for floats, `texto` for strings, `booleano` for bools
- Use `lista<T>` for arrays, `diccionario<K,V>` for maps, `numero` for dynamic number/map

## Builtins Available
- `__ffi_cargar/load` — Load DLL
- `__ffi_llamar/call` — Call DLL function
- `__ffi_asignar/alloc` — Allocate memory
- `__ffi_liberar/free` — Free memory
- `__ffi_escribir/write` — Write bytes
- `__ffi_leer/read` — Read bytes
- `__ffi_peek` — Read u32
- `__ffi_poke` — Write u32
- `__map_*` — Map operations
- `__str_*` — String operations
- `__regex_*` — Regex operations
- `__json_*` — JSON parse/stringify
- `__tipo_de` — Type introspection
- `__tarea_*` — Async task operations
- `__coro_*` — Coroutine operations
- `__hash_sha256/sha512` — SHA hashing
- `__aes_encriptar/desencriptar` — AES encrypt/decrypt
- `__jwt_codificar/decodificar` — JWT encode/decode
- `__tiempo_ahora/formatear/parsear` — Time operations
- `__gui_*` — GUI window operations
- `__fs_listar/leer_archivo/escribir_archivo` — File operations
- `__js_eval/call` — JS interop

## Code Style
- `//` for comments, `///` for doc comments
- 4 spaces for indentation
- Open braces on same line: `funcion void foo() {`
- Match syntax: `elegir (expr) { caso Patron: accion; }`
- String concatenation: `"hola " + nombre`
- Struct definition: `estructura Nombre { campo: tipo, }`
- Import: `importar "modulo.nv";`

## Architecture
- Rust: minimal OS-level builtins (FFI, crypto, threading)
- LÚMEN: everything else (stdlib, games, GUI, TUI, charts)
- Tests go in `crates/lumen-vm/tests/e2e.rs`
- Examples go in `examples/`
- Stdlib goes in `stdlib/`
