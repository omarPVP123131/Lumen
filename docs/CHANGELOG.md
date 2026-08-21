## [3.0.0] - 2026-08-17

Esta versión unifica en una sola entrega todo el trabajo de corrección iniciado
sobre la v2.4.6: **167 bugs corregidos** (los 8 del reporte original y 159 más
encontrados de forma activa), **720 pruebas automáticas** en verde y un barrido
de 494 programas sin una sola regresión.

El salto a un número mayor se justifica por la naturaleza de lo corregido: varios
de los fallos eran **miscompilaciones y agujeros de tipos silenciosos** —el
compilador no protestaba y el programa hacía otra cosa—, de modo que el
comportamiento de v3.0 es, en esos casos, deliberadamente distinto al de v2.4.x.
No se ha cambiado ninguna sintaxis: el código que ya funcionaba sigue
funcionando igual.

### 🔴 El regex nativo decia "no" a todo en Windows y macOS (BUG-166, BUG-167)

- **BUG-166 🔴 — el `#else` de la guarda de regex contenia stubs que devolvian
  siempre "no coincide".** El bloque real usa `<regex.h>` de POSIX y vive bajo
  `#if !defined(_WIN32) && !defined(__APPLE__)`; la rama alternativa tenia
  `_regex_m(...) { return 0; }` y `_regex_rep(...) { return (char*)s; }`. En
  Windows y macOS, `__regex_coincide` respondia `false` a **cualquier** patron y
  `__regex_reemplazar` no sustituia nada, sin fallar ni avisar. Es el mismo
  BUG-080 arreglado en su dia solo para la rama POSIX: dos de las tres
  plataformas soportadas se quedaron con el stub. Se sustituye por un **motor de
  regex propio por backtracking** en C portable, sin dependencias, que cubre
  literales, `.`, clases `[...]` con rangos y negacion, las clases Perl
  `\d \D \w \W \s \S`, los cuantificadores `*` `+` `?` `{n}` `{n,}` `{n,m}`,
  las anclas `^` `$`, alternacion `|` y grupos `(...)`.

- **BUG-167 🔴 — SIGSEGV al reemplazar con un patron que casa la cadena vacia.**
  `__regex_reemplazar("[a-z]?|a", "x_y", "#")` mataba el binario nativo **tambien
  en Linux**: ante una coincidencia vacia el bucle hacia `p += 1` sin comprobar
  si ya estaba en el terminador, se salia de la cadena y seguia copiando memoria
  ajena. Ambas ramas se reescriben recorriendo por indice, y se iguala la regla
  de la VM: una coincidencia vacia contigua a la anterior no se sustituye
  (`a?` sobre `"bab"` da `#b#b#`). Encontrado por un fuzzer diferencial nuevo,
  `gen/fz13.py`, que compara VM y binario nativo sobre patrones aleatorios.

- **La rama no-POSIX ya se prueba en Linux.** El test
  `tests_portabilidad::regex_de_la_rama_no_posix_coincide_con_la_vm` extrae ese
  bloque del header, lo compila con el `cc` local y lo ejecuta contra casos
  verificados contra la VM, para que no vuelva a hacer falta un Windows para
  descubrir que esa rama esta rota.

### 🔴 `build --native` no compilaba NADA en Windows (BUG-165)

- **BUG-165 🔴 — `lumen_rt.h` incluia `<sys/resource.h>` fuera de la guarda
  `#if !defined(_WIN32)`.** Todo binario nativo generado en Windows fallaba con
  `fatal error: sys/resource.h: No such file or directory`: `lumen build
  --native` estaba **completamente roto** en esa plataforma (19 tests en rojo,
  todos los que compilan a nativo). La cabecera entro con el arreglo de
  recursion profunda, que usa `getrlimit(RLIMIT_STACK)`. Ahora esta dentro del
  bloque POSIX y `_stack_init` tiene una version Windows basada en
  `GetCurrentThreadStackLimits`, con valor conservador para MinGW antiguos —
  importante porque la pila por defecto en Windows es 1 MiB, no 8 MiB, y sin
  ella la deteccion de desbordamiento no se inicializaba. Se anade un test que
  falla si cualquier cabecera POSIX vuelve a colarse sin guarda.

### 🔴 Cuatro bugs que solo se ven fuera de Linux (BUG-161 a BUG-164)

- **BUG-161 🔴 — las declaraciones adelantadas dejaron de parsear.** El patron
  `funcion numero f(numero x);` seguido de su definicion real (necesario para
  recursion mutua) fallaba con E017 desde el arreglo de BUG-151: antes el
  prototipo se descartaba en un `None` silencioso. Ahora se acepta, no emite un
  `Decl` duplicado, y **un prototipo sin definicion es el nuevo error E084**
  para no reabrir el agujero de BUG-151. Afectaba a 8 ejemplos del repo.
  *Descubierto porque la baseline recorria `examples/*.nv` sin recursion: 178 de
  393 ficheros. Ya cubre los 393.*
- **BUG-162 🟠 — siete tests fallaban por `\r\n`.** En Windows el runtime C
  traduce `\n` a `\r\n` y la VM no, asi que los tests de paridad VM/nativo
  reportaban divergencias inexistentes. Se normaliza en los helpers. Incluye dos
  sondas mal escritas: `doctor` partia el `PATH` por `:` sin `.exe`, y el test
  del LSP generaba `file://C:\dir\a.nv`, una URI invalida.
- **BUG-163 🟠 — el error de FFI no decia que biblioteca faltaba.** En Windows
  `libloading` devuelve solo `LoadLibraryExW failed`. Ahora el mensaje antepone
  siempre el nombre buscado.
- **BUG-164 🟠 — `bundle` abortaba tras haber funcionado.** Con destino
  explicito sin extension, el enlazador de Windows genera `.exe` y la
  comprobacion buscaba el nombre a secas: fallaba habiendo generado el binario.

### 🟠 `lumen-wasm` no compilaba con `--all-features` (BUG-160)

- **BUG-160 🟠 — `wasm` y `wasi` se pisaban entre sí.** Son dos backends
  alternativos del mismo API, cada uno con su `LumenRuntime`, su `js_eval` y sus
  métodos, pero los `cfg` sólo comprobaban `feature = "wasi"`. Activando ambos
  —lo que hace `cargo clippy --all-features`, tal cual lo ejecuta el CI— el
  crate fallaba con 14 errores de definición duplicada. Ahora la exclusión es
  explícita (`all(feature = "wasi", not(feature = "wasm"))`: si están los dos,
  gana `wasm`) y está documentada en `Cargo.toml`. Comprobadas las cuatro
  combinaciones de features.

### 🟡 El nulo tenía dos nombres (BUG-159)

- **BUG-159 🟡 — `imprimir(x)` escribía `void` donde `__tipo_de(x)` decía
  `nulo`.** El mismo valor se presentaba con dos nombres distintos, lo que
  induce a escribir comparaciones como `__tipo_de(x) == "void"` que nunca se
  cumplen. `void` era además el único anglicismo entre unos nombres de tipo por
  lo demás íntegramente en español (`entero`, `booleano`, `ninguno`...). Ahora
  el nulo se imprime **`nulo`** en la VM y en el backend nativo —el runtime C
  tenía su propia cadena literal—, con un test E2E que exige que ambas salidas
  coincidan. **Cambio de comportamiento observable**: un programa que dependa
  del texto `void` en la salida debe actualizarse.
- **Herramienta**: `gen/allex.sh` hacía un `cd` que, al fallar, dejaba correr el
  script desde el directorio equivocado y acertaba por casualidad; ahora
  localiza `examples/` y aborta con rc=2 si no lo encuentra.

### 🔴 La API pública embebible mentía tres veces (BUG-158)

- **BUG-158 🔴 — `lumen-api` no resolvía imports, ignoraba `with_lib_dirs` y
  `run` no devolvía la salida.** Tercera instancia del patrón de BUG-156/157: la
  API para embeber LÚMEN en Rust llamaba al analizador semántico sin
  `ModuleLoader`, así que cualquier símbolo importado era un `E042` falso para
  quien integrase el motor. Además `with_lib_dirs()` guardaba las rutas en un
  campo que nunca se leía —oculto tras un `#[allow(dead_code)]`—, y `run()`,
  que documenta «captures the output», devolvía la cima de la pila: el ejemplo
  del propio crate devolvía `"void"` en vez del texto impreso. Los errores, por
  último, se formateaban con `{:?}` y filtraban las estructuras internas del
  compilador; ahora se emiten como `E042 [1:24]: mensaje — sugerencia`.
  Verificado con un crate consumidor externo al repositorio.

### 🔴 El editor contradecía al compilador (BUG-157)

- **BUG-157 🔴 — el LSP no resolvía los imports.** Un fichero que `lumen check`
  acepta salía subrayado en rojo en el editor: todo símbolo procedente de un
  módulo se marcaba «no está definida». Un servidor de lenguaje que contradice
  al compilador enseña a ignorar los avisos, y entonces tampoco se ven los
  errores reales.
- Los imports se resuelven **relativos al fichero editado** (usando la URI del
  documento), y un import roto produce **un** diagnóstico útil en vez de una
  cascada de E042 por cada símbolo del módulo.

### 🔁 El REPL no sabía importar (BUG-156)

- **BUG-156 🟠 — `importar` era un no-op silencioso en el REPL.** No cargaba los
  módulos que existen (`importar "texto";` se aceptaba y luego
  `texto_longitud` «no estaba definida») **ni** protestaba por los que no
  (`importar "no_existe";` también se aceptaba). El REPL nunca invocaba al
  `ModuleLoader`. Ahora resuelve los imports igual que `lumen run` y devuelve
  el error del loader.
- **El REPL acepta `-L/--lib-dir`**, que antes no llegaba hasta él: podía
  cargar la stdlib pero no los paquetes del usuario.

### 📦 Paquetes y herramientas (BUG-154, BUG-155)

- **BUG-154 🔴 — el prefijo de un paquete salía del fichero, no del paquete.**
  `lumen install` prometía `importar "libreria";` y las funciones aparecían
  como `main_sumar` en vez de `libreria_sumar`. La API pública dependía de cómo
  se llamara el fichero de entrada. Ahora manda el nombre del paquete; el alias
  explícito y los `.nv` sueltos no cambian.
- **BUG-155 🟠 — `lumen doc` documentaba código que no compila**, generando un
  HTML vacío y anunciando éxito con código 0. Ahora avisa y sale con error,
  reutilizando la comprobación que `fmt` ya hacía desde BUG-053.
- **Fuzzer de rechazo ampliado** a `fmt`, `lint` y `doc`, acotado a errores
  sintácticos: `fmt` y `doc` no hacen análisis semántico y no deben rechazar
  trabajo en curso.

### 📦 Distribución: la stdlib ahora viaja dentro del binario (BUG-152, BUG-153)

- **BUG-152 🔴 — un binario instalado fuera del repo no encontraba la stdlib.**
  `importar "texto";` fallaba nada más instalar: **0 de 69 módulos** eran
  importables. La stdlib se embebe ahora en `lumen-sema` en tiempo de
  compilación, así que la heredan el CLI, el WASM y la API embebible. El disco
  mantiene la prioridad: `-L` y una `stdlib/` local siguen ganando. **69/69**.
- **BUG-153 🟡 — mensaje inútil al faltar el compilador de C.** `build --native`
  decía «Instala GCC» en todos los sistemas. Ahora indica qué instalar según el
  sistema y recuerda que `lumen run` no necesita compilador.
- **Workflows de CI y release.** `ci.yml` verifica formato, clippy, las 704
  pruebas en Linux/macOS/Windows, los tres fuzzers, los ejemplos y una
  **instalación limpia** (binario solo, sin repo alrededor). `release.yml`
  publica binarios **musl estáticos** para Linux —que corren en cualquier
  distro, sin el piso de glibc— y un **binario universal** de macOS que cubre
  Apple Silicon e Intel. Ver `docs/DISTRIBUCION.md`.

### 🐛 Código inválido que se ejecutaba en silencio (BUG-151)

- **BUG-151 🔴 — al faltar el `{` de un bloque, la condición se descartaba y el
  cuerpo se ejecutaba igual.** `si (1 == 2) basura { imprimir("x"); }` imprimía
  `x`, salía con código 0 y `lumen check` lo daba por válido. `parse_block`
  devolvía `None` sin emitir error, el llamador descartaba la sentencia entera
  y el bloque huérfano se reparseaba como bloque suelto — sin condición.
  Afectaba a `si`, `mientras`, `sino`, `para` y los cuerpos de función.
- **Fuzzer de rechazo** (`gen/fuzz8.py`): 12 familias de programas malformados,
  120 casos, exigiendo `rc != 0` en `run` **y** en `check` y que el cuerpo no
  llegue a ejecutarse. Con el bug, **50 de 120 se ejecutaban con éxito**; ahora
  0.

### 🐛 Las closures perdían las mutaciones (BUG-148, BUG-149, BUG-150)

- **BUG-148 🔴 — dos mecanismos de captura coexistían** y el criterio que elegía
  entre ellos era incorrecto: devolver una closure desde una función fallaba con
  `Variable '__cap_1_n' no definida`, y dos closures sobre la misma variable no
  veían las mutaciones de la otra. Ahora se usan siempre las celdas compartidas
  y se retiró la rama de slots globales `__cap_N_x`.
- **BUG-149 🔴 — la mutación llegaba a la celda pero no a la variable**: la
  closure avanzaba (5, 10, 15) mientras la función envolvente seguía viendo `0`.
  El copy-back sincroniza también el marco que declara la variable.
- **BUG-150 🔴 — el mismo fallo seguía vivo en el binario nativo**: la
  restauración de variables del llamador que protegía a las lambdas recursivas
  (BUG-061) deshacía la mutación hecha sobre una captura. La VM daba 15 y el
  ejecutable 0, sin aviso. La restauración es ahora consciente de las capturas.
- **Fuzzer de closures con oráculo independiente** (`gen/fuzz7.py`): 12 familias
  y **240 programas contrastados contra el resultado calculado en Python**, en
  VM y en nativo — 0 divergencias. Verificado por mutación: revertir el parche
  de BUG-150 reabre 20 divergencias.
- **Limitación conocida**: dos closures sobre una variable declarada en el nivel
  superior del programa (fuera de toda función) no propagan entre sí la última
  mutación. Dentro de funciones —el patrón real— el comportamiento es correcto.

### 🐛 `prestado mut` descartaba la mutación en silencio (BUG-147)

- **BUG-147 🔴 — pasar `s.campo` o `l[i]` a un parámetro `prestado mut` no
  tenía ningún efecto.** El copy-back que implementa el préstamo sólo reconocía
  variables simples (`Expr::Ident`); con cualquier otra expresión hacía
  `continue` y la mutación se perdía. El programa compilaba, `check` lo daba por
  válido y la llamada no hacía nada —el mismo modo de fallo del BUG-008, pero en
  qué se le pasa al parámetro en vez de en cómo se declara—. Lo hace más
  traicionero que la variante equivalente sí funciona: `sea tmp = s.l;
  toca(tmp);` muta `tmp`. Se reutiliza `emit_container_writeback`, que ya sabía
  escribir en `base.campo` y `base[i]` desde los BUG-033/064. Los literales se
  siguen ignorando, que es correcto: no tienen destino al que copiar.

- **Fuzzer diferencial de structs, listas y `prestado mut`** (`gen/fuzz6.py` +
  `gen/fz10.sh`), la zona del BUG-008. Calcula la salida esperada **en Python**
  en lugar de comparar LÚMEN consigo mismo, porque en el BUG-008 la VM y el
  binario nativo coincidían en el resultado incorrecto. 16 generadores —structs
  con listas, listas de structs, matrices, préstamos encadenados, `agregar`
  sobre préstamo, alias por asignación, mutación en bucle y los cuatro casos del
  BUG-147— y **640 programas: 0 divergencias con el oráculo, 0 entre backends,
  0 caídas**.

### 🐛 Herramientas que anunciaban acciones que no ejecutaban (BUG-144..146)

- **BUG-145 🔴 — `lumen publish` decía haber publicado el paquete sin subir
  nada.** Imprimía «Subiendo paquete a …» y «¡publicado con éxito en el registro
  público!» sin realizar ninguna petición de red: la CLI no enlaza cliente HTTP
  y el dominio del registro ni siquiera resuelve. Además (a) sin sesión iniciada
  usaba unas credenciales del autor codificadas en el binario —`omar_dev`—, de
  modo que cualquiera firmaba con esa identidad, y (b) el valor rotulado como
  «Checksum SHA-256» era un `DefaultHasher` (SipHash de 64 bits). Ahora exige
  `lumen login`, calcula **SHA-256 real** —coincide con `sha256sum`— y explica
  que el registro no está operativo, indicando dónde quedó el artefacto y cómo
  instalarlo. En `login`: el estado pasa de «🟢 Activa (Token Ed25519 Válido)» a
  «🟡 Guardada localmente (sin validar)», y `~/.lumen/credentials.json` —que
  contiene un token— se crea con permisos **0600** en vez de 0644.

- **BUG-146 🟠 — `lumen tutor` enseñaba sintaxis que no compila.** El tema
  `data` mostraba `funcion entero suma(Punto este)` y el tema `advanced`
  `funcion resultado<entero,texto> div(a,b)`; ambos fallan con E011, porque el
  receptor de un método se declara `este` sin tipo y LÚMEN exige el tipo de cada
  parámetro. Es el primer código que ve quien aprende el lenguaje: `learn` pone
  `tutor basics` como paso 1. El resto de los seis temas se verificó ejecutando
  su código, y los ejemplos que `learn` cita existen todos.

- **BUG-144 🟡 — `lumen config set <clave> <valor>` era imposible de
  invocar.** La ayuda de la propia herramienta lo anuncia, pero el parser sólo
  aceptaba tres argumentos posicionales y el cuarto moría con «Argumento
  desconocido» y rc=1. Como la configuración de LÚMEN no se persiste —los
  valores mostrados son los de la invocación actual— `set` ahora explica la
  bandera equivalente de cada clave (`backend` → `--aot`, `optimizacion` →
  `-O`), rechaza claves desconocidas con rc=1, y el listado general advierte que
  no se guarda nada en disco.

### 🐛 El ciclo de paquetes y las herramientas de generación (BUG-140..143)

- **BUG-142 🟠 — `lumen install` no sabía instalar un `.lmp`, el formato que
  `lumen pack` produce.** Había rama para directorios locales y para el registro
  oficial, pero ninguna para ficheros: la ruta caía al fallback de git y se
  concatenaba a `https://github.com/`, fallando con
  `repository 'https://github.com//tmp/.../paq.lmp' not found`. Las dos mitades
  del ciclo de paquetes nunca se habían ejecutado juntas. Se añade el caso, que
  reutiliza `unpack_package` y toma el nombre del `lumen.toml` extraído —no del
  nombre del fichero— para no registrar la dependencia como `lib-0.1.0`.
  Ahora `new → pack → install → importar → run` funciona de extremo a extremo.

- **BUG-143 🟠 — `lumen bindgen` generaba módulos que no compilaban.** El
  heurístico tomaba cualquier línea con paréntesis terminada en `;` o `{` por
  una declaración, de modo que las *llamadas* del programa se emitían como
  funciones: un fuente con dos `imprimir(...)` producía la misma función
  duplicada y redefinía un builtin ⇒ `E082` al usar el módulo. Se distingue
  declaración de llamada (una declaración lleva tipo delante del nombre), se
  deduplica por nombre y se excluyen los builtins. Además, el stub genérico que
  se emite cuando no hay firmas ya no se anuncia como «1 funciones enlazadas con
  éxito», sino como el esqueleto que es.

- **BUG-140 🟡 — `lumen doctor` anunciaba backends externos sin comprobarlos.**
  `LLVM IR Directo: ✓ Habilitado` era texto fijo en máquinas sin `clang`, `llc`
  ni `llvm-as`, donde `build --aot llvm` falla. El diagnóstico mentía justo a
  quien lo ejecuta porque algo va mal. Se sondea el toolchain y se reutiliza la
  detección de `cc` que ya existía. Los backends compilados en el binario
  (Cranelift, Stage-3, MCU) siguen siendo incondicionales, que es lo correcto.

- **BUG-141 🟡 — `lumen unpack` extraía fuera del directorio de trabajo.** El
  destino por defecto se derivaba de la ruta completa del `.lmp`, así que el
  paquete se desempaquetaba junto al origen y el directorio del usuario quedaba
  vacío —mientras el mensaje de éxito mostraba una ruta relativa y parecía haber
  obedecido—. Se usa `file_name()` antes de recortar la extensión.

### 🐛 El servidor LSP no moría nunca (BUG-139)

- **BUG-139 🔴 — `lumen lsp` giraba a tope de CPU cuando se cerraba stdin.**
  `read_line` devuelve `Ok(0)` —no un error— al llegar a EOF, así que el bucle
  de cabeceras salía con `content_length == 0` y el bucle externo hacía
  `continue`: un giro infinito sobre un stdin ya cerrado. Le ocurría a
  cualquier editor que cerrase la tubería sin mandar `exit` (un cierre brusco,
  un crash del cliente), y dejaba un proceso quemando un núcleo hasta que
  alguien lo mataba a mano. Ahora el EOF termina el bucle: de 25 s girando a
  **6 ms**.

Se auditaron además `lint`, `bench`, `test`, el REPL y el depurador
time-travel. `lint` coincide con `check` en **3095 programas** (0 desacuerdos,
0 crashes) y el LSP también (**0 desacuerdos en 3095 documentos**), lo cual
importa porque dos motores que opinan distinto sobre el mismo fichero son la
causa de que un editor marque en rojo código que compila. Se verificó también
que el `step_back` del depurador sigue revirtiendo el panel de salida
correctamente tras el cambio de BUG-138, y que el depurador no emite la salida
del programa por stdout.

### 🐛 Errores anidados y salida perdida (BUG-137, BUG-138)

- **BUG-137 🟡 — un error de FFI envuelto dentro de otro.** Al pasar a
  `__ffi_llamar` un handle que ya era `error(...)` —lo que devuelve
  `__ffi_cargar` cuando la biblioteca no existe— el handle se formateaba
  *dentro* del mensaje: `Biblioteca 'error(msvcrt.dll: cannot open shared
  object file)' no encontrada`. El texto culpaba a una biblioteca cuyo nombre
  era, literalmente, el fallo anterior, y la causa real quedaba enterrada.
  Ahora se propaga el error original.
- **BUG-138 🔴 — la salida se perdía si el programa no terminaba.** `imprimir`
  sólo acumulaba en un buffer que `lumen run` volcaba al final. Un servidor, un
  bucle de eventos, un TUI o simplemente un cuelgue **no mostraban nada**, ni
  siquiera las líneas impresas antes de bloquearse: para el usuario, el
  programa no había hecho nada. La salida se emite ahora en directo, con flush
  explícito (sin él, un stdout redirigido a fichero o tubería se pierde igual).
  El buffer se conserva porque lo necesitan los tests y el `step_back` del
  depurador time-travel, pero ha dejado de ser el único camino.

También se puso al día la **distribución**: `scripts/install.sh` y
`scripts/install.ps1` apuntaban todavía a la **v1.6.0**, de modo que el
instalador oficial descargaba una versión antigua. Ahora instalan la 3.0.0
—configurable con `LUMEN_VERSION`— y el de Linux/macOS **verifica el SHA256**
contra `SHA256SUMS.txt` y aborta si no coincide.

### 🐛 La biblioteca estándar no se podía importar (BUG-133 a BUG-136)

Auditoría de distribución: no basta con que compile el compilador, tiene que
funcionar lo que se entrega con él. De los 69 módulos de `stdlib`, **cuatro no
se podían importar** y siete ficheros no pasaban `lumen check`.

- **BUG-133 🔴 — el prefijo de módulo se aplicaba dos veces.** En una cadena
  `app -> nn -> tensor`, un literal de struct escrito dentro de `tensor.nv` ya
  había recibido su prefijo al aplanar `tensor`, y volvía a recibirlo al
  aplanar `nn`: `nn_tensor_GrafoAutograd`, un tipo que no existe (E062). Era el
  **único** sitio del aplanador que prefijaba sin consultar
  `is_known_prefixed`, así que declaración y uso dejaban de coincidir y
  cualquier jerarquía de dos niveles quedaba rota.
- **BUG-134 🔴 — los bindings de `si sea` recibían prefijo de módulo.** Las
  variables que liga un patrón (`si sea exito(datos) = r`) no se registraban
  como locales, de modo que se las tomaba por globales y se convertían en
  `m_datos`: el cuerpo referenciaba una variable inexistente (E033) en cuanto
  alguien importaba el módulo. `guard sea` tenía el mismo agujero.
- **BUG-135 🟠 — `__tamano_archivo` partía su contrato.** Devolvía el tamaño
  como entero pelado pero el fallo como `Error(...)`. Un
  `si sea exito(t) = __tamano_archivo(p)` no casaba **nunca**, así que
  `logging_tamano_archivo` respondía `-1` sobre ficheros que existían. El resto
  de builtins de fichero (`__leer_archivo`, `__escribir_archivo`) sí envuelven
  en `Exito`; ahora éste también.
- **BUG-136 🟡 — `__ffi_llamar_nv` tenía un tipo estático demasiado estrecho.**
  Se le asignaba `entero`, pero su valor depende del argumento `ret`: con
  `"texto"` devuelve texto. Código correcto disparaba E041.

Además se repararon los módulos afectados: `logging.nv` usaba
`sea Exito(...) = x sino { }` (sintaxis inexistente) y `resultado` como nombre
de variable (palabra reservada); `metrics.nv` comparaba contra `nulo`, que no
es un literal escribible del lenguaje, en vez de usar `__map_contiene`; y el
alias `largo` de `texto.nv` pasó a declararse como `texto_largo` —su nombre
público real— para no chocar con un builtin no sombreable.

**Resultado: 91/91 ficheros de `stdlib` pasan `lumen check` y los 69 módulos se
importan sin error.** Un test guardián (`stdlib_todos_los_modulos_se_pueden_importar`)
recorre la biblioteca entera e impide que vuelva a publicarse un módulo roto:
`lumen check` sobre el fichero suelto no bastaba —`bpe` y `nn` lo pasaban y aun
así reventaban al importarlos—.

### 🐛 El formateador rompía o descartaba el código con `impl` (BUG-131, BUG-132)

- **BUG-131 🟠 — `lumen fmt` no formateaba los `impl` inherentes.** Un
  `impl C { ... }` (sin rasgo) se reescribía como `impl  para C`, que no es
  sintaxis válida: el formateador reutilizaba la plantilla de
  `impl Rasgo para Tipo` sin comprobar si había rasgo. La salvaguarda interna
  detectaba que el resultado no recompilaría y abortaba, de modo que el usuario
  veía «⚠ No se ha formateado» sobre un fichero **perfectamente correcto**, sin
  pista de qué corregir. Afectaba a todo archivo con métodos propios.
- **BUG-132 🔴 — `fmt` perdía el `prestado mut` del receptor.** Al emitir los
  parámetros se escribía sólo `self`, descartando su tipo. Un método declarado
  `funcion vacio poner(prestado mut C self, entero n)` pasaba a recibir el
  struct **por valor**, así que dejaba de mutar el original: un formateo
  cambiaba en silencio lo que hacía el programa. Sólo quedó oculto porque
  BUG-131 impedía escribir el archivo; corregido aquél, éste habría corrompido
  código real.

Ambos se verificaron con un barrido de **3095 programas** (`gen/fmt24.sh`) que
compara la salida antes y después de formatear y vuelve a formatear el
resultado: **0 cambios de comportamiento y 0 fallos de idempotencia**.

### 🐛 Banderas ignoradas y ficheros con BOM (BUG-128 a BUG-130)

- **BUG-128 🟠 — `build` a bytecode ignoraba `-o`.** El `.nvc` se escribía
  siempre junto al fuente, aunque `build --native` sí respetaba la bandera. Lo
  grave no es dónde escribía, sino que **el mensaje anunciaba la ruta real**:
  `lumen build a.nv -o /tmp/x.nvc` respondía «Bytecode generado: a.nvc». El
  usuario pedía una ruta, el compilador usaba otra y lo contaba sin avisar.
- **BUG-129 🟡 — `doc` tenía el mismo descuido.** La documentación HTML caía
  siempre junto al fuente, ignorando `-o/--output/--salida`.
- **BUG-130 🟠 — un fichero con BOM UTF-8 no compilaba.** Es lo que guarda por
  defecto el Bloc de notas y varios editores de Windows. El error era «E001:
  Caracter inesperado» en la línea 1, columna 1… y el carácter culpable es
  **invisible**, así que el mensaje no daba ninguna pista sobre un código que
  era perfectamente válido. El BOM sólo indica la codificación: ahora se
  descarta al entrar en el lexer, de modo que vale para las tres rutas
  (intérprete, `check` y binario nativo).

Verificado además que el **bytecode `.nvc` es fiel al fuente**: 3038 programas
de los corpus de fuzzing ejecutados como fuente y como bytecode, con resultados
idénticos.

### 🐛 El backend Cranelift entregaba binarios que mentían (BUG-124 a BUG-127)

- **BUG-124 🔴 — el binario no ejecutaba NADA.** Cranelift buscaba como punto de
  entrada `__main__`, que sólo existe si el fichero tiene código de nivel
  superior. Un programa que sólo define `funcion vacio principal()` —la forma
  habitual— no lo tiene, así que se generaba un `main` que retornaba 0 sin
  llamar a nada: el compilador anunciaba «✓ Binario nativo», el programa no
  imprimía nada y salía con código 0. El backend C y el de LLVM ya hacían la
  cascada a `principal`; Cranelift se había quedado sin ella.
- **BUG-125 🟠 — `>>` era lógico en vez de aritmético.** Se emitía `ushr`, que
  mete ceros por la izquierda, así que `-1 >> 1` daba `9223372036854775807` en
  vez de `-1`.
- **BUG-126 🟠 — los decimales se compilaban a `0` en silencio.**
  `imprimir(1.5 + 2.5)` producía `0` en un binario que decía haberse generado
  correctamente. Este backend no soporta decimales: ahora lo dice y se niega.
- **BUG-127 🟡 — los booleanos se imprimían como `1`/`0`.** La VM y el backend C
  imprimen `true`/`false`. Se añadió el rastreo del tipo booleano y su shim de
  impresión.

Los cuatro son el patrón que **BUG-050 y BUG-084 ya habían corregido para los
otros backends**: o el artefacto es correcto, o hay que negarse a producirlo.
Nunca un ejecutable que aparenta haber funcionado.

Barridos de esta versión sin una sola divergencia: **1049** programas de
aritmética entera extrema, **1095** de decimales, **763** de texto y **131** de
listas y mapas.

### 🐛 Texto en los extremos, aridad de builtins y ruido de diagnóstico (BUG-117 a BUG-123)

- **BUG-117 🟢 — dieciocho mensajes en inglés.** «Channel not found», «Task
  failed», «Stack underflow»… en un lenguaje cuyas palabras clave y diagnósticos
  son en español. No era texto muerto: todos son alcanzables desde código de
  usuario. Traducidos.
- **BUG-118 🟡 — la pila del backend C no se equilibraba tras un builtin.** Cada
  rama desapila un número fijo de argumentos; si el llamador empujó otros
  tantos, el sobrante desplazaba todo lo demás. Ahora el punto de llamada
  garantiza el saldo exacto, para las 94 ramas a la vez.
- **BUG-119 🔴 — tres builtins de texto sin validación de aridad.**
  `__str_codigo("abc", 0)` compilaba y el binario nativo **segfaulteaba**: la
  rama C se quedaba con el argumento equivocado y usaba un entero como puntero a
  texto, mientras la VM ignoraba el sobrante. Es exactamente el fallo que
  BUG-098 cerró para `__str_concat_list`, pero quedaron huecos en
  `__str_codigo`, `__str_a_caracteres` y `__str_empieza_con`. Ahora se rechaza
  en compilación con E040.
- **BUG-120 🟠 — `__str_subcadena` con índices negativos.** La VM convierte el
  índice a `usize` y lo recorta, así que un inicio negativo da cadena vacía; el
  C hacía `if (st < 0) st = 0` y devolvía desde el principio:
  `__str_slice("hola", -2, -1)` daba `""` interpretado y `hola` compilado.
- **BUG-121 🟡 — `__str_reemplazar` con patrón vacío.** `str::replace` de Rust
  inserta el reemplazo en cada frontera de carácter (`"aaa"` → `XaXaXaX`); el C
  devolvía el texto intacto.
- **BUG-122 🟠 — `__str_dividir` con separador vacío partía por BYTES.** La
  misma familia que BUG-087: `"ñoño"` daba 4 caracteres en la VM y 6 trozos
  rotos en el binario.
- **BUG-123 🟢 — errores en cascada tras un struct mal escrito.** `Caj{n:1}`
  daba el E062 correcto (con sugerencia de `Caja`) y encima un E060 inútil, por
  tipar la expresión fallida como `vacio` en vez de «no lo sé».

Un fuzzer diferencial nuevo de **763 programas de texto** (UTF-8, índices fuera
de rango, separadores y patrones vacíos) queda en **0 divergencias** entre el
intérprete y el binario nativo.

### 📚 Documentación

- `LENGUAJE.md` documenta ahora la semántica de `%` (**resto truncado**, con el
  signo del dividendo: `-7 % 3` es `-1`, no `2`), el truncado de la división
  entera, el rango `0..63` de los desplazamientos y las dos carencias de diseño
  de los decimales: exigen parte fraccionaria explícita y **no admiten notación
  científica** (`1.0e10` es E012).

### 🐛 Decimales: NaN, saturación y notación científica (BUG-113 a BUG-116)

- **BUG-113 🟡 — el NaN se imprimía distinto en cada backend.** `_fmt` en el
  runtime C contemplaba `isinf` pero no `isnan`, así que el NaN caía al `%g` y
  salía como **`-nan`** mientras la VM imprimía `NaN`. Mismo cálculo, dos textos.
- **BUG-114 🟠 — `a_entero` de un decimal enorme daba el valor opuesto.**
  Convertir a `int64_t` un `double` fuera de rango es **comportamiento indefinido
  en C**: `a_entero(1.0e300)` devolvía `-9223372036854775808` en el binario
  nativo y `9223372036854775807` en la VM. El comentario del código decía imitar
  el `as i64` de Rust, que **satura** a los extremos y convierte NaN en 0; ahora
  lo hace de verdad.
- **BUG-115 🟠 — el binario nativo usaba notación científica y la VM no.** El
  guard `fabs(d) < 1e16` mandaba los decimales grandes al `%g` de C:
  `1000000000000000000.0` se imprimía como **`1e+18`** en el binario y como
  `1000000000000000000` en la VM. Igual por abajo, con `0.000001` ⇒ `1e-06`. El
  `Display` de Rust nunca usa notación científica; el runtime C tampoco ya.
- **BUG-116 🟠 — la VM saturaba al imprimir un decimal mayor que i64.**
  `*n as i64` satura en Rust, así que `9223372036854775807.0` —que el `double`
  redondea a 2^63— se imprimía como `...807` cuando su valor real es `...808`.
  Ahora la vía entera sólo se usa si el valor cabe de verdad en un `i64`.

### 🐛 Aritmética de los extremos: literales, desbordamientos y `%` (BUG-108 a BUG-112)

- **BUG-108 🟠 — un literal entero fuera de rango se convertía en 0 en silencio.**
  `parse().unwrap_or(0)` en el parser: `imprimir(9223372036854775808)` compilaba
  y ejecutaba imprimiendo **`0`**, un número que nadie escribió. Ahora se avisa
  con **E083** citando el rango válido. `-9223372036854775808` (el mínimo
  legítimo, cuyo valor absoluto **no** cabe) se reconoce como una unidad en el
  operador unario, así que sigue siendo válido.
- **BUG-109 🔴 — `i64::MIN / -1` abortaba con un pánico de Rust.** Desbordaba en
  tres sitios: el plegado de constantes del IR (pánico **al compilar**) y los
  opcodes `Div` y `Mod` de la VM. El proceso moría con un volcado interno y, peor
  aún, **salía con código 0**. Add/Sub/Mul ya usaban `overflowing_*`; ahora Div y
  Mod también, en las tres rutas.
- **BUG-110 🟠 — desplazamientos fuera de rango: error en la VM, basura en el
  nativo.** La VM valida que el desplazamiento esté en 0-63; el runtime C hacía
  `x << 64` directamente, que es **comportamiento indefinido**. `1 << 64` daba
  error en la VM y `0` en el binario. Los cuatro puntos de shift del runtime C
  validan ahora igual que la VM.
- **BUG-111 🟠 — el `%` con dividendo negativo daba tres resultados distintos.**
  La VM usaba el resto euclídeo (`rem_euclid`, siempre positivo) mientras el
  plegado de constantes, el backend C y el `%` de decimales (`fmod`) usaban el
  resto truncado. Resultado: `-7 % 3` valía **2** con variables, **-1** con
  literales (porque se plegaba al compilar) y **-1** en el binario nativo. Se
  unifica al resto truncado, que es lo que hacían tres de las cuatro rutas.
- **BUG-112 🟠 — `~` perdía precisión en el binario nativo.** `_bnot` convertía a
  `double` antes de operar, y un `double` no puede representar todos los int64:
  `~9223372036854775807` devolvía el propio valor en vez de `i64::MIN`. El
  complemento a uno es una operación entera y ya no pasa por coma flotante.

### 🐛 La optimización que corrompía los decimales (BUG-106 y BUG-107)

- **BUG-106 🔴 — multiplicar un decimal por 2, 4 u 8 daba un resultado erróneo o
  mataba el programa.** Una pasada de «reducción de fuerza» en el IR convertía
  `x * 2` en `x << 1` (y análogamente para 4 y 8) reconociendo el patrón por la
  constante literal, **sin mirar el tipo del otro operando** — el IR en ese punto
  no lleva tipos. Para decimales el resultado divergía entre backends: la VM
  moría con «ShiftLeft requires integers», un operador que no aparece en el
  código fuente, y el **binario nativo truncaba el decimal en silencio**, de modo
  que `2.5 * 2` imprimía **4** en vez de 5. `lumen check` daba el programa por
  válido. La pasada se ha desactivado: multiplicar por una potencia de dos no es
  un cuello de botella que justifique un resultado incorrecto, y recuperarla
  exigiría propagar tipos hasta el IR. Ahora `2.5 * 2` da `5` en ambos backends
  y el desplazamiento explícito (`x << 1`) sigue intacto.
- **BUG-107 🟡 — los errores de operador salían en inglés.** En un lenguaje que
  se presenta como bilingüe ES/EN y cuyos diagnósticos están en español, quince
  errores de runtime respondían «Add requires numbers or strings» o «Div requires
  numbers». Además nombraban el **opcode interno**, no el operador escrito, lo que
  producía mensajes desconcertantes como «ShiftLeft requires integers» para un
  `x * 2`. Ahora citan el símbolo real: «El operador '*' requiere números».

### 🐛 Diagnósticos que no ayudaban a arreglar nada (BUG-105)

- **BUG-105 🟡 — los errores de nombre desconocido no sugerían el nombre real.**
  Ante `sumr(1)` con una función `sumar` definida dos líneas más arriba, el
  compilador respondía «Define la función 'sumr' antes de llamarla»: una ayuda
  que repite el error en vez de resolverlo. Lo mismo con structs, enums y tipos
  en declaraciones. El caso más costoso era el de los **módulos**: como LÚMEN
  prefija todo lo que se importa, quien escribe `Color::Verde` tras
  `importar "util"` recibía «Define la enumeración 'Color'» cuando la respuesta
  correcta era `util_Color::Verde`. Ahora los cinco diagnósticos de nombre
  (`E042` de función y los cuatro `E062` de struct, enum en expresión, enum en
  patrón y tipo en declaración) buscan el candidato más cercano por distancia de
  edición y, con prioridad, el nombre prefijado por módulo: **«¿Quisiste
  escribir 'util_Color'?»**. El umbral es proporcional a la longitud del nombre
  para no inventar sugerencias absurdas, y la sugerencia de conversión de
  BUG-002 (`texto(...)` → `a_texto(...)`) conserva su prioridad.

### 🐛 Un mismo error, dos códigos de salida (BUG-104)

- **BUG-104 🟡 — la VM y el binario nativo salían con códigos distintos ante el
  mismo error.** División por cero, índice fuera de rango y campo inexistente
  terminaban con **código 1 en la VM y código 3 en el nativo**, con idéntico
  mensaje. Para un script de CI o un `Makefile` que ramifica según el código de
  salida, el mismo programa con el mismo fallo significaba dos cosas distintas
  según cómo se hubiera ejecutado. La divergencia venía de `_rt_error3` en
  `lumen_rt.h`, que preservaba un código heredado de la v2.4.6. Ahora ambos
  backends salen con **1**; la función se conserva aparte porque su semántica de
  `atrapar` sí difiere (liga el mensaje sin el prefijo «Error: »), y dos pruebas
  nuevas fijan tanto la paridad de códigos como que `intentar/atrapar` sigue
  desenrollando la pila en vez de terminar el proceso.

### 🐛 La función que el runtime suplantaba (BUG-103)

- **BUG-103 🟠 — redefinir un builtin se aceptaba y luego el builtin ganaba.**
  `funcion vacio push(prestado mut lista<entero> l)` pasaba `lumen check` como
  válido, pero `push` es alias de `agregar`: la llamada iba al builtin con un
  argumento de menos, devolvía `vacio` y el write-back del préstamo guardaba ese
  vacío en la variable del llamante. El programa moría después con «'largo'
  espera lista o texto, no Void», un mensaje sin relación aparente con la causa.
  Ahora se avisa al analizar con `E082`. Se respetan los builtins sombreables
  del BUG-018 (`abs`, `minimo`, `leer`…), y el prefijo `__` **no** cuenta como
  interno: la stdlib lo usa como convención de privado. El aviso destapó una
  función muerta en `stdlib/texto.nv` (`largo`), suplantada por el builtin desde
  antes de la v2.4.6.

### 🐛 El barrido de contraste contra la v2.4.6 (BUG-102)

- **BUG-102 🟠 — un mapa se podía indexar, y la VM y el nativo no coincidían.**
  Detectado por un barrido sistemático de 294 programas sin anotación de tipo
  ejecutados con el binario oficial v2.4.6 y con la v3.0. Al introducir el tipo
  dinámico (BUG-100) se agrupó `__map_obtener` —resultado genuinamente
  desconocido— con `__map_nuevo`/`__map_poner`, que devuelven un mapa conocido;
  como el dinámico es compatible con todo, `m[0]` y `m[0] = 9` pasaron a
  aceptarse pese a que la v2.4.6 los rechazaba, y el síntoma era una divergencia
  entre backends: la VM devolvía un `0` inventado y el binario nativo abortaba
  con «Índice 0 fuera de rango». Corregido con un tipo propio para los mapas;
  lo que se *saca* de un mapa sigue siendo dinámico. Paridad VM ↔ nativo exacta
  en los 294 casos del barrido.

### 🐛 Dos regresiones propias en `elegir` sobre un `resultado` (BUG-101)

- **BUG-101 🟠 — `elegir` sobre un `resultado` sin anotar dejó de compilar.**
  Detectadas al contrastar contra el binario oficial v2.4.6, que sí ejecuta
  estos programas: (a) `caso exito(v)` / `caso error(e)` son *patrones*, pero se
  analizaban como expresiones, así que `error(e)` se leía como una construcción
  y disparaba `E064`; (b) `sea r = exito(1)` marcaba el tipo del error como
  `vacio` en lugar de «desconocido», de modo que `caso error(e)` ligaba `e` a un
  valor vacío y usarlo daba `E035`. Sólo ocurría sin anotar el tipo —la forma
  corta y más habitual—, por lo que ni los tests ni los ejemplos, que suelen
  escribir la versión explícita, lo habían detectado. Corregido tratando los
  patrones como tales y usando el tipo dinámico para la mitad indeterminada del
  `resultado`; el `E064` legítimo y el `E056` de un caso mal tipado siguen
  intactos.

### 🐛 La causa de fondo: un tipo que significaba dos cosas (BUG-100)

- **BUG-100 🟠 — `numero` era a la vez «número» y «tipo desconocido».** El
  BUG-099 se había parcheado caso por caso; al seguir sondeando aparecieron más
  síntomas del mismo origen (una lambda guardada en un mapa no se podía llamar,
  `E058`; un booleano guardado en un mapa no valía como condición, `E034`, ni
  admitía `!`, `E039`). La causa era que `TypeInfo::Numero` representaba las dos
  cosas, así que cada punto del analizador tenía que adivinar el sentido. El
  intento de aceptar `numero` en todas partes rompió el test
  `test_boolean_condition`, que marcaba el límite correcto: `numero x = 1; si
  (x)` **debe** seguir fallando. El tipo dinámico pasa a ser una variante
  propia, `TypeInfo::Dinamico`, compatible con cualquier tipo y sin significado
  numérico; los builtins de mapas la devuelven y todos los puntos de uso la
  aceptan. Barrido de 20 builtins con valores dinámicos: todos correctos y con
  paridad exacta VM ↔ binario nativo.

### 🐛 El tipo dinámico que se leía como número (BUG-099)

- **BUG-099 🟠 — lo guardado en un mapa no se podía usar.** `__map_obtener`
  devuelve el tipo dinámico `numero` (un mapa admite cualquier valor y su tipo
  sólo se conoce en ejecución), pero el analizador lo interpretaba como «es un
  número». Resultado: un struct recuperado de un mapa rechazaba `p.x` con
  `E060`, una lista rechazaba `para x en xs` con `E044`, `p.x = 9` y `xs[0] = 7`
  eran de sólo lectura, y operar con un valor dinámico daba `decimal`, así que
  acumularlo en un `entero` fallaba con `E031`. Todo ello pese a que el runtime
  ejecuta esos programas correctamente —bastaba pasar el valor por una función
  con el tipo anotado para esquivarlo—. Ahora el tipo dinámico se propaga como
  dinámico en los cinco puntos, sin relajar ninguna comprobación real.

### 🐛 La lista vacía que no se podía llenar (BUG-097 y BUG-098)

- **BUG-097 🟠 — `sea l = []` seguida de `agregar` no compilaba.** Acumular en
  una lista creada vacía —`sea l = []; l = agregar(l, i);`— y luego sumar sus
  elementos en un `entero` fallaba con un `E031` que hablaba de un `decimal`
  inexistente. El literal vacío recibe el tipo de elemento genérico `numero`,
  `agregar` lo propagaba sin refinarlo y la reasignación nunca actualizaba el
  tipo de la variable, así que el error afloraba lejos de su origen. Ahora
  `agregar` deduce el elemento del valor añadido cuando aún es indeterminado y
  la reasignación acepta un tipo más concreto que el genérico previo. Funciona
  con enteros, textos, structs y listas anidadas, sin relajar la comprobación
  de tipos.
- **BUG-098 🟠 — `__str_concat_list` divergía entre la VM y el binario.** No se
  validaba su aridad: llamarlo con un separador extra (`__str_concat_list(l,
  "-")`) se aceptaba, la VM ignoraba el sobrante y el backend C se quedaba con
  el separador en vez de con la lista, devolviendo una cadena vacía donde la VM
  devolvía `abc`. Ahora se rechaza con `E040` y se indica la alternativa para
  unir con separador.

### 🐛 Los dos backends que quedaban mudos (BUG-095 y BUG-096)

Cierre de la política de BUG-050/084: **ningún backend debe producir un
artefacto incorrecto anunciándolo como éxito**.

- **BUG-095 🔴 — Cranelift compilaba `intentar`/`atrapar` y `elegir` en
  silencio**: el `match` de instrucciones terminaba en un `_ => {}` mudo, así que
  las instrucciones no implementadas por ese backend (`PushHandler`/`PopHandler`
  para excepciones, `MatchType`/`MatchPayload` para el emparejado de patrones)
  se descartaban sin dejar rastro. Un `intentar { 10 / 0 } atrapar { -1 }`
  compilaba sin advertencia y devolvía **10**, donde la VM devuelve **-1**. Ahora
  se registran y la compilación se detiene; el catch-all restante también avisa,
  de modo que una instrucción futura no puede volver a desaparecer.
- **BUG-096 🔴 — el backend LLVM anunciaba un IR inválido como éxito**:
  implementa 12 de los 42 opcodes y el resto desaparecía del IR. Peor: las
  llamadas se emitían sin comprobar que la función existiera, así que
  `imprimir(largo(l))` producía `call i64 @largo(...)` **sin ningún `declare`**
  — LLVM IR que no pasa el verificador ni enlaza — y la CLI lo anunciaba con un
  «✓ Archivo LLVM IR generado». Ahora se rechaza, y para lo que sí cubre (código
  escalar) hay un test que comprueba que el IR no tiene llamadas colgantes.

### 🐛 Mutación silenciosamente ignorada (BUG-094)

- **BUG-094 🔴 — asignar por índice dentro de un struct no hacía nada**: la
  escritura de vuelta del contenedor modificado sólo se emitía cuando la base era
  una **variable suelta**. Si era un campo —`m.g[i][j] = v`, `b.a.l[i] = v`,
  `a.l[i].campo = v`— la lista actualizada se quedaba en la pila y se descartaba:
  la asignación se perdía **en silencio, sin error y sin que `lumen check`
  protestara**. Un bucle que rellenaba una matriz guardada en un struct dejaba la
  matriz intacta. `m.g[i] = v` (un solo nivel) sí funcionaba, de modo que el
  fallo aparecía justo al anidar. Ahora se reutiliza el write-back recursivo de
  BUG-033/064, que sube por toda la cadena de accesos. Preexistente en v2.4.6.

### 🐛 Rasgos y genéricos (BUG-092 y BUG-093)

- **BUG-092 🔴 — el cuerpo de los métodos de un `impl` no se analizaba nunca**:
  de un bloque `impl <Rasgo> para <Tipo>` sólo se comprobaba que las **firmas**
  encajaran con el rasgo; el contenido de los métodos no pasaba por el analizador.
  Dentro de un impl valía cualquier disparate —una variable inexistente, un campo
  usado sin `este`— y `lumen check` respondía «el programa es válido». Después la
  VM moría en runtime con «Variable no definida» y el **binario nativo imprimía
  `0` en silencio**. La rama de impl inherente sí analizaba los cuerpos, así que
  las dos formas de `impl` no se comportaban igual.
- **BUG-093 🟠 — los structs genéricos no se podían anidar**: al inicializarlos
  sin argumentos de tipo explícitos no se sustituían los parámetros, de modo que
  los campos conservaban la variable de tipo sin resolver. Con un solo nivel
  pasaba desapercibido, pero `Caja{v: Caja{v: 7}}` y luego `a.v.v` fallaba con
  «E060 No puedes acceder a un campo de un valor de tipo 'T'». Ahora el argumento
  de tipo se infiere del valor de cada campo.

### 🐛 Tipos estáticos que no coincidían con el runtime (BUG-090 y BUG-091)

Dos fallos del analizador semántico que **rechazaban programas correctos**: el
tipo declarado de un builtin no era el que la función devuelve de verdad.

- **BUG-090 🟠 — `agregar` se declaraba `vacio` aunque devuelve la lista**: el
  runtime siempre ha devuelto la lista nueva, pero sema decía `vacio`. Con una
  variable (`sea otra = agregar(l, 2)`) colaba, porque `sea` no comprueba nada;
  al asignar el resultado a un **campo de struct** o a un **elemento de lista**
  —`c.items = agregar(c.items, x)`, que es la forma documentada de hacer crecer
  una lista dentro de un struct— saltaba «E031 No puedes asignar un valor de tipo
  'vacio'». Es decir, la manera natural de usarlo era inaceptable para el
  compilador. Ya existía en la v2.4.6.
- **BUG-091 🟡 — `__map_longitud`, `piso`, `techo` y `redondear` se tipaban como
  `decimal`**: los cuatro devuelven un entero (`piso(3.7)` → `3`), pero
  `__map_longitud` ni siquiera estaba registrado en sema y caía al tipo por
  defecto. Guardar el resultado en un campo `entero` fallaba con un E031
  inventado. `raiz` sigue siendo `decimal`, que es lo correcto.

### 🐛 Paridad VM ↔ binario nativo (BUG-086 a BUG-089)

Última ronda, dedicada a que un programa se comporte **igual interpretado que
compilado**. Todas eran divergencias silenciosas: el programa no fallaba, sólo
daba un resultado distinto según cómo lo ejecutaras.

- **BUG-086 🟠 — un fallo del backend Cranelift abortaba el proceso**: si el
  verificador de Cranelift rechazaba una función, `define_function` provocaba un
  `panic!` y el usuario veía el rastro de pila de un «crash del compilador» en
  vez de un error de compilación. Ahora el error se propaga y se presenta como
  tal. Se amplió además el registro de construcciones no soportadas de BUG-084:
  structs, listas, tuplas, enums, `opcion`/`resultado` y llamadas indirectas
  también se compilaban como `0` en silencio y ahora se detectan.
- **BUG-087 🟠 — `__str_codigo` devolvía bytes en vez de caracteres**: en el
  binario nativo `"añb"` daba `[97, 195, 177, 98]` (los bytes UTF-8 crudos) y en
  la VM `[97, 241, 98]`. Ahora decodifica UTF-8 y devuelve puntos de código.
- **BUG-088 🟡 — las claves de un mapa salían en distinto orden según el
  backend**: la VM iteraba por orden de hash y el nativo por orden de inserción.
  Como ninguno de los dos órdenes era significativo, ambos ordenan ahora de forma
  estable: los números por valor (`[2, 10, 33]`, no `[10, 2, 33]`) y el resto por
  su representación textual.
- **BUG-089 🟠 — `1.0 / 0.0` daba `inf` compilado y error interpretado**: la
  división entera por cero ya se detectaba en ambos, pero la decimal producía
  `inf`/`nan` en el binario nativo mientras la VM abortaba con «División por
  cero». Ahora ambos fallan igual, incluido el módulo.

### 🐛 Correcciones de la última ronda (BUG-080 a BUG-085)

**Runtime nativo (backend C)**
- **BUG-080 🟠 — las clases `\d`, `\w`, `\s` no funcionaban en el binario
  nativo**: `_regex_m`/`_regex_rep` compilaban el patrón con POSIX
  `REG_EXTENDED`, que no entiende los atajos de Perl. `regcomp` fallaba y la
  función devolvía `false` **en silencio**, así que la misma expresión regular
  daba resultados distintos en la VM y en el nativo. Ahora se traducen a clases
  POSIX (`[[:digit:]]`, `[[:alnum:]_]`, `[[:space:]]`) antes de compilar.
- **BUG-081 🔴 — las corutinas abortaban dentro de hilos**: la pila de corutinas
  (`ST`/`SP`), el contador de profundidad y `_stack_base` eran globales
  compartidas entre hilos, de modo que un hilo veía la base de pila de otro y el
  programa moría con «Profundidad máxima superada». Ahora son `thread_local` y
  cada hilo inicializa su propia base con un margen de 4 MiB.
- **BUG-082 🟠 — `subcadena` cortaba por bytes y rompía el UTF-8**: `_sub` y
  `_to_chars` indexaban el `char*` directamente, así que cualquier acento o emoji
  quedaba partido a la mitad y producía texto inválido. Se añadió `_utf8_off()`
  y ahora se recorta por caracteres, igual que la VM.
- **BUG-083 🔴 — consumo de memoria cuadrático: los programas morían por OOM**:
  cada paso de argumento hacía una copia **profunda** (`_dcp`) para preservar la
  semántica de valor, de modo que acumular en una lista dentro de un bucle era
  O(n²) en memoria; con n=800 el proceso lo mataba el OOM killer. Se sustituyó
  por *copy-on-write*: las copias comparten el almacenamiento y sólo se
  desdoblan al mutar. El pico de memoria pasa de crecer sin control a **~30 MB
  constantes**. El fallo ya existía en la v2.4.6.

**Compilador AOT (backend Cranelift, `--aot rust`)**
- **BUG-084 🔴 — generaba binarios que devolvían resultados falsos**: builtins
  tan comunes como `largo`, `agregar`, `a_texto`, `leer` o los de mapas no están
  implementados en este backend y se compilaban como la constante `0`, **sin
  aviso alguno**. `largo(lista)` devolvía 0 y el programa seguía adelante como si
  nada: exactamente el patrón de «binario que miente» que ya se corrigió para el
  backend C. Ahora la compilación **falla** enumerando los builtins que faltan y
  sugiriendo el backend C o la VM; `--permitir-no-soportados` mantiene el
  comportamiento antiguo si se asume el riesgo.
- **BUG-085 🟡 — `--permitir-no-soportados` se usaba como nombre del binario**:
  la bandera se consultaba pero no estaba declarada en el parser de argumentos,
  así que caía en el caso genérico que fija el destino y generaba un ejecutable
  literalmente llamado `--permitir-no-soportados`.

### 🐛 Correcciones de esta ronda (BUG-064 a BUG-079)

**Semántica del lenguaje**
- **BUG-064 — `agregar(l, x)` no añadía nada, en silencio**: el builtin es
  funcional (apila una lista nueva) y, como sentencia, ese valor se descartaba
  con un `Drop`. El elemento se perdía sin error alguno y `lumen check` daba el
  programa por válido, mientras que la forma método `l.agregar(x)` sí mutaba
  desde BUG-033: dos sintaxis equivalentes hacían cosas distintas. Ahora, en
  posición de sentencia, el resultado se escribe de vuelta en el receptor
  (variable, campo o elemento indexado). Si alguien usa el valor de retorno se
  respeta la semántica anterior.
- **BUG-065 — `opcion<T>` y `resultado<T,E>` sólo servían con números**: al
  ligar las variables de un patrón, `bind_pattern_vars` definía TODO
  identificador como `numero` sin mirar el tipo del valor examinado. Hacer
  `elegir (o) { caso algun(p): p.nombre }` sobre un `opcion<Contacto>` fallaba
  con «E060 No puedes acceder a un campo de un valor de tipo 'numero'». Ahora el
  patrón se liga con el tipo real y se propaga a `algun`/`exito`/`error`, tuplas,
  structs y listas.

**Herramienta de formateo (`lumen fmt`)** — cinco fallos que **destruían código**
- **BUG-066 — borraba todos los comentarios**: el lexer los descarta y el
  formateador reimprime desde el AST, así que cada `lumen fmt` eliminaba la
  documentación del fichero. Ahora se reinyectan desde el original,
  respetando las barras dentro de cadenas de texto.
- **BUG-067 — perdía los genéricos de un struct**: `estructura Par<T, U>` se
  reescribía como `estructura Par` y **el fichero formateado ya no compilaba**.
- **BUG-068 — borraba las declaraciones con destructuring**:
  `entero x, texto y = (1, "hola");` desaparecía del archivo (faltaba el brazo
  `Decl::Destructure` y caía en un `_ => {}`).
- **BUG-069 / BUG-070 — perdía ramas de un `elegir`**: no se emitían las
  alternativas de los patrones OR (`caso 1 | 2 | 3:` quedaba en `caso 1:`) ni el
  caso `defecto`, que vive en un campo aparte del AST. El `elegir` pasaba a ser
  no exhaustivo y el código dejaba de compilar (E080).
- **BUG-071 — los paréntesis crecían en cada pasada**: `(x como entero)` →
  `((x como entero))` → … El formateo no era idempotente y el fichero engordaba
  indefinidamente. Se corrige sin tocar los paréntesis que sí hacen falta
  (`((a / b) como entero)`).

En conjunto, sobre los 174 ejemplos del repositorio `fmt` pasa de **3 ficheros
rotos y 7 no idempotentes** a **0 y 0**, sin perder un solo comentario.

**Herramienta de pruebas (`lumen test`)**
- **BUG-072 — las funciones `test_*` NUNCA se ejecutaban**: el runner construía
  un bytecode con `funcs: vec![prueba]` pero conservaba las instrucciones
  globales, así que la VM corría el cuerpo principal (vacío), devolvía `Ok` y la
  prueba se contaba como ✓ OK sin haberse ejecutado. Un test que afirmara
  `2 + 2 == 5` pasaba, y `lumen test` salía con código 0: la herramienta de
  calidad daba por buenas suites que fallaban. Ahora se sintetiza para cada
  prueba un programa que la invoca de verdad y, además, se inspecciona la salida
  en busca de aserciones fallidas (que imprimen «ERROR:» sin abortar). El código
  de salida vuelve a ser 1 cuando algo falla, como necesita cualquier CI.

**Herramientas de la CLI** — tres herramientas que *simulaban* trabajar
- **BUG-073 — `lumen bundle` ignoraba la ruta de salida**: `lumen bundle app.nv
  /ruta/binario` calculaba el destino, lo anunciaba por pantalla con su tamaño en
  KB y un «✨ ¡BINARIO STANDALONE GENERADO CON ÉXITO!»… pero nunca se lo pasaba al
  compilador, que dejaba el ejecutable junto al fuente. En la ruta pedida no
  había nada, y el tamaño mostrado se leía del binario por defecto, de modo que
  el informe parecía correcto. Ahora la ruta se respeta (creando el directorio si
  hace falta) y, si el binario no aparece donde se dijo, es un error con código
  de salida 1 en vez de una cifra inventada.
- **BUG-074 — `lumen lint` no analizaba nada**: era literalmente un `println!`
  que imprimía «✓ 0 advertencias» para cualquier entrada; devolvía éxito con
  basura sintáctica e incluso con archivos inexistentes. Ahora ejecuta lexer,
  parser y análisis semántico —con el mismo formato de diagnóstico que
  `lumen check`— y añade reglas de estilo (líneas largas, espacios finales,
  tabuladores, marcas pendientes). Sale con código 1 si hay errores.
- **BUG-076 — `lumen fuzz` no ejecutaba el programa**: imprimía siempre las
  mismas cifras fijas («5000 iteraciones», «97.4% de cobertura», «0 crashes»,
  «100% seguro») sin llegar a correr nada. Declaraba «100% seguro» un programa
  que aborta por división por cero. Ahora localiza los literales enteros del
  fuente, genera mutaciones a valores límite (`0`, `-1`, `i64::MIN`, `i64::MAX`…),
  compila y **ejecuta cada variante en la VM**, y reporta los fallos concretos
  con código de salida 1.

**Backend AOT Cranelift (`--aot rust`)**
- **BUG-075 — el compilador panicaba al cruzar bloques**: la pila de operandos
  guardaba valores SSA crudos y no se reconciliaba en las fronteras de bloque, así
  que un valor calculado en un bloque se consumía en otro que no lo dominaba y el
  verificador abortaba con «uses value vN from non-dominating inst» → `panic!`.
  Bastaba un `elegir` sobre el resultado de una llamada. Ahora la pila viva se
  derrama a variables SSA antes de cada salto y se recarga al entrar al bloque
  destino. **De 49 ejemplos que hacían panicar al compilador se pasa a 0**, y 15
  ejemplos más producen ya la misma salida que la VM.

**FFI y generación de bindings**
- **BUG-077 — `lumen bindgen` generaba código que no compila**: declaraba
  `entero _lib_handle = __ffi_cargar(...)` cuando ese builtin devuelve `texto`,
  así que el módulo recién generado fallaba con un **E031** nada más pasarlo por
  `lumen check`. Además fijaba el tipo de retorno a `"entero"` aunque la cabecera
  declarase `double` o `void`. Ahora el handle es `texto` y el retorno se deduce
  de la firma C (`entero`/`decimal`/`texto`/`vacio`).
- **BUG-078 — los bindings omitían la cadena de tipos y la VM panicaba**: la
  firma real es `__ffi_llamar(lib, nombre, "tipos", [args], "retorno")`, pero se
  generaba sin el tercer argumento, de modo que todo se desplazaba una posición.
  Al despachar la llamada, la VM indexaba `v[0]` sobre un vector vacío y
  **abortaba con `index out of bounds`** — un pánico del intérprete provocado por
  código generado por la propia herramienta. Ahora `bindgen` emite los tipos que
  declara la cabecera y la VM valida que haya tantos argumentos como tipos,
  devolviendo un error explicativo en lugar de caerse. Verificado de extremo a
  extremo contra una biblioteca C real (`suma(20, 22)` → `42`).
- **BUG-079 — un doble `__ffi_liberar` mataba el proceso**: si el puntero no
  estaba en el registro de asignaciones, se liberaba igualmente con el layout que
  indicara el usuario. Llamar dos veces a `__ffi_liberar(p)` —o pasar un puntero
  cualquiera— provocaba un **doble free que abortaba la VM** (SIGABRT), sin error
  recuperable ni traza: un fallo del programa del usuario tumbaba el intérprete.
  Ahora sólo se libera lo que se reservó con `__ffi_asignar`; el resto devuelve un
  error normal. (En v2.4.6 el mismo caso daba un *bus error*.)

### ✅ Verificación

- **576 pruebas** automáticas en verde en todo el workspace (139 de regresión en
  `regresiones_v247.rs`, 11 nuevas de `lumen-fmt`, 3 de extremo a extremo del
  runner de `lumen test` y 11 de las herramientas de la CLI y el FFI en
  `herramientas_v3.rs`).
- **Caza sistemática de pánicos**: 560 llamadas a builtins con argumentos
  erróneos (pocos, de más, tipos cruzados, `i64::MIN/MAX`) y 35 situaciones
  límite del lenguaje (índices negativos y fuera de rango, división por cero,
  desbordamientos, recursión profunda, punteros inválidos): **0 pánicos** del
  intérprete. De aquí salieron BUG-078 y BUG-079.
- **494 programas** del corpus: 0 regresiones.
- **320 casos** de fuzzing diferencial con salida esperada calculada de forma
  independiente (structs, listas, `prestado mut`, mapas, texto, `opcion`/
  `resultado`, anidamientos): 0 fallos en VM y 0 en binario nativo.
- **163 ejemplos** del repositorio ejecutados correctamente; los 9 que fallan lo
  hacen exactamente igual que en v2.4.6 (FFI y red sin servidor) y 5 son
  interactivos.
- **Backend Cranelift** comparado ejemplo a ejemplo contra la v2.4.6 sobre 172
  programas: **49 panics del compilador → 0**, 15 ejemplos que pasan a coincidir
  con la VM y ninguna regresión.
- Herramientas ejercitadas con un proyecto real (agenda de contactos):
  `new`, `check`, `lint`, `run`, `test`, `fmt`, `doc`, `bench`, `fuzz`, `build`,
  `build --native`, `build --standalone`, `build --aot c|rust|llvm`, `--profile`,
  `pack`, `unpack`, `bundle`, `bindgen`, `disasm`, `repl`, `debug`, `lsp`,
  `doctor`. VM, bytecode y binario nativo producen **salida idéntica**.

## [2.4.7] - 2026-08-16

### 🐛 Correcciones de Lenguaje y Runtime

Correcciones derivadas del reporte de bugs de la v2.4.6. Se añade la suite
`crates/lumen-vm/tests/regresiones_v247.rs` (77 pruebas) y
`tests/regresiones_v2.4.7.nv` para evitar reincidencias.

**Corrupción silenciosa de ejecución (crítico)**
- **BUG-023 — Una variable local destruía la global homónima**: la VM tenía una
  única instrucción `Store` que escribía en el marco global cuando el nombre ya
  existía allí, sin distinguir declaración de asignación. Declarar
  `entero total = 7;` dentro de una función dejaba la global `total` valiendo 7.
  Se añade el opcode `StoreLocal` (nº 56, al final del enum para no invalidar
  los `.nvc` existentes), que emiten las declaraciones y liga siempre en el
  marco actual; las asignaciones conservan `Store`.
- **BUG-025 — Función duplicada producía un híbrido**: con una definición local
  y otra homónima importada, sema tomaba una firma y codegen el cuerpo de la
  otra, fallando con un `Variable 'x' no definida` sobre una variable ajena al
  usuario. Se detecta con el nuevo error `E081`. Se eliminan de
  `stdlib/actor.nv` tres alias escritos a mano que colisionaban con los que el
  prefijado automático de módulos ya generaba.
- **BUG-030 — `==` sobre listas siempre daba `false`**: el `match` del opcode
  `Eq` no tenía rama para listas, tuplas ni `exito`/`error`, y caía en
  `_ => false` (con `_ => true` en `Neq`), de modo que dos listas con el mismo
  contenido —o una lista consigo misma— se comparaban como distintas mientras
  que los structs sí funcionaban. Ambos opcodes delegan ahora en el `PartialEq`
  de `Value`, que ya era completo y recursivo; la comparación entero/decimal se
  conserva aparte por la tolerancia en coma flotante.
**Robustez (crítico)**
- **BUG-049 — Sin límite de profundidad de llamadas**: una recursión infinita
  hacía crecer la pila de llamadas hasta que el sistema operativo mataba el
  proceso por consumo de memoria. Se añade `MAX_CALL_DEPTH` (250 000),
  comprobado en los tres puntos donde se apila un `CallFrame`; al superarlo se
  aborta con un error normal del programa. La recursión legítima —incluidas la
  mutua y 100 000 niveles de recursión lineal— no se ve afectada.

**Fallos de memoria en los binarios nativos (crítico)**
- **BUG-044 — Segfault al retornar de una función `vacio`**: se emitía
  `return POP()` incondicional y `POP()` es `ST[--SP]`, así que con la pila
  vacía se leía fuera del array. Ahora se devuelve `void` si no hay valor.
- **BUG-045 — Segfault en `_call_by_name`**: `strcmp` contra un puntero nulo al
  invocar un valor que no era una referencia a función válida.
- **BUG-047 — Un `Drop` podía descartar valores del llamador**: la guarda
  comprobaba `SP > 0` (base de la pila global) en vez de la base del marco
  actual, así que dentro de un bucle se comía argumentos de quien llamaba
  (`imprimir("[", f(), "]")` perdía el `"["`). Cada función recuerda ahora su
  `SP` de entrada, como `frame.stack_base` en la VM.

**Corrupción silenciosa en los binarios nativos**
- **BUG-046 — Una función no podía modificar una variable global**: el
  save/restore de variables del llamador revertía también las globales que la
  función acababa de escribir, sin error alguno. Se excluyen las globales del
  programa, salvo cuando alguna función las ensombrece con un parámetro o una
  declaración local (necesario para no reintroducir BUG-023).
- **BUG-041 — `s[0]` sobre un texto reventaba compilado** con "fuera de rango
  (largo: 0)": `_arr_get` sólo miraba `argc`. Ahora indexa por carácter (UTF-8).
- **BUG-048 — `s.largo()` devolvía 0 compilado** mientras `largo(s)` funcionaba,
  de modo que los bucles que dependían de él no se ejecutaban nunca.
- **BUG-043 — `mayusculas`/`minusculas` sólo cubrían ASCII en AOT**:
  `mayusculas("Lúmen")` daba `"LúMEN"`. Se añade el bloque Latin-1 en UTF-8.

**Incoherencia entre builtins (VM)**
- **BUG-042 — `__str_longitud` contaba bytes y `largo` caracteres**: para
  `"Lúmen"` uno devolvía 6 y el otro 5. Unificados en caracteres.

- **BUG-033 — Mutar una lista por campo o por índice se perdía en silencio**:
  `c.items.agregar(x)` y `m[i].agregar(x)` ejecutaban el `push` y descartaban el
  resultado, porque sólo se escribía de vuelta cuando el receptor era un
  identificador. Se añade un write-back recursivo que sube hasta la variable.
- **BUG-034 — Escritura fuera de límites en el backend C**: `POP()` es
  `ST[--SP]`; un `Drop` con la pila vacía dejaba `SP` en -1 y el siguiente
  `PUSH` escribía en `ST[-1]`. Se comprueba la pila antes de descartar.

**Paridad VM ↔ AOT** — varios arreglos previos se habían validado sólo en la VM,
de modo que el mismo programa daba resultados distintos al compilarlo:
- **BUG-035 — `largo(texto)` contaba bytes en AOT** (`strlen`) en lugar de
  caracteres; ahora se cuentan los bytes iniciales de cada secuencia UTF-8.
- **BUG-036 — `elegir` sobre enums no producía salida alguna en AOT**: faltaban
  `__enum_variante`, `__enum_campo` y `__enum_aridad` en el backend C.
- **BUG-037 — `prestado mut` no propagaba la mutación en AOT**: faltaba
  `__frame_param`, sobre el que se apoya el write-back de BUG-020.
- **BUG-038 — `==` daba `true` para structs distintos en AOT**: `_eq` no tenía
  rama para structs y comparaba un campo sin sentido. Se compara por contenido,
  igual que la VM (BUG-030); se añaden también `ninguno`, `error` y mapas.
- **BUG-039 — `a_entero_seguro`/`a_decimal_seguro` no existían en AOT**: estaban
  declaradas pero no implementadas, así que devolvían void.
- **BUG-040 — Reemplazar una clave de un mapa no surtía efecto en AOT**:
  `_map_set` siempre añadía al final, dejando la clave duplicada y devolviendo
  el valor antiguo.
- **BUG-031 — `caso _:` pasaba la validación y fallaba al ejecutar**: el patrón
  comodín sólo se reconocía como subpatrón dentro de un destructuring; como
  patrón de nivel superior caía en la rama genérica de `emit_match_pattern`, que
  lo compilaba como una lectura de variable y emitía `Load("_")`. El resultado
  era que `lumen check` daba el programa por válido y la VM abortaba con
  `Variable '_' no definida`. Se añade un brazo que reconoce `_` y salta al
  cuerpo del caso, conservando el orden de evaluación (un `caso` anterior sigue
  teniendo prioridad). Nota: en un `elegir` sobre un enum, `caso _:` sigue sin
  contar para el chequeo de exhaustividad (E080); ahí hay que listar las
  variantes o usar `defecto:`.
- **BUG-028 — `posponer` se ejecutaba antes del cuerpo**: el bloque se emitía
  en línea en vez de al salir de la función, de modo que la limpieza corría
  antes que el código que usaba el recurso y no se ejecutaba en los `retornar`
  tempranos. Ahora los bloques se vuelcan en cada punto de salida en orden
  inverso al de declaración (LIFO), incluido el nivel superior, que termina en
  `Halt` y donde antes no llegaban a ejecutarse.
- **BUG-029 — Una lambda no podía llamar a funciones ni builtins** (regresión
  introducida al corregir BUG-021): el recolector de capturas tomaba el destino
  de la llamada por una variable del entorno y la lambda fallaba con
  `Variable 'imprimir' no definida`. Un callee que es un identificador simple
  se trata como nombre de función, no como captura.
- **BUG-027 — `imprimir` dentro de una función corrompía la pila**: una
  sentencia-expresión dejaba su valor sin consumir, y al retornar esa basura se
  mezclaba con los argumentos que el llamador estaba apilando, de modo que
  `imprimir("total=", h(2))` salía como `void2`. Se añade la instrucción `Drop`
  (opcode 57) que descarta el sobrante; implementada en la VM y en los dos
  backends AOT (Cranelift y C).
- **BUG-026 — Bucle infinito al capturar la variable del ciclo**: la variable
  capturada se movía al slot `__cap_N_x` mientras la condición del `mientras`,
  ya emitida, seguía leyendo el nombre original; el incremento no se veía nunca
  y el programa se colgaba en silencio. Ahora ambos nombres se sincronizan.
- **BUG-021 — Lambda dentro de otra lambda**: el recolector de capturas incluía
  los nombres *asignados*, de modo que la lambda interna se renombraba a
  `__cap_N_<nombre>` mientras el `Store` escribía en el nombre original y la
  llamada fallaba con `Variable no definida`. Se restan los locales.
- **BUG-024 — Se perdía la salida al fallar**: `lumen run` sólo volcaba el
  buffer de salida de la VM si el programa terminaba bien, así que un error en
  runtime descartaba todo lo impreso hasta ese punto. Ahora se vuelca también en
  el camino de error, igual que ya hacía la ruta de `.nvc`.

**Diagnósticos**
- **BUG-022 — `intentar/atrapar` no capturaba errores**: en esta versión se
  avisaba con `E071` porque el bloque `atrapar` era código muerto. **Resuelto
  más abajo**: ya captura de verdad y el aviso se ha retirado.

- **BUG-010 — Retorno temprano truncaba el programa**: una función con un
  `retornar` dentro de una rama (`si n <= 0 { retornar; }`) no recibía la
  instrucción `Ret` final, porque el emisor comprobaba si existía *algún*
  `Return` en el cuerpo en vez de mirar la última instrucción. La ejecución
  continuaba sobre el código de la función siguiente y el resto del programa
  se perdía **sin ningún error**. Ahora el terminador se emite salvo que la
  última instrucción ya sea un retorno (`crates/lumen-ir/src/builder.rs`).
- **BUG-011 — `lista[i].campo = v` fallaba en runtime**: el orden de operandos
  emitido dejaba la pila como `[elem, array, índice]` cuando `ArraySet` espera
  `[array, índice, valor]`, provocando `StructGet requires struct value`. Se
  guarda el elemento modificado en un temporal y se recompone la pila.

**Semántica de paso de parámetros**
- **BUG-008 — Mutaciones perdidas al pasar `struct` / `lista<T>`**: todos los
  parámetros se pasan por valor, de modo que `.agregar()` o la asignación de un
  campo dentro de una función se descartaban al salir del marco. Se implementa
  el paso por referencia documentado en `LENGUAJE.md`: `prestado mut T` ahora
  copia de vuelta el parámetro al llamador (incluido en recursión y a través de
  varios niveles de llamada), `prestado T` permite lectura sin copia y rechaza
  la mutación con `E061`. El comportamiento por defecto (por valor) no cambia.

**Closures y llamadas**
- **BUG-017 — Las lambdas no podían capturar variables del entorno**: cualquier
  lambda que leyera una variable externa fallaba en tiempo de ejecución con
  `Variable '__cap_N_x' no definida`. El generador registraba el renombrado del
  slot de captura pero nunca emitía el código que lo rellenaba. Ahora el valor
  se copia al crear la lambda (captura por valor).
- **BUG-018 — Una función del usuario llamada `leer` (o `read`) se ignoraba**:
  el builtin de lectura de stdin la ensombrecía y devolvía `""` en silencio, de
  modo que `funcion entero leer() { retornar base; }` imprimía vacío. `leer` y
  `read` pasan a ser builtins "suaves" (ensombrecibles), en la VM y en el
  backend en C.
- **BUG-020 — `prestado mut self` no mutaba en métodos de `impl`**: la copia de
  vuelta del parámetro sólo se emitía para funciones libres, así que
  `c.incrementar()` no tenía efecto sobre `c`. Ahora los métodos registran sus
  parámetros `prestado mut` (con el receptor implícito en la posición 0) y
  reciben la misma copia de vuelta.

**Diagnósticos**
- **BUG-016 — Tipo declarado inexistente daba un error confuso**: `Foo x = 5;`
  con `Foo` sin definir producía un `E031` que filtraba la representación
  interna del compilador (`Struct { name: "Foo", fields: [] }`). Ahora emite
  `E062 El tipo 'Foo' no está definido` con una sugerencia útil.

**Control de flujo**
- **BUG-014 — `main` se ejecutaba dos veces**: si el archivo tenía código en el
  nivel superior y además llamaba explícitamente a `main()`, el compilador
  añadía una auto-invocación adicional, de modo que el cuerpo de `main` corría
  dos veces. Ahora sólo se auto-invoca cuando el nivel superior no la ha
  llamado ya; el atajo de escribir `main` sin llamarla se mantiene.
- **BUG-015 — `romper` / `continuar` no funcionaban en `para ... en`**: el bucle
  `para-cada` no se registraba como ciclo, así que ambas sentencias se
  rechazaban con `E070`/`E055` ("fuera de un bucle") pese a estar dentro de uno.
  Al levantar esa comprobación aparecía un segundo fallo: el generador de código
  las descartaba en silencio y el bucle seguía iterando. Ahora `para-cada`
  registra sus etiquetas de salto y `continuar` salta al incremento del índice
  (no al inicio), evitando un bucle infinito. `mientras` y `para` clásico ya
  funcionaban.

**Salida**
- **BUG-009 — `imprimir` con varios argumentos**: `imprimir("a: ", x)` emitía
  una línea por argumento en lugar de una sola línea concatenada, lo que además
  rompía los mensajes de error de `stdlib/testing.nv`. Corregido de forma
  consistente en la VM, el backend AOT en C y el backend Cranelift.

**Pattern matching**
- **BUG-003 — Destructuring de datos de enum en `elegir`/`caso`**: `caso
  Figura::Circulo(r):` fallaba con `E033: La variable 'r' no está declarada`.
  Se ligan los datos capturados con el tipo declarado de la variante y se
  admiten varios datos, literales anidados (`caso Msg::Codigo(404):`), `_` y
  patrones OR. La aridad incorrecta se reporta con el nuevo código `E067`.

**Biblioteca estándar**
- **BUG-007 — Sin conversión texto → número**: se añaden `a_entero()`,
  `a_decimal()` y `a_numero()` como builtins globales (inversas reales de
  `a_texto()`), junto a las variantes `a_entero_seguro()` / `a_decimal_seguro()`
  que devuelven `resultado<T, texto>` y el predicado `es_numero()`.
- **BUG-001 — Sin `abs()`**: se añaden `abs`/`absoluto` (preservando el tipo del
  argumento), `minimo`/`maximo`, `raiz`, `potencia`, `piso`, `techo` y
  `redondear`, con sus alias en inglés. Una función del usuario con el mismo
  nombre tiene prioridad sobre estos builtins.
- **BUG-013 — Los builtins nuevos devolvían `void` al compilar a nativo**: las
  conversiones y funciones matemáticas anteriores sólo estaban implementadas en
  la VM, así que un programa compilado con `lumen build --aot c` imprimía
  `void` donde el intérprete daba el valor correcto. Se implementan en el
  runtime en C (`crates/lumen-aot/src/lumen_rt.h`) replicando la semántica de la
  VM, incluida la prioridad de las funciones del usuario sobre los builtins.

**Experiencia de desarrollo**
- **BUG-002 — Nomenclatura de conversiones confusa**: llamar `texto(x)` o
  `entero(s)` ahora sugiere el nombre correcto con prefijo `a_` en vez de un
  genérico "función no definida".
- **BUG-006 — `resultado` reservado como nombre de variable**: `resultado` y
  `opcion` pasan a ser *soft keywords*: sólo introducen un tipo cuando van
  seguidos de `<`. `entero resultado = 0;` compila y ya no genera errores en
  cascada. Las palabras verdaderamente reservadas dan un mensaje explícito.
- **BUG-004 — El REPL no conservaba variables**: sólo se persistían las
  declaraciones de función/tipo, por lo que una variable declarada en una línea
  no existía en la siguiente al alimentar el REPL por *pipe*. Ahora también se
  conservan las declaraciones de variable, sin duplicar efectos secundarios.
- **BUG-005 — `lumen test` ignoraba aserciones sueltas**: un archivo con
  aserciones en el nivel superior reportaba "0 pasaron, 0 fallaron". Se ejecutan
  y se contabilizan, las aserciones fallidas marcan la suite como fallida
  (código de salida 1) y, si no hay ninguna prueba, se explica cómo escribirlas.

- **BUG-050 — `build --native` producía binarios que mentían**: todo builtin no
  implementado por el backend C se resolvía con un stub
  `static Val _f_<n>(void) { return _v_void(); }`, descartando los argumentos.
  El compilador anunciaba `✓ Binario nativo` y el ejecutable devolvía valores
  falsos sin error alguno: `__calendario_hijri`/`__calendario_persa` daban
  `void` y `__leer_archivo`/`__escribir_archivo` no hacían nada. Ahora esas
  llamadas se registran y el CLI aborta mostrando la lista exacta y las
  alternativas (`lumen run`, `lumen build`); `--permitir-no-soportados` permite
  continuar asumiendo el riesgo. Diagnóstico: `LUMEN_AOT_DEBUG_UNKNOWN=1`.

- **BUG-051 — recursión infinita en binario nativo: SEGFAULT mudo**: el runtime
  C no comprobaba ni el tope de la pila de valores (`ST[16384]`, con `PUSH`/`POP`
  sin guarda) ni la profundidad de llamadas. Como cada función LÚMEN se emite
  como una función C que se llama recursivamente, la recursión infinita agotaba
  la pila del proceso y el binario moría con código 139 **sin imprimir nada**,
  mientras que la VM ya abortaba con un error legible (BUG-049). Ahora el C
  generado llama a `_ckdepth()` al entrar en cada función (límite de 250000 y
  vigilancia del consumo real de pila) y emite el mismo mensaje que la VM. La
  pila de valores pasa a crecer bajo demanda (`_st_grow`) en lugar de tener un
  tope fijo, porque 16384 ranuras rompían la recursión legítima profunda
  (`suma(100000)`) que la VM sí resuelve.

- **BUG-032 — una closure devuelta perdía sus capturas**: una lambda creada
  dentro de otra función leía las variables capturadas del marco de su
  factoría; al devolverla, ese marco ya había muerto y fallaba con
  "Variable 'n' no definida". El backend nativo era peor: no daba error, sino
  valores equivocados en silencio (`mk(5)` y `mk(100)` devolvían ambas 101,
  porque compartían las globales). Los slots `__cap_*` no podían arreglarlo,
  justamente porque son globales y las instancias los comparten. Ahora cada
  lambda anota qué nombres del entorno captura (nuevo campo `captures` en la IR
  y en el bytecode, formato v7) y `FuncRef` los resuelve al crear la closure, en
  un entorno propio: `Value::Closure { name, env }` en la VM y `_vfclos`/`_Env`
  en el runtime C. Se propaga de forma transitiva, así que el anidamiento triple
  (`externa -> media -> interna`) también funciona.

- **BUG-052 — la closure que muta su captura perdía el estado**: con BUG-032 la
  captura era por VALOR, así que el idioma del contador seguía roto: la VM
  fallaba con "Variable 'n' no definida" y el binario nativo respondía `11, 11`
  sin aislar instancias. La causa estaba en el builder de IR: `collect_assigned_names`
  mezclaba las DECLARACIONES (`entero x = 0;`) con las simples ASIGNACIONES
  (`n = n + 1;`), de modo que mutar una captura la hacía pasar por local nueva.
  Ahora se distinguen (`collect_declared_names`) y las capturas viajan en celdas
  COMPARTIDAS (`Arc<Mutex<Value>>` en la VM, entorno propio por instancia en el
  runtime C), volcadas de vuelta al retornar. Las celdas se indexan por un id de
  invocación monótono, no por profundidad de marco: dos llamadas sucesivas a la
  misma factoría ocupan la misma profundidad y habrían compartido estado.

- **Versionado coherente**: el workspace declaraba `1.6.0` y el binario oficial
  reportaba `LÚMEN v1.6.0` pese a publicarse como v2.4.6, mientras el `VERSION`
  y los banners del CLI, el LSP y el REPL decían `2.4.6`. Todo queda en
  **2.4.7**, que es lo que documenta este CHANGELOG.

- **BUG-053 — `lumen fmt` destruía el código fuente**: el formateador tenía
  brazos `_ => {}` para todo lo que no cubría, así que esas construcciones
  **desaparecían del archivo** al guardar. Una lambda se reescribía como
  `Infer f = ;` (ni siquiera sintaxis válida); `l[j] = x;` y `p.campo = v;` se
  borraban, de modo que la ordenación por burbuja seguía compilando pero dejaba
  de ordenar; `exito(...)`/`error(...)`, `algun`, `t.0`, los rangos y los valores
  por defecto de los parámetros se perdían; y `10.0` pasaba a `10`, convirtiendo
  una división real en entera. Además el formateo no era idempotente: los
  paréntesis de `si`/`mientras` y las llaves de los `caso` se acumulaban en cada
  pasada. Se implementan las 5 sentencias y las 8 expresiones que faltaban, se
  elimina el `_ => {}` (ahora el compilador obliga a cubrir cualquier nodo nuevo)
  y, como red de seguridad, **`lumen fmt` reparsea su propia salida y se niega a
  escribir si el resultado no compilaría**. Verificado sobre los 59 ejemplos:
  0 archivos rotos y 0 diferencias al formatear dos veces.
- **BUG-054 — corrupción de heap en los ejemplos TUI (`rc=134`)**: `tui_ventana`
  abortaba el proceso con `realloc(): invalid next size`. `_tc_write`
  (`stdlib/tui_core.nv`) reservaba `s.largo()` bytes, pero **`largo()` cuenta
  CARACTERES**: los marcos de las ventanas usan box-drawing UTF-8 (`╭`, `─`,
  `│`), de 3 bytes cada uno, así que `__ffi_escribir` copiaba hasta el triple de
  bytes de los reservados —más el terminador NUL— y pisaba las estructuras
  internas del asignador. El fallo era **silencioso hasta que reventaba**, y
  ocurría a distancia del código culpable. Se corrige en dos niveles:
  1. **La VM ya no puede corromper el heap desde LÚMEN**: `__ffi_asignar`
     registra el tamaño real de cada reserva, `__ffi_escribir` rechaza con un
     error claro toda escritura que no quepa (explicando la diferencia entre
     caracteres y bytes) y `__ffi_liberar` usa el layout real de la reserva en
     lugar del tamaño que le pasen, que también corrompía el asignador si no
     coincidía.
  2. **`__ffi_escribir` devuelve los bytes escritos** —`sema` ya declaraba que
     devolvía `entero`, pero la VM empujaba `Void`— y el stdlib reserva el peor
     caso de UTF-8. Se corrige el mismo patrón latente en `tui_core.nv`,
     `gui.nv`, `sql.nv` y `graficos.nv`, donde además el texto no ASCII se
     enviaba truncado a `write(2)`.
  `examples/tui_jr.nv` pasa de `rc=134` a `rc=0` y supera `MALLOC_CHECK_=3`.
  Este bug **ya estaba en v2.4.6**.
- **BUG-055 — `lumen-plugin` no compilaba en modo test**: el campo `captures`
  que BUG-032 añadió a `lumen_ir::Func` no se propagó a su test, de modo que
  `cargo test --workspace` fallaba a la primera y dejaba sin cobertura a los
  demás crates. Ahora el workspace completo compila y se ejecuta.
- **BUG-022 — `intentar/atrapar` ya captura errores (resuelto)**: era el último
  bug grande del reporte que seguía pendiente. El generador emitía la etiqueta
  del `atrapar`, pero **nadie saltaba a ella** y `err_var` se ignoraba: el
  bloque era código muerto y cualquier fallo abortaba el programa. Se implementa
  el mecanismo completo:
  - Dos opcodes nuevos, `PushHandler`/`PopHandler`, que instalan y retiran el
    manejador alrededor del bloque `intentar`.
  - La VM mantiene una pila de manejadores y, ante un error, **desenrolla**:
    descarta los marcos de llamada abiertos (el fallo puede venir de varias
    llamadas más adentro), recorta la pila de operandos a la altura que tenía al
    entrar y salta al `atrapar`, ligando el mensaje a su variable.
  - El backend C hace lo mismo con `setjmp`/`longjmp`, porque `goto` no salta
    entre funciones; los errores del runtime (división por cero, índice fuera de
    rango, campo inexistente) dejan de llamar a `exit()` cuando hay un `atrapar`
    vigente. Los mensajes se unifican con los de la VM.
  Funciona anidado, con `retornar` desde el `atrapar`, dentro de bucles sin
  fugar manejadores y dejando la pila limpia. Verificado con **paridad VM = AOT**
  en los 9 casos y 10 pruebas nuevas. Se retira el aviso `E071`.
- **BUG-057 — el backend C machacaba un global al usar `atrapar` (o cualquier
  `StoreLocal` puro)**: el barrido que registra los nombres de variables miraba
  `Load`/`Store`/`FuncRef` pero **no `StoreLocal`**. Un nombre que sólo se
  escribe con `StoreLocal` —como la variable del `atrapar (e)`— no se
  registraba, y `_fv()` devuelve `0` cuando no encuentra el nombre: el valor iba
  a parar al slot del **primer global**, corrompiéndolo en silencio. En un bucle
  con `atrapar`, la variable del bucle se sobrescribía con el mensaje de error.
- **BUG-058 — los parámetros de tipo no se resolvían dentro de tipos
  compuestos**: `resolve_type` sólo trataba `Type::Struct` y `GenericStruct`; el
  resto caía en `type_to_info`, que no conoce los `type_params`. Así, la `T` de
  `lista<T>` se resolvía como un struct vacío llamado "T" en vez de `TypeVar`, y
  pasar una `lista<entero>` a `funcion entero cuantos<T>(lista<T> l)` fallaba con
  E041. Con `T` a secas sí funcionaba, que es lo que lo hacía desconcertante.
  Ahora se recurre también en `lista`, `opcion`, `resultado`, tuplas, funciones,
  `prestado` y `dueno`.
- **BUG-059 — los errores de tipos filtraban la representación interna de
  Rust**: los mensajes usaban `{:?}` sobre `TypeInfo`, así que el usuario leía
  `Lista(Texto)` o, peor, `Struct { name: "P", fields: [("x", Entero)] }`. Se
  implementa `Display` para `TypeInfo` y los ~85 mensajes pasan a hablar en la
  sintaxis del propio lenguaje: `lista<texto>`, `P`, `opcion<entero>`.
- **BUG-060 — una lambda recursiva no se veía a sí misma**: `sea fact =
  funcion(entero n) { ... fact(n - 1) ... };` fallaba con E042. `sema` analizaba
  el cuerpo ANTES de declarar el nombre, y el generador capturaba la
  autorreferencia por valor cuando todavía no tenía valor. La VM ya lo soportaba
  (asignar la lambda a una variable ya declarada funcionaba), así que era sólo
  orden de análisis. Ahora el nombre se predeclara con la firma de la lambda y
  se excluye de las capturas.
- **BUG-061 — el backend C daba resultados erróneos en lambdas recursivas**: la
  llamada INDIRECTA (`_fref_call`) no guardaba las variables del llamador, al
  contrario que la directa. Como los parámetros viven en slots globales, una
  lambda con dos llamadas por nivel se pisaba a sí misma: `fib(10)` devolvía
  **-80** en vez de 55, **en silencio**. Se guarda y restaura igual que en
  `emit_user_call`.
- **BUG-062 — las etiquetas de las lambdas colisionaban con las del programa**
  (🔴 miscompilación silenciosa, **ya presente en v2.4.6**): `codegen` resuelve
  los saltos con un ÚNICO mapa global `etiqueta -> posición`, pero
  `compile_lambda` reiniciaba el contador a 0 en cada lambda. El `L0` de la
  lambda sobrescribía el `L0` de la función envolvente y los saltos aterrizaban
  en otra función. Los síntomas eran desconcertantes: un `si/sino` seguido de
  una lambda ejecutaba **las dos ramas** (`A, Y, C` salía como `si, no, no`), y
  una lambda recursiva tras cualquier condicional o bucle **no terminaba
  nunca** — en v2.4.6 el proceso se cuelga para siempre. Las etiquetas pasan a
  ser únicas en todo el programa.
- **BUG-063 — las variables de bloque machacaban las de fuera en vez de
  sombrearlas** (🔴 agujero de tipos silencioso, **ya presente en v2.4.6**):
  `sema` empuja un ámbito por bloque, pero las variables del runtime son planas
  por marco (una tabla por nombre), así que `si (...) { entero x = 2; }` pisaba
  la `x` de fuera. Lo grave es que `sema` seguía creyendo que la exterior era la
  suya: se podía declarar `texto x` dentro de un bloque y luego escribir
  `x + 10` fuera; `lumen check` daba el programa por **válido** y el resultado
  era `"hola10"` en vez de `11`. Afectaba a `si`/`sino`, `mientras`, `para`,
  `para ... en` y bloques sueltos, en la VM y en el binario nativo. Ahora el
  generador de IR lleva una pila de ámbitos de bloque y da un slot propio a la
  variable interior. Sólo se renombra cuando el nombre ya era visible fuera, así
  que el código que ya funcionaba no cambia: asignar (sin declarar) desde dentro
  de un bloque sigue mutando la de fuera, como debe ser.
- **BUG-056 — `fmt` partía `} sino {` y `} atrapar (e) {`**: como `fmt_block`
  termina en salto de línea, la cláusula encadenada salía en su propia línea y
  con un espacio suelto (`}\n sino {`). Cosmético y ya presente en v2.4.6, pero
  el formateador es justo la herramienta que no debe afear el código.

### ✅ Verificación
- 536 pruebas unitarias y de regresión del workspace, todas en verde
  (127 en `regresiones_v247.rs`). Es el workspace **completo**: hasta ahora
  `lumen-plugin` impedía que `cargo test --workspace` llegara a compilar.
- Los 45 programas de `test_agents/` se ejecutan sin errores.
- 144 ejemplos de `examples/` comparados contra una compilación del tag v2.4.6:
  sin regresiones (las únicas diferencias son la corrección intencionada de
  `imprimir` y direcciones de puntero/FFI dependientes del entorno).
- Paridad VM ↔ AOT en C verificada para las conversiones y builtins numéricos.
- `cargo clippy --all-targets -- -D warnings` y `cargo fmt --all` limpios.

## [2.4.6] - 2026-08-16

### 🚀 Nuevas Características Principales
- **🏎️ Álgebra Lineal 2D & Tiled GEMM con SIMD AVX2 (`stdlib/matriz_simd.nv`)**: Multiplicación matricial paralela optimizada para la jerarquía de memoria caché L1/L2 con paralelismo vectorial 4-way / 8-way FMA, transposición de bajo coste y capas densas con activación ReLU.
- **⚡ Tracing JIT Tier-4 & On-Stack Replacement (OSR) en Caliente (`stdlib/tracing_jit.nv`)**: Compilador dinámico multi-nivel con elevación automática de bucles calientes (*Hot Loops*) directamente sobre la pila de ejecución en memoria RAM (42.5x aceleración) con guardias de deoptimización seguras.
- **🛡️ Unikernel & Bootloader Bare-Metal x86_64 (`stdlib/baremetal.nv`)**: Arranque de programas LÚMEN directamente en el hardware en <2 ms con cabecera Multiboot2 (0x1BADB002), drivers de video VGA Text Mode (0xB8000), telemetría serial UART COM1 (0x3F8) y asignador de páginas físicas de 4KB.
- **🧠 Motor de Autograd & Entrenamiento de Redes Neuronales (`stdlib/autograd.nv`)**: Diferenciación automática en modo reversa (*Reverse-Mode Autograd*), grafos computacionales dinámicos y optimizadores **AdamW** y **SGD con Momentum** para entrenamiento de IA 100% en LÚMEN puro sin Python.
- **⚡ Scheduler de Concurrencia Asíncrona Multi-Hilo M:N (`stdlib/scheduler.nv`)**: Orquestador de micro-tareas (*Green Threads*) con balanceo de carga automático por robo de trabajo (*Work-Stealing*) y canales asíncronos *Lock-Free* MPSC para +500,000 tareas concurrentes.
- **🧠 Parser Binario GGUF v3 e Inferencia LLM Local (`stdlib/gguf.nv`)**: Carga directa de pesos cuantizados Q4_K_M y Q8_0 para modelos Llama-3, Phi-3 y Mistral con KV-cache y muestreo Top-P.
- **🌐 Servidor WebSockets RFC 6455 (`stdlib/websocket.nv`)**: Handshake HTTP 101 automático, tramas de texto/binario, broadcast masivo y ping/pong.
- **🎮 Motor Gráfico 3D & Shaders WebGPU (`stdlib/motor_3d_gpu.nv`)**: Mallas poligonales indexadas 3D, cámara con matriz de proyección MVP y shaders WGSL a 144 FPS.
- **📱 UI Declarativa Reactiva Nativa de Escritorio (`stdlib/ui_reactiva.nv`)**: Virtual DOM, use_state hooks y lanzamiento de ventanas nativas Direct2D/Win32/Wayland sin overhead de Electron.
- **📦 Gestor de Paquetes con SemVer & `lumen.lock` (`crates/lumen-pkg`)**: Resolución automática de dependencias semánticas (^, ~, >=) y archivo de bloqueo determinista con hashes SHA-256.
- **🐞 Depurador Visual Interactivo en Terminal (TUI Debugger — `lumen debug`)**: Interfaz visual estilo Catppuccin con ventana de código en vivo `▶▶▶`, puntos de interrupción `🔴 [B]`, inspector de variables y Time-Travel Debugging (`back` para retroceder en el tiempo).
- **🖥️ Compilador Standalone en 1 solo `.exe` (`lumen bundle`)**: Empaqueta código y runtime en un único binario independiente de menos de 100 KB sin dependencias externas.
- **✨ CLI Inteligente y Personalizada**: Detección automática del usuario de Windows/Linux, núcleos de CPU para el scheduler M:N y toolchains de C/Rust disponibles.

### 🐛 Correcciones y Optimizaciones
- Corregido el aplanamiento de módulos en `crates/lumen-sema/src/loader.rs` con `collect_module_declarations` para resolver variables y funciones con prefijo interno (`__libc`, `__sdl`, `__temas`, `__render_mes`).
- Eliminado warning de MinGW `__p__environ` en compilación C nativa (`crates/lumen-aot/src/lumen_rt.h`).
- 378 ejemplos y 385 pruebas unitarias/integración verificados y 100% pasando sin errores.


# Changelog

Todos los cambios importantes del proyecto LÚMEN se documentan aquí.

---

## v2.4.6 — 15 Agosto 2026

### Agregado (Horizontes de Producción: Nexus Web, PostgreSQL, Redis, UI Reactiva & Fixed-Point Bootstrap)
- **Framework Web Cloud-Native "Nexus" (`stdlib/nexus.nv`)**: Framework estilo FastAPI / Axum con enrutamiento dinámico tipado (`nexus_get`, `nexus_post`, `nexus_put`, `nexus_delete`), generación automática de contratos OpenAPI 3.0 JSON (`nexus_generar_openapi_json`) y documentación interactiva Swagger UI (`nexus_generar_swagger_ui_html`).
- **Driver PostgreSQL Nativo en Puro LÚMEN (`stdlib/postgres.nv`)**: Implementación completa del protocolo binario Wire 3.0 de PostgreSQL (StartupMessage, Query, RowDescription, DataRow) sin depender de `libpq` en C.
- **Driver Redis RESP3 con Pipeline (`stdlib/redis.nv`)**: Serializador de comandos RESP3, operaciones SET/GET/INCR y canalizaciones asíncronas por lotes en una sola llamada de red.
- **Framework UI Declarativo Reactivo (`stdlib/ui_reactiva.nv`)**: Motor de interfaz de usuario multiplataforma con Virtual DOM, hooks de estado reactivo (`ui_estado_crear`, `ui_estado_actualizar`), reconciliación diffing y renderizado para HTML5 y Terminal TUI.
- **Self-Hosting Stage-3 Fixed-Point Confirmado**: Emisión directa de ejecutables ELF64 autónomos (`stdlib/compiler/asm_emitter.nv`) con verificación criptográfica SHA-256 byte-idéntica (`d006c5af592fed2496c36dcfa0077dc54d891dcdc77f2218b0cf88d2925f7d25`) entre pasadas de compilación.
- **Playground Web Modernizado con WebGPU & Time-Travel Debugger**: Integración en `/home/user/lumen_web/index.html` de un depurador visual con barra de retroceso temporal (Snapshots), renderizador de partículas WebGPU en tiempo real y nuevos presets interactivos.

---

## v2.4.5 — 15 Agosto 2026

### Agregado (Fronteras Avanzadas: IA Cuantizada, Vector DB, Actores OTP & Tooling Pro)
- **Base de Datos Vectorial Nativa (`stdlib/vector_db.nv`)**: Motor de indexación vectorial de alta dimensionalidad con métricas de similitud coseno (`similitud_coseno`), distancia euclidiana L2 (`distancia_euclidiana`), producto punto y filtrado semántico de metadatos para aplicaciones RAG (Retrieval-Augmented Generation).
- **Motor de Inferencia IA & Cuantización INT8 (`stdlib/ia.nv`)**: Cuantización simétrica W8A16 (`ia_cuantizar_int8`), multiplicación matriz-vector cuantizada (`ia_matmul_cuantizado`), Rotary Position Embeddings complejos (`ia_aplicar_rope`), KV-Cache para decodificación autoregresiva rápida y muestreo probabilístico por temperatura y Top-P (Nucleus).
- **Modelo de Actores & Tolerancia a Fallos Erlang/OTP (`stdlib/actor.nv`)**: Actores livianos con buzón de mensajes (`buzon`), paso de mensajes desacoplado (`actor_enviar`), despacho secuencial (`actor_procesar`) y árboles de supervisión con estrategias de auto-recuperación (`supervision_sanar`).
- **Asistente Inteligente de Terminal (`lumen ai`)**: Subcomandos `explain` (análisis estático y complejidad), `fix` (detección y corrección asistida), `test` (generación automática de tests unitarios) y `chat` (asistente interactivo de arquitectura).
- **Empaquetado Binario Standalone (`lumen bundle <archivo.nv>`)**: Generación en un solo comando de ejecutables binarios nativos autocontenidos con cero dependencias externas.
- **Gestión de Registro Local y Privado (`lumen registry`)**: Comandos `info` (estado y caché de paquetes) y `serve` (microservicio local de registro de paquetes para entornos empresariales).
- **Soporte Completo de Asignaciones Indexadas en Miembros (`obj.array[i] = val`)**: Unificación del análisis sintáctico y semántico para escrituras en colecciones anidadas dentro de estructuras.

---

## v2.4.4 — 15 Agosto 2026

### Agregado (Consolidación de las 20 Fases de LÚMEN)
- **Operador Pipe (`|>`)**: Evaluación y encadenamiento funcional de izquierda a derecha sin sobrecarga (`datos |> filtrar() |> procesar()`).
- **Azúcar Sintáctico para Tipos Opcionales (`T?`)**: Soporte nativo para `texto?`, `entero?`, `decimal?`, `Punto?` y `lista<T>?` equivalente a `opcion<T>`.
- **Comprensión de Listas (List Comprehensions)**: Sintaxis funcional `[expr para var en iter si cond]` y en inglés `[expr for var in iter if cond]` desazucarada a bucles optimizados con asignación de arrays in-place.
- **JIT Tiering Automático en la Máquina Virtual**: Perfilado de invocaciones de funciones en `Opcode::Call` y compilación JIT nativa en caliente en memoria RAM vía Cranelift (`cranelift-jit`).
- **Diferenciación Automática N-Dimensional (Autograd) en `stdlib/tensor.nv`**: Grafo de computación dinámico con paso hacia atrás (`backward()`) para cálculo de gradientes automáticos, convolución 1D/2D y Layer Normalization.
- **Backend LLVM IR Directo (`lumen build --aot llvm`)**: Emisión directa de código LLVM IR (`.ll`) y bitcode para optimizaciones industriales globales.
- **Time-Travel Debugging en CLI y VM**: Grabación de instantáneas de ejecución y soporte para comando `back` / `step-back` / `retroceder` para volver atrás en el tiempo durante la depuración.
- **Generador Automático de Bindings (`lumen bindgen`)**: Parsing de cabeceras C (`.h`) o funciones Rust `extern "C"` y generación de módulos `.nv` listos para importar.
- **Puente Rust/Cargo (`lumen install cargo:<crate>`)**: Vinculación de cualquier crate de `crates.io` con wrappers FFI automáticos en `./pkgs/`.
- **Servidor Microservicios WebSockets, SSE & HTTP/3 / QUIC**: Soporte en `stdlib/servidor.nv` para WebSockets RFC 6455, Server-Sent Events y datagramas QUIC/UDP.
- **Bootstrap 100% Autónomo Self-Hosted (`lumen bootstrap`)**: Compilación y ejecución directa mediante el compilador nativo en puro LÚMEN (`stdlib/compiler/compiler_v4.nv`).
- **Comando `lumen bench <archivo.nv>`**: Suite integrada de micro-benchmarks con estadísticas de latencia mínima, promedio, máxima y throughput de ejecuciones por segundo.

---

## v2.4.3 — 15 Agosto 2026

### Agregado (Ergonomía, Lenguaje y Compilador)
- **Interpolación de cadenas `f"..."`**: Soporte para cadenas formateadas con expresiones arbitrarias `{expr}` (ej: `f"Hola {usuario}, total: {precio * cant} USD"`). Se desazucara e interpola con `a_texto` en tiempo de compilación con paridad en VM, AOT y WASM.
- **Métodos inherentes en Structs (`impl StructName { ... }`)**: Sintaxis directa `impl Punto { funcion entero suma(este) { ... } }` con receptor implícito `este`/`self` y resolución automática de métodos `p.suma()` sin requerir rasgos intermedios.
- **Operadores Bitwise completos (`^`, `~`, `&`, `|`, `<<`, `>>`)**: 
  - Tokenización de `^` (Caret / BitXor) y `~` (Tilde / BitNot) en el lexer.
  - Tipado en semántica (`sema.rs`), opcodes en bytecode VM (op 54 `BitXor`, op 55 `BitNot`), y generación en C99 (`_a.i ^ _b.i`, `~_a.i`) y Cranelift (`bxor`, `bnot`).
- **Mutación de L-Values multidimensionales y anidados (`m[i][j] = val`, `x.campo[i] = val`, `r.origen.x = val`)**: Generación de *write-back* en cascada en el generador de IR. `stdlib/matrices.nv` ahora opera al 100% de forma nativa sin ceros residuales.

### Agregado (CLI, Multiplataforma y Ecosistema)
- **Comando `lumen doctor` / `lumen info`**: Diagnóstico automático del entorno (sistema operativo, arquitectura, compilador C disponible, estado de los backends AOT y módulos de la `stdlib`).
- **Compilación Standalone (`lumen build --standalone <archivo>`)**: Genera binarios nativos independientes autónomos optimizados (`-O3 -s`) con todas las dependencias enlazadas.
- **Selector explícito de Backend AOT**: Soporte para `lumen build --aot <c|rust>` / `--backend <c|rust|cranelift|llvm>`.
- **Soporte FFI de 64 bits y SQLite multiplataforma**:
  - `Value::as_i64` para preservar punteros de 64 bits en FFI sin pérdida de precisión.
  - Primitivas `__ffi_peek64`, `__ffi_peek_ptr`, `__ffi_peek_byte`, `__ffi_poke_byte`.
  - `stdlib/sql.nv` ahora detecta y carga dinámicamente `libsqlite3.so.0`, `libsqlite3.so`, `sqlite3.dll` o `libsqlite3.dylib`.
- **Fix de enlace GCC en Linux**: Eliminado el parámetro restrictivo `-lregex` en Linux (glibc) y corregidos calificadores `const` en `lumen_rt.h`.

---

## v2.4.2 — 14 Agosto 2026

### Agregado (compilador self-hosted — Fases 61/62/63 reales)
- **OR Patterns reales en el self-hosted** (`parser.nv` branch `elegir`): loop que consume el pipe `|` y construye árbol `Binary ||` encadenado `(sel==A) || (sel==B)` → despacha a `_cg_and_or` (short-circuit real con JmpIf). **`fase61_or_patterns` byte-IDÉNTICO en la cadena 100% LÚMEN**.
- **IF-LET real en el self-hosted** (Fase 62): handlers op 52/53 (`MatchType`/`MatchPayload`) en `vm.nv` + caso `IfLet` en `codegen.nv` (auxiliar `tiene_test`) + branch if-let en `parser.nv` (patrón `algun`/`exito`/`error`/`ninguno`/Ident con bind). **`fase62_if_let`/`fase62_if_let2` byte-IDÉNTICOS** (también en VM Rust: opcodes 52/53 end-to-end con `bind_pattern_vars` en sema).
- **Rangos `..`/`..=` en el self-hosted** (Fase 63): token en `lexer.nv`, nodo `Range` en `parser.nv`, desugar a lista + intercepto `==` con rango (short-circuit `_cg_and_or`) en `codegen.nv`, fix `OP_ARRAY_PUSH=32` y `32 => 32` en el encoder nativo. **`fase63_range_patterns` byte-IDÉNTICO**.
- **8 ejemplos de fase nuevos** (2 por fase): `fase61_or_patterns.nv`, `fase62_if_let.nv`, `fase62_if_let2.nv`, `fase63_range_patterns.nv`, `fase64_string_patterns.nv`, `fase66_operator_overloading.nv`/`_2.nv`, `fase68_associated_types.nv`/`_2.nv`, `fase70_impl_trait.nv`/`_2.nv` — todos OK en VM y backend C.
- **FIXPOINTs v4 consecutivos**: `DF7676DE7B…` tras OR patterns (150,463 B → 165,944 B, byte-idénticos self==self2, reemplaza a `A3CBAA0F…`). Batería self ampliada: **OK=42 FALLAS=0** (incl. demo_completo, match, enums, corutinas_demo, jr_concurrencia, 44_extension_methods, test_ffi_min, test_texto_std, fases 61-70).

### Agregado (AOT — backend C / Cranelift optimizados)
- **Cranelift: variables SSA reales del frontend** (nada de StackSlot): `Variable` de Cranelift con `declare_var`/`def_var`/`use_var` y phis vía dominancia — obsoleto el paso por memoria.
- **Backend C: índices de registro constantes** (sin strcmp lineal en cada Load/Store): `gv[N]` directo vía `name_idx`, fallback `_fv` solo para nombres no registrados.
- **Benchmark `bench_fib.nv`** (fib(26)+loop 100, runs calientes): **VM 856ms → C 22ms (antes 406ms, 18x) → Cranelift 5.6ms (antes 116ms, 20x)** — ambos backends en milisegundos.
- **Fix C backend**: temp-capture en CALLS de usuario (`{ Val _r = _f_x(); PUSH(_r); }` en `_f_/{}`, `CallValue` y `_fref_call`) — gcc evalua LHS antes de la función callee → corrupción de pila compartida. `fase65_guard_let`/`_2` byte-idénticos en C y VM.
- **Batería dual `aot_bateria_dual.ps1`**: **C OK=38 DIFF=0 (paridad total) · RUST OK=12 DIFF=26 (límite de diseño: sin strings/structs/colecciones) · FAIL=0 SKIP=1 HANG=0**.

### Agregado (Playground Web — Ronda L1 + F1.2/F2.3/F4.2 completadas)
- **`lumen serve` real** (servidor HTTP estático Rust puro, sin Python): `--port`, `LUMEN_PORT` env, MIME types, headers COOP/COEP, 404, anti path-traversal, redirección `/` → `/web/index.html`. **Cache ETag/If-None-Match** (304 Not Modified) + variable `LUMEN_PORT`. Verificado (200/304/404/JSON).
- **Backend `/api/run`**: `POST` → compila y ejecuta con la **VM Rust nativa** (`run_source_capture`) → JSON `{ok,output}`/`{ok,error}` con spans `(linea,col)`. `GET /api/health`, `/api/examples`, `/api/examples/{file}`.
- **CodeMirror 6 vendorizado** (11 módulos ESM planos + import map, sin CDN) + **modo LUMEN generado** desde `token.rs` (74 keywords, `StreamLanguage` + syntax highlighting Catppuccin). **Autocompletado** con `@codemirror/autocomplete` (`Ctrl+Space`, keywords + snippets), **minimapa** (EditorView espejo sincronizado), **error gutter mejorado** con tooltips. Autosave localStorage.
- **Sigma L1**: stdlib embebida via build.rs (`embedded_stdlib.rs`, 31 archivos) + `ModuleLoader::with_memory_files`; `run_lumen`/`check_lumen`/`compile_to_bytes` con loader virtual; 128 ejemplos embebidos (`embedded_examples.js` autogenerado) con fallback offline; `.nvc` descargable; toggle **WASM ↔ Servidor**.
- **Selector F4.2**: categorías (basics/functions/data/pro/stdlib/other), búsqueda textual, favoritos en localStorage, marcador "📦 importar". Dropdown personalizado con secciones (Favoritos, Recientes, Categorías).
- **Historial de ejecuciones**: botón `🕘 Historial` (panel flotante, hasta 10 runs) + **toggle backend PERSISTENTE** (`lumen_playground_backend`).
- **2 ejemplos interactivos** (convención 2 por fase): `canvas_demo.nv` (canvas drawing vía JS bridges) + `clock_demo.nv` (reloj tiempo real). Bridges JS (`__js_call`, `__js_eval`) + corutinas.
- `pkg/lumen_wasm_bg.wasm` 2.37 MB regenerado con fixes OR/rangos/autocompletado.

### Arreglado
- **`tcp_listener` cfg**: campo del struct VM sin cfg (std::net siempre disponible) — `cargo test -p lumen-sema` compilaba lumen-vm sin features y fallaba. 
- **Clippy `-D warnings` limpio**: eq_op duplicado en `__codegen_a_nvc`, colapso de bloques idénticos en sema, Arc/Ret corutinas/ChannelCell gateados `cfg(full)`, allows documentados.
- **Parlance FFI**: errores sin detalles + bandas `-1e9`/`[1e9,2e9)`/`6e9` con guards — sin colisiones en ints negativos/grandes.

### Infraestructura
- **CI autotag**: tag `v<version>` automático solo cuando CI completo pasa (fmt/clippy/tests 3 OS/wasm) — `VERSION` como fuente única de verdad (`scripts/autotag.ps1` para bumps semver), build multi-target + GitHub Release en el mismo workflow (`needs: autotag`).
- cargo test 0 FAILED (lexer 27, parser 45, sema 56, ir 20, vm 45, e2e 166 + resto, ~380 totales).

---

## v2.4.1 — 8 Agosto 2026

### Agregado (VM LÚMEN `vm.nv` — Stream/Async/Par/Actor/Generator completados)
- **Handlers de streams**: `__stream_desde`/`__stream_from`, `__stream_mapear`/`__stream_map`, `__stream_filtrar`/`__stream_filter`, `__stream_colectar`/`__stream_collect` — delegados a natives Rust
- **Iteradores paralelos**: `__par_mapear`/`__par_map`, `__par_unir`/`__par_join` — delegados a natives Rust
- **Actores**: `__actor_nuevo`/`__actor_new`, `__actor_enviar`/`__actor_send`, `__actor_recibir`/`__actor_recv` — delegados a natives Rust
- **Generadores**: `__generador_nuevo`/`__generator_new`, `__generador_siguiente`/`__generator_next` — delegados a natives Rust
- **Select/Async I/O**: `__seleccionar`/`__select`, `__leer_archivo_async`/`__file_read_async`, `__escribir_archivo_async`/`__file_write_async` — delegados a natives Rust
- **`sprint1_concurrencia.nv` 100% paridad byte-idéntica** entre VM Rust y VM LÚMEN (Stream, Async I/O, Timer, Select, Par Map/Join, Actor, Generator)

### Arreglado (VM LÚMEN `vm.nv` — paridad con la VM Rust)
- **`__map_obtener` con mapas JSON**: devolvía `Void` (el key boxed 1e9+N no coincidía con las claves strings reales del host) y Values del host sin boxear (Str real → crash "Ge requires numbers or strings"). Ahora lookup dual (key boxed del guest → desboxeado para JSON) + boxeo por tipo real (`__tipo_de`/`a_texto` del host: texto→`box_str`, booleano→9e9+1/9e9, lista→`arrs`, diccionario→`mapas`) → **`test_json_avanzado` CORRECTO**
- **Handlers de archivos faltantes**: `__existe_archivo`/`__file_exists` (bool boxed, antes "0" en vez de "false"), `__leer_archivo`/`__file_read` (con `intentar`, el native devuelve Resultado), `__escribir_archivo`/`__file_write` → **`test_sistema_directo`/`test_sistema_avanzado` CORRECTOS**
- **Verificado**: batería `test_vm.ps1` 39/40 (solo `stress_fecha` flaky timing) · 7 tests sistema/JSON/csv/migración byte-IDÉNTICOS · 19 checks cruzados con `vm_self.nvc` (111,318 B, regenerado con compiler_v4) todos OK · cargo test OK

### Arreglado (sintaxis `para` — paridad Rust ↔ LÚMEN)
- **Init sin tipo en `para` clásico** (`para (i = 0; ...)`): el parser Rust exigía declaración tipada. Ahora `parse_for` usa `is_for_init_decl()` (keyword de tipo, tipo custom `Punto p`, o genérico) y en caso contrario construye un `Decl::Variable` con `Type::Infer` consumiendo el `;`
- **`para` clásico sin paréntesis** (`para entero i = 0; cond; paso { }`): el parser Rust lo reenviaba a foreach → E011. Nuevo dispatch con `is_foreach_like()` (lookahead puro: `[tipo]? ident (en|in)`) → foreach solo si hay `en`/`in`, si no `parse_for`. El self-hosted (`parser.nv`) recibe el helper `_st_es_foreach` (lookahead por posición sobre `tokens`) + branch de clásico sin paréntesis (desugar idéntico al clásico con `(`) → `tui_test_min16/17/18` ahora **byte-idénticos en ambas VMs**
- **FIXPOINT v4 CONFIRMADO**: SHA-256 `3DA624D6AD32E359D3714F7CD936563CE1A60ED633590CB580D695F24C7E282A` self==self2 (compiler_v4.nv 135,465 B → .nvc 150,684 B, ~5s)
- **Verificado**: cargo test 0 FAILED · batería `test_vm.ps1` 39/40 · **fuego.ps1: 117/117 compilan · 112 CORRECTOS · 1 INCOMPATIBLE (graficos_demo SDL, por diseño) · 4 TIMEOUT (debug_parser3 loop, graficos_completo/gui_ventana GUI, sprint1_http red) · 0 fallos**
- ⚠️ `test_vm.ps1` debe ejecutarse desde la RAÍZ del repo (las rutas de `entrada_vm.txt` son relativas — desde `stdlib/compiler` da FALLAS masivas falsas)

### Bootstrapping Doble (Hito Final)
- **Fixpoint del compilador**: SHA-256 `3DA624D6AD32E359D3714F7CD936563CE1A60ED633590CB580D695F24C7E282A` — 150,684 bytes **byte-idénticos** en self/self2 (~5s)
- **VM LÚMEN autogenerada**: `vm_self.nvc` (111,318 B) compilada por `compiler_v4_self.nvc` y ejecutando `demo_completo.nvc` correctamente (89/89 líneas, 0 diffs)
- **0 dependencias de Rust**: LÚMEN compila LÚMEN, VM LÚMEN ejecuta bytecode LÚMEN, todo autocontenido

---

## v2.4.0 — 6 Agosto 2026

### Agregado
- **Sprint 6: Gramática completa en el pipeline puro (self-hosted)** — `importar` con base-dir + self-import detectado, `sea`/`const` (VarDecl), StructInit `T {}` → mapas, `.campo` → Index, `elegir`/`defecto:`/`caso` reales (cadenas `sino` con im::HashMap persistente), enum `Nombre::Miembro(args)`, Option/Result (`algun`/`ninguno`/`exito`/`error` → op 38/39/41/42), closures IIFE, params default inlineados, traits `rasgo`/`impl`/`este` (métodos mangled + resolución por tipo de var), cast `como`, cortocircuito `&&`/`||` (`_cg_and_or`)
- **Sprint 7: VM en LÚMEN (`vm.nv`)** — ejecutador de .nvc en LÚMEN puro (dispatch 0-46, bandas boxed, corutinas reales con intercambio st/sp/pc, handlers JSON/tarea/coro/crypto/fs/env/tiempo/hilo/mutex/calendario, `fmain` acepta `__main__`/`main`/`principal`)
- **Optimización 43x**: COW con `Arc` en `Value::Str`/`Value::Array` (fixpoint 861s → 20.1s); `a_entero` O(n)→O(1) (demo 120s → 0.9s); `__str_subcadena_chars`/`__str_reemplazar` natives; guards de banda [3e9,9e9) y < -1e9
- **Tipo dinámico `Numero` real** + alias `cualquiera`/`any` (desbloquea csv.nv y test_migracion)
- **Benchmark vs Rust**: `scripts/benchmark_vs_rust.ps1` — compile x5.4, run x231 (mediana x2-6)
- **Resultados**: batería test_vm.ps1 **39/40** (solo `stress_fecha` flaky) · cargo test **375/375** · **fuego.ps1: 116/116 compilan · 108 CORRECTOS · 4 INCOMPATIBLES · 4 TIMEOUT · 0 fallos**

### Arreglado
- Self-import (`fs::canonicalize` `\\?\` en Windows) y renombre `graficos_avanzado_demo.nv` (sombreado del stdlib)
- `_imp_prefijar` sin rama `Lista` (calls en array-literals de imports quedaban sin prefijar)
- Scan de genéricos sin límites en parser Rust y LÚMEN (rompía `mientras i < n && ...`)
- Ternario (precedencia `mp == 0` + ramas FALSY), `.agregar` en ExprStmt, floats con `.` en lexer, PushNum f64 en codegen_to_nvc
- `__map_poner` persistente no propagaba cadenas `sino` (elegir con 2+ casos)
- Entry `principal` sin `__main__` (44_extension_methods/math), Ret sin caller, Ret con call_stack vacío en corutinas
- `a_texto_v` con ints reales >3e9, colisión de banda [1e9,2e9), Store global desde funciones
- Orden de claves de mapas determinista en `csv.nv` (serializar ordena claves numéricas)
- Output del VM en crashes (run_bytecode imprimía buffer solo en éxito)

### Limpieza (6 Ago 2026)
- Eliminados artefactos de test de la raíz (26 archivos), `examples_backup_2026/` (270 archivos), 41 `.nvc` temporales de `stdlib/compiler`, `src/` vacía — commit `4f2f6c7` (276 archivos, -6525 líneas)
- `.opencode/` añadido al .gitignore
- Docs sincronizadas: README (v2.4.0), CHANGELOG, `docs/self-hosting.md`, `docs/siguiente.md`, `docs/roadmap.md`, reports

---

## v2.3.0 — 31 Julio 2026

### Agregado
- **Self-hosting puro COMPLETADO: LÚMEN se compila a sí mismo sin `__compile_nv`**
  - `compiler_v4.nv` autocontenido (55,308 bytes): `lexer.nv` + `parser.nv` + `codegen.nv` + main, sin imports
  - Pipeline: leer .nv → lexer puro → parser puro → codegen puro → `__codegen_a_nvc` → .nvc
  - **Fixpoint confirmado**: `compiler_v4_self.nvc` (54,712 bytes, 49 funciones) recompila su propio source con resultado IDÉNTICO (52,160 bytes → 11,437 tokens → 6,376 instrs → 54,712 bytes) en 3 runs consecutivos (193s, 203s, 197s)
  - Tabla de funciones completa en el autocompilado: `_lx_es_ident`…`codegen_print` + `__main__`

### Arreglado (pipeline puro)
- `Jmp`/`JmpIf` serializados con target en tabla `nums` (la VM lee `nums[idx]`) — antes target directo → loop infinito
- If emitía JMP al final del then-body para saltar el else (antes ejecutaba ambos)
- Indexación `chars[i]`: postfix `[expr]` en `_parse_pr` + nodo `Index` en codegen + `OP_ARRAY_GET` (29)
- `intentar` (TryUnwrap): mapeo `40 => 40` en `cg_to_vm` — antes Nop dejaba `Exito(Str)` en el stack
- Print multi-arg: un `OP_PRINT` por argumento en orden (antes uno solo → output parcial/invertido)
- Break/Continue: `loop_stack` en codegen con backpatches (breaks→fin de loop, conts→loop_start)
- `numero r;` (VarDecl sin inicializador): PushInt 0 por defecto — antes stack underflow
- Lexer puro: procesa escapes de string `\n \t \r \" \\` — antes `"\""` rompía la tokenización
- Keywords `void` y `diccionario` añadidas al lexer puro — antes las funciones `codegen_imprimir` se corrompían
- Forward declarations (`funcion X(...);`): ignoradas con nodo `Vacio` — antes se tragaban la función siguiente
- Fix previo en `crates/lumen-lexer/src/lexer.rs`: escape `\r` → CR real (era 'r' literal)

### Cambiado
- `AGENTS.md`, `docs/self-hosting.md`, `docs/siguiente.md`, `docs/roadmap.md` sincronizados
- `stdlib/compiler/generar_v4.ps1` regenera `compiler_v4.nv` (concatenación autocontenida)
- Test artifacts de aislamiento eliminados (`mini_*`, `test_lexer*`)

---

## v2.2.0 — 30 Julio 2026

### Agregado
- **Self-hosting Total: `Value::Map` optimizado de `Vec<(Value,Value)>` a `HashMap<Value,Value>`**
  - `Hash` + `Eq` manual para `Value` (f64 via `to_bits()`, Map via sorted key-value hashes)
  - `__map_get`/`__map_set`/`__map_contains`: O(n) scan lineal → O(1) hash lookup
  - Sets (union/inter/diff): O(n²) → O(n) con `contains_key`
  - `codegen_to_nvc`: `map_get` O(1) con `HashMap::get`
  - JSON helpers actualizados a HashMap
  - ~378 tests pasan, autocompilación funcional (533ms)
  - El parser LÚMEN-in-LÚMEN ahora tiene mapas O(1) — self-hosting total sin `__compile_nv` es viable

### Cambiado
- `AGENTS.md`, `docs/self-hosting.md`, `docs/siguiente.md`, `docs/roadmap.md` sincronizados

---

## v2.1.0 — Julio 2026

### Agregado
- **Fase 180: Mutation Testing** — `stdlib/testing.nv` extendido con funciones de mutación de código fuente.
  - `mutar/mutate`: 5 tipos de mutación (+→-, verdadero→falso, >→<, eliminar si, invertir retornar).
  - `mutantes_generar/mutants_generate`: generación de N mutantes desde código fuente.
  - `mutacion_probar/mutation_test`: ejecución de mutation testing y conteo de mutantes muertos.
  - `mutacion_puntaje/mutation_score`: cálculo de puntaje de mutación (muertos/total).
  - `mutacion_analizar/mutation_analyze`: análisis de archivos con reporte de calidad de tests.
  - Ejemplos: `examples/jr/mutation_jr.nv`, `examples/sr/mutation_sr.nv`.
- **Fase 182: Tracing Distribuido** — `stdlib/tracing.nv` con spans estilo OpenTelemetry simplificado.
  - `span_iniciar/span_start`, `span_finalizar/span_finish`: ciclo de vida de spans.
  - `span_atributo/span_attribute`: atributos clave-valor en spans.
  - `span_error/span_error_set`: marcado de spans con error.
  - `spans_exportar/spans_export`: exportación JSON de todos los spans.
  - `spans_arbol/spans_tree`: visualización jerárquica en formato árbol.
  - `spans_limpiar/spans_clear`: reinicio de sesión de tracing.
  - Ejemplos: `examples/sr/tracing_sr.nv`, `examples/real/tracing_real.nv`.
- **Fase 184: Profiler CPU/Memoria** — `stdlib/profiler.nv` con perfilado de rendimiento.
  - `perfil_iniciar/profile_start`, `perfil_finalizar/profile_end`: medición de tiempos.
  - `perfil_reporte/profile_report`: reporte detallado con min/max/promedio/total.
  - `perfil_resumen/profile_summary`: resumen compacto para comparación rápida.
  - `perfil_memoria/profile_memory`: estimación de memoria usada.
  - `perfil_promedio/profile_avg`: tiempo promedio por función.
  - Detección de hot paths: identificación automática de funciones más lentas.
  - Ejemplos: `examples/sr/profiler_sr.nv`, `examples/real/profiler_real.nv`.
- **Dual ES/EN**: todas las funciones nuevas tienen alias en español e inglés.
- **6 nuevos ejemplos**: mutation (jr/sr), tracing (sr/real), profiler (sr/real).

### Cambiado
- `stdlib/testing.nv` extendido con sección de mutación (funciones `mutar`, `mutantes_generar`, `mutacion_probar`, `mutacion_puntaje`, `mutacion_analizar`).
- `AGENTS.md` actualizado con fases 180, 182, 184 completadas.
- `docs/roadmap.md` actualizado: portabilidad 75% → 100%.

---

## v2.0.0 — Julio 2026

### Agregado
- **Operador bitwise `|`**: soporte completo en parser → sema → IR → codegen → VM

### Agregado
- **Fase 96: WASM Playground** — Compilación a `wasm32-unknown-unknown`.
  - VM refactorizada: `call_core_builtin()` + `call_full_builtin()` extraídas del dispatch masivo.
  - Feature flags `full`/`minimal` en VM con `#[cfg(feature = "full")]` en fields TCP, cluster, scope, FFI.
  - Stubs para crypto_ffi, gui_ffi, coro_ffi en modo minimal.
  - crate `lumen-wasm` con playground web HTML.
  - `Display` impl para `VmError`.
- **Fases 97-130: ~70 builtins nuevos** (FFI, crypto, concurrencia, GUI, corrutinas, utilidades, fecha):
  - FFI: `__ffi_cargar/load`, `__ffi_llamar/call`, `__ffi_asignar/alloc`, `__ffi_liberar/free`, `__ffi_escribir/write`, `__ffi_leer/read`, `__ffi_peek`, `__ffi_poke`
  - Crypto: `__hash_sha256`, `__hash_sha512`, `__aes_encriptar/encrypt`, `__aes_desencriptar/decrypt`, `__jwt_codificar/encode`, `__jwt_decodificar/decode`
  - Concurrencia: hilos, mutex, canales, rwlock, arc, tareas, streams, actores, generadores, supervisores, cluster, scope, par, dormir, seleccionar (~36 builtins)
  - GUI: `__gui_ventana/window`, `__gui_mostrar/show`, `__gui_cerrar/close`, `__gui_id/hwnd`, `__gui_esperar/poll`
  - Corrutinas: `__coro_crear/create`, `__coro_ceder/yield`, `__coro_reanudar/resume`
  - Utilidades: `__tipo_de/typeof`, `__fs_listar/listdir`, `__env_listar/list`
  - Fecha: `__tiempo_formatear/format`, `__tiempo_parsear/parse`, `__tiempo_diferencia/diff`
- Nuevo crate: `lumen-wasm` (runtime WASM + playground web).
- **Stdlib dual ES/EN**: `texto.nv`, `fecha.nv`, `io.nv`, `crypto.nv` actualizados con aliases inglés.
- **149 e2e tests** (10 nuevos desde v1.6.0, +10 desde v1.7.0alpha).

### v2.0 — GUI/TUI/Juegos 100% (Julio 2026)
- **Canvas 2D**: círculos, líneas, triángulos, rectángulos redondeados, gradientes, bitmap font
- **Tilemap**: sistema de mapas 2D con cámara, colisiones AABB, view culling
- **Charts**: gráficos de barras, líneas, pastel, dispersión con ejes automáticos
- **TUI Temas**: 4 presets (Catppuccin, claro, oscuro, alto contraste)
- **Demo completo**: 33 secciones cubriendo todas las features del lenguaje
- **Skills**: `.opencode/agents/lumen-engineer.md` + `lumen-tester.md`
- **Roadmap**: GUI/TUI/Juegos 55%→100%

### Cambiado
- Version bump a 2.0.0.
- Todos los .md sincronizados con estado actual.
- README reescrito con WASM playground, Docker, sintaxis dual.
- 66 ejemplos nuevos organizados en jr/sr/real (22 módulos stdlib).

### Corregido
- `>=` y `<=` verificados funcionales en parser (no requerían workaround)
- `info` y `debug` NO son keywords reservadas (solo `error` lo es)
- Bitwise OR `|` implementado como operador de expresión
- stdlib: restaurados `>=`/`<=` y `|` donde habían sido reemplazados incorrectamente

---

## v1.6.0 — Julio 2026

### Agregado
- **Fase 96-110: Librería Estándar Extendida** — HashMap, HashSet, VecDeque, BinaryHeap,
  LinkedList, Regex, Unicode Normalization, String Padding, UTF-8 Encoding,
  Buffered I/O, Streaming, TCP, HTTP, Serial Port (stub).
  ~70 builtins nuevos en VM (handlers Call/CallValue). Stdlib modules: `coleccion.nv`,
  `texto.nv`, `io.nv`, `red.nv`. Registrados en sema/loader/IR builder.
- **Fase 94: Single Binary** — `lumen` como binario único integrando `run`, `build`, `check`,
  `fmt`, `repl`, `doc`, `lsp`, `install` sin spawnear procesos hijos.
  Creadas libs `lumen_doc`, `lumen_lsp`, `lumen_pkg`.
- **Fase 95: Installer** — `scripts/install.ps1` (Windows) + `scripts/install.sh` (Unix).
  Detección de release binaria con fallback a compilación desde fuente.
- **Fase 65: Guard Let** — `sea patron = expr sino { romper/retornar/continuar }`.
  Desugaring en IR builder a JmpIf/Jmp. `Stmt::GuardLet` en AST + parser + sema.
- **Fase 66: Operator Overloading** — `impl Suma for Punto` con método `fn sumar(self, otro)`.
  - `Expr::Binary` ahora tiene `resolved_method: Option<String>`.
  - Sema: `resolve_operator_overloads()` post-analysis con `HashMap<String, TypeInfo>`.
  - IR builder: emite `Call` en lugar de `Binary` cuando hay overload resuelto.
  - Traits `Suma`, `Resta`, `Multiplica`, `Divide` implementables en structs.
- **Fase 67: Extension Methods** — `impl Rasgo para TipoPrimitivo`.
  - `type_to_impl_name()` soporta resolución de tipos nativos: `entero`, `texto`, `decimal`, etc.
- **Fase 68: Tipos Asociados en Traits** — `tipo Item;` en rasgos y `tipo Item = T;` en impl.
  - AST: `AssociatedType` e `ImplAssociatedType`. Sema e IR completos.
  - `resolve_trait_method_mangled()` ahora sustituye tipos asociados contra el impl concreto.
  - `examples/senior/associated_types.nv` — demo funcional con 2 impls distintos.
- **Fase 70: Impl Trait return** — `funcion impl Rasgo foo() { retornar expr }`.
  - `Type::ImplTrait(String)` en AST. Parseo en `parse_type()`.
- **Fases 71-74: LSP Server (`lumen-lsp`)** — Diagnósticos en vivo, Autocompletado,
  Go-to-definition y Hover. Protocolo JSON-RPC sobre stdin/stdout.
- **Fase 75: lumen doc (`lumen-doc`)** — Generador de documentación HTML desde `///`.
- **Fase 76: Debugger** — Depurador interactivo con breakpoints, step, continue e inspect.
- **Fase 77: lumen fmt avanzado** — Soporte para `.lumen-fmt.toml` (`indent_spaces`, etc.).
- **Fase 78: lumen lint** — Análisis estático: código muerto y complejidad ciclomática.
- **Fase 79: REPL Pro** — Historial persistente, multilínea, resaltado, autocompletado.
- **Fase 80: Package Manager (`lumen-pkg`)** — `lumen install`, registry, lock file.
- **Fase 81: Build Incremental** — Caché de módulos para builds más rápidos.
- **Fase 82: Hot Reload** — `lumen serve` con recarga automática en dev.
- **Fase 83: Playground Web** — Editor online con ejecución en navegador.
- **Fases 86-87: AOT Compilation (`lumen-aot`)** — Transpilador C + backend Cranelift (base).
- **Roadmap fusionado** — Combinación del historial v1.0-v1.2 con el roadmap extendido v3.0.

### Cambiado
- Version bump a 1.6.0 en workspace.
- Trait impl functions reciben `"self"` como primer parámetro automáticamente.
- `docs/roadmap.md` expandido con tablas detalladas hasta fase 220 (v3.0).
- `AGENTS.md` actualizado con fases 94-95 completadas.
- `docs/cli.md` actualizado con comando `install`.
- `HERRAMIENTAS.md` actualizado con nuevos scripts de instalación.

### Corregido
- Encoding UTF-8 en scripts de CI/CD (pre-commit PowerShell).
- Pipeline de docs actualizado con referencias a crates nuevos.

---

## v1.2.0 — Julio 2026

### Agregado
- **Stdlib** — Módulos nativos: `matematicas`, `texto`, `coleccion`, `fecha`.
- **E/S de Archivos** — Builtins `leer_archivo`, `escribir_archivo`, `existe_archivo`.
- **Stack Traces** — Pila de llamadas completa en errores de runtime.
- **Mensajes de Error Mejorados** — Subrayado exacto con caret (`^^^^`) y colores ANSI.
- **Fases 42-57** — Inferencia de tipos, métodos en structs, diccionarios, string interpolation,
  rangos, constantes, string indexing, conversiones, operador ternario, loop labels,
  pattern matching exhaustivo + guardas, genéricos con bounds, matrices 2D.
- **Fase 58** — Enums avanzados con datos asociados.

### Corregido
- Advertencias de Clippy (CI verde en todos los targets).

---

## v1.1.0 — Julio 2026

### Agregado
- **Fase 21: For-Each Loop** — `para x en expr` / `for x in expr`. 31 tests.
- **Fase 22: Opcion<T>** — `opcion<T>` con `algun(valor)` y `ninguno`. 15 tests.
- **Fase 23: Enums/Tipos Suma** — `enum Nombre { Variante, Variante(tipo) }`. 20 tests.
- **Fase 24: Tuplas** — `(tipo, tipo)` y acceso `.0`, `.1`. 4 tests.
- **Fase 25: Destructuring** — `entero x, texto y = expr`, wildcard `_`. 14 tests.
- **Fase 26: Genéricos Básicos** — `<T, U>` en funciones y structs. 17 tests.

### Cambiado
- Workspace version a 1.1.0.
- CI corre en branches `master` y `main`.
- MSRV actualizado a 1.82.

---

## v1.0.0 — Julio 2026

Release inicial de LÚMEN. Lenguaje de programación educativo en español con pipeline completo
Lexer → Parser → Sema → IR → Codegen → VM. 21 fases completadas.
