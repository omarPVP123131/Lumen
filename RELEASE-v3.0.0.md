# LÚMEN v3.0.0

Esta versión unifica todo el trabajo de corrección iniciado sobre la v2.4.6:
**167 bugs corregidos** (los 8 del reporte original y 159 más encontrados de
forma activa), **720 pruebas automáticas** en verde y la biblioteca estándar
completa —91 ficheros, 69 módulos importables— sin un solo error.

## Por qué un número mayor

No se ha cambiado ninguna sintaxis: **el código que ya funcionaba sigue
funcionando igual**. El salto se justifica por la naturaleza de lo corregido.
Varios fallos eran *miscompilaciones y agujeros de tipos silenciosos*: el
compilador no protestaba y el programa hacía otra cosa. En esos casos el
comportamiento de la v3.0 es, deliberadamente, distinto al de la v2.4.x —porque
el anterior era incorrecto—.

## Lo más importante de esta entrega

- **La biblioteca estándar se puede usar.** Cuatro módulos (`logging`,
  `metrics`, `bpe`, `nn`) no se podían ni importar, y siete ficheros no pasaban
  `lumen check`. Hoy: **69/69 módulos importables, 91/91 ficheros válidos**, con
  un test guardián que recorre la biblioteca entera en cada ejecución de la
  suite.
- **Las importaciones anidadas funcionan.** El prefijo de módulo se aplicaba dos
  veces en cadenas `app → nn → tensor`, y los bindings de `si sea exito(v)` se
  tomaban por globales. Cualquier jerarquía de dos niveles estaba rota.
- **La salida ya no se pierde.** `imprimir` sólo volcaba su buffer al terminar el
  programa: un servidor, un TUI o un cuelgue no mostraban *nada*, ni siquiera lo
  impreso antes de bloquearse. Ahora se emite en directo.
- **Los backends nativos dan resultados correctos o se niegan a compilar.** Nunca
  un binario que devuelve valores erróneos en silencio.
- **El servidor LSP termina cuando debe.** Giraba a tope de CPU indefinidamente
  si el editor cerraba la tubería sin mandar `exit`.
- **`fmt` ya no corrompe el código.** Perdía el `prestado mut` del receptor, lo
  que convertía un método mutador en uno por valor.

El detalle bug a bug está en [`CHANGELOG.md`](CHANGELOG.md).

## Instalación

### Linux / macOS

```sh
curl -fsSL https://raw.githubusercontent.com/omarPVP123131/Lumen/main/scripts/install.sh | sh
```

El instalador verifica el SHA256 del artefacto contra `SHA256SUMS.txt` y aborta
si no coincide. Si no hay binario para tu plataforma, compila desde fuente
(requiere Rust).

### Windows

```powershell
irm https://raw.githubusercontent.com/omarPVP123131/Lumen/main/scripts/install.ps1 | iex
```

### Desde fuente

```sh
git clone https://github.com/omarPVP123131/Lumen.git
cd Lumen
cargo build --release
./target/release/lumen --version    # LÚMEN v3.0.0
```

## Verificar la descarga

```sh
sha256sum -c SHA256SUMS.txt
```

## Comprobación rápida tras instalar

```sh
lumen new mi_app
cd mi_app
lumen run src/main.nv
```

## Estado verificado de esta entrega

| Comprobación | Resultado |
|---|---|
| Suite de pruebas (workspace completo) | **720 passed / 0 failed** |
| Ficheros de `stdlib` que pasan `lumen check` | **91 / 91** |
| Módulos de `stdlib` importables | **69 / 69** |
| Ejemplos ejecutados | **166 OK / 0 fallos reales** |
| `clippy` | **0 avisos** |
| Paridad VM ↔ nativo ↔ bytecode | **3038 programas, 0 divergencias** |
| Idempotencia y semántica de `fmt` | **3095 programas, 0 fallos** |
| Acuerdo `lint` ↔ `check` y `lsp` ↔ `check` | **3095 programas, 0 desacuerdos** |
| Ciclo de paquetes `new → pack → install → importar → run` | **completo** |
| Código que enseña `lumen tutor` | **compila y da el resultado anunciado** |
| Fuzzer de structs/listas/`prestado mut` contra oráculo | **640 programas, 0 divergencias** |
| Fuzzer diferencial de regex VM ↔ nativo | **750 patrones, 1500 comparaciones, 0 divergencias** |

De los ejemplos, 6 son Windows-only (cargan `msvcrt.dll`, `ws2_32.dll` o
`user32.dll` por FFI) y 5 son interactivos —TUI, GUI, reloj— o bucles infinitos
deliberados usados para depurar el parser. Ninguno indica un fallo del lenguaje.

## Limitaciones conocidas

- **El backend nativo no cubre todos los builtins.** Los de fichero
  (`__leer_archivo`, `__escribir_archivo`, `__tamano_archivo`), `__desde_utf8` y
  `__regex_capturar` no están implementados en AOT. La política es explícita: si
  un programa los usa, `build --native` **se niega a compilar** y lo dice, en vez
  de emitir un binario que devolvería `void` en silencio. Usa la VM o el
  bytecode (`lumen build`) para esos programas, o `--permitir-no-soportados` si
  aceptas el riesgo.
- **Cranelift** rechaza legítimamente indexado y longitud de listas, decimales,
  `abs`, `minimo` y `maximo`.
- **FFI de Windows en Linux**: los ejemplos que cargan DLLs de Windows fallan con
  un error claro sobre el fichero que falta. Es lo esperado.
- **El REPL no persiste el estado** cuando se le alimenta por una tubería no
  interactiva.
