# LÚMEN

**Lenguaje de programación educativo de alto rendimiento por TriXxo**

---

## Visión

LÚMEN es un lenguaje de programación diseñado para **enseñar a programar con claridad, eficiencia y honestidad intelectual**. Su filosofía rechaza ocultar el funcionamiento interno de la computadora, eligiendo en cambio exponer los fundamentos reales de la programación mediante una sintaxis accesible y un motor de ejecución eficiente.

LÚMEN está dirigido a estudiantes, autodidactas y entornos educativos que buscan comprender profundamente:

* **Flujo de control**: cómo un programa toma decisiones y ejecuta instrucciones
* **Gestión de datos**: cómo se almacenan, transforman y transfieren los datos en memoria
* **Ejecución real**: cómo una máquina interpreta y ejecuta código

---

## Principios de diseño

### 1. Claridad sobre comodidad
Nada ocurre implícitamente. Cada operación debe ser explícita, permitiendo al programador comprender exactamente qué ejecuta la máquina.

### 2. Rendimiento real
Compilación y ejecución veloces, optimizadas incluso para equipos modestos. El aprendizaje no debe estar limitado por recursos de hardware.

### 3. Simplicidad pedagógica
Conjunto mínimo de reglas, bien definidas y coherentes. Menos conceptos, mejor comprendidos.

### 4. Determinismo absoluto
El mismo código produce el mismo resultado en cualquier contexto. Sin comportamientos aleatorios o dependientes del entorno.

### 5. Autonomía total
Sin dependencias externas, sin configuraciones complejas. Un compilador, un runtime, todo incluido.

---

## Características principales

### Lenguaje

* **Sintaxis en español** con equivalentes opcionales en inglés
* **Tipado estático explícito** que refuerza la comprensión de tipos de datos
* **Gramática minimalista** diseñada para minimizar la ambigüedad
* **Mensajes de error pedagógicos** que explican el problema y sugieren soluciones

### Infraestructura

* **Compilador propietario** escrito desde cero
* **Formato de bytecode propio** (.nvc) optimizado para lectura educativa
* **Máquina virtual ultraligera** basada en arquitectura de stack
* **Gestión de memoria predecible** con stack y heap controlado
* **CLI intuitiva** con comandos directos y respuestas claras

---

## Arquitectura del compilador

```
┌──────────────────────┐
│  Código fuente (.nv) │
└──────────┬───────────┘
           │
           ▼
     ┌─────────┐
     │  Lexer  │  ← Análisis léxico (tokens)
     └────┬────┘
          │
          ▼
    ┌──────────┐
    │  Parser  │  ← Construcción del AST
    └────┬─────┘
         │
         ▼
┌──────────────────┐
│ Análisis         │  ← Verificación de tipos y semántica
│ semántico        │
└────┬─────────────┘
     │
     ▼
┌─────────────────┐
│ Generador de IR │  ← Representación intermedia
└────┬────────────┘
     │
     ▼
┌──────────────────┐
│ Compilador de    │  ← Generación de bytecode
│ bytecode         │
└────┬─────────────┘
     │
     ▼
┌──────────────────┐
│  Bytecode (.nvc) │
└────┬─────────────┘
     │
     ▼
┌──────────────────┐
│   LÚMEN VM       │  ← Ejecución
└──────────────────┘
```

---

## Ejemplos de código

### Bucle básico
```lumen
numero contador = 0

mientras (contador < 5) {
    imprimir(contador)
    contador = contador + 1
}
```

### Función con parámetros
```lumen
funcion numero suma(numero a, numero b) {
    retornar a + b
}

numero resultado = suma(3, 7)
imprimir(resultado)
```

### Condicional
```lumen
numero edad = 18

si (edad >= 18) {
    imprimir("Eres mayor de edad")
} sino {
    imprimir("Eres menor de edad")
}
```

---

## Interfaz de línea de comandos

### Ejecutar código fuente directamente FUNCIONA
```bash
lumen run programa.nv
```

### Compilar a bytecode (EN PRUEBAS)
```bash
lumen build programa.nv
# Genera: programa.nvc
```

### Ejecutar bytecode compilado PROXIMAMENTE
```bash
lumen run programa.nvc
```

### Verificar sintaxis y semántica sin ejecutar PROXIMAMENTE
```bash
lumen check programa.nv
```

### Mostrar bytecode generado (modo educativo) PROXIMAMENTE
```bash
lumen disasm programa.nvc
```

---

## Enfoque pedagógico

LÚMEN ha sido diseñado específicamente como herramienta de aprendizaje:

### Mensajes de error educativos
Los errores no solo indican qué falló, sino **por qué** y **cómo corregirlo**:

```
Error de tipo en línea 5, columna 12:
  No puedes asignar un texto a una variable de tipo 'numero'

  numero edad = "veinte"
                ^^^^^^^^

  Sugerencia: Usa un valor numérico como 20 en lugar de texto
```

### Ejecución determinista
La VM detiene la ejecución ante cualquier error, reforzando el aprendizaje mediante corrección inmediata.

### Transparencia técnica
La VM y el bytecode están documentados y pueden estudiarse como parte del currículum, eliminando la "magia" de la ejecución.

### Sin abstracciones innecesarias
El lenguaje evita construcciones que oculten el funcionamiento real, privilegiando siempre la comprensión sobre la conveniencia.

---

## Especificación técnica (v1.0)

### Incluido en la versión inicial

* **Tipos de datos básicos**: numero, texto, booleano
* **Variables**: declaración, asignación, scope
* **Operadores**: aritméticos, lógicos, comparación
* **Estructuras de control**: si/sino, mientras, para
* **Funciones**: declaración, parámetros, retorno
* **Entrada/Salida**: leer(), imprimir()
* **Comentarios**: // línea y /* bloque */

### Explícitamente excluido (decisión pedagógica, en v1.0)

* **POO**: se introduce complejidad conceptual prematura
* **Concurrencia**: requiere modelo mental avanzado
* **Garbage Collection**: se prefiere gestión manual educativa
* **Sistema de módulos**: fase posterior (ver Fase 10 del roadmap)
* **Librerías externas**: evita dependencias y complejidad

Estas exclusiones son **intencionales** en v1.0 y responden a la filosofía pedagógica de LÚMEN: dominar lo fundamental antes de lo avanzado. El roadmap detallado a continuación describe cuándo y cómo se incorporan estas capacidades en versiones posteriores, sin sacrificar madurez de ingeniería.

---

## Decisiones técnicas fundacionales (deben tomarse antes de escribir código)

Antes de empezar desde cero, hay decisiones de arquitectura que determinan todo lo demás. Se documentan aquí para que queden fijadas y no se re-discutan a mitad de la implementación.

### Lenguaje de implementación del compilador/VM

Tres opciones razonables, con trade-offs:

| Opción | Ventajas | Desventajas |
|---|---|---|
| **Rust** (recomendado) | Memoria segura sin GC, rendimiento nativo, excelente para parsers/VMs, `cargo` da build system + testing + benchmarking gratis, gran ecosistema para escribir intérpretes (`logos`, `criterion`) | Curva de aprendizaje si el equipo no lo conoce |
| **C** | Transparencia total (coherente con la filosofía "sin magia"), control absoluto de memoria, ideal si se quiere que el propio compilador sea material didáctico | Gestión manual de memoria es fuente de bugs, sin red de seguridad, build system más manual |
| **C++** | Punto medio, RAII ayuda con gestión de recursos | Complejidad del lenguaje puede contradecir la filosofía de simplicidad del proyecto |

**Recomendación**: Rust para el compilador y la VM. Justificación: cumple "rendimiento real" y "determinismo absoluto" sin el riesgo de UB de C/C++, y su tooling (`cargo test`, `cargo bench`, `cargo clippy`) acelera enormemente llegar a un producto maduro. Si la prioridad pedagógica es que los propios estudiantes lean el código fuente del compilador como parte del curso, C es defendible por transparencia — pero exige mucha más disciplina de testing.

### Formato de número por defecto
Decidir ahora: ¿`numero` es entero de 64 bits, float de doble precisión (IEEE-754), o un tipo unificado que decide en tiempo de ejecución? Recomendación: separar en el futuro `entero` y `decimal` (Fase 9), pero en v1.0 usar `f64` internamente para simplicidad, documentando explícitamente el redondeo — la transparencia sobre la representación es parte de la filosofía del lenguaje.

### Estrategia de gestión de memoria
Sin GC (decisión ya tomada en el documento). Se necesita definir: ¿stack-only con tamaños fijos en v1.0, o heap manual desde el inicio para `texto` (strings de tamaño variable)? Recomendación: stack para tipos primitivos, un heap simple de arena (bump allocator) para `texto` y, más adelante, arrays/registros, liberado determinísticamente al salir de scope (similar a regiones). Esto se explica en detalle en la Fase 7.

### Estrategia de versionado y repos
- **SemVer estricto** (MAJOR.MINOR.PATCH) desde el primer commit.
- Un solo monorepo: `/lexer`, `/parser`, `/sema`, `/ir`, `/codegen`, `/vm`, `/cli`, `/docs`, `/tests`, `/examples`.
- Bytecode `.nvc` versionado con un número de versión en el header, para que la VM rechace bytecode de versiones incompatibles con un error explicativo (coherente con "errores pedagógicos").

---

## Roadmap extremadamente detallado (desde cero hasta lenguaje maduro)

Este roadmap sustituye y expande la hoja de ruta anterior. Está organizado en **17 fases secuenciales**, cada una con objetivo, tareas técnicas concretas, entregables verificables, criterios de éxito (tests que deben pasar) y una estimación de esfuerzo asumiendo **una persona trabajando part-time** (ajustar según dedicación real). Cada fase asume que las anteriores están terminadas y con tests en verde — no se avanza con deuda técnica oculta.

### Fase 0 — Cimientos del proyecto
**Objetivo**: tener el esqueleto del repositorio y las herramientas de desarrollo antes de escribir una sola línea de lenguaje.

Tareas:
- Elegir e instalar el lenguaje de implementación (Rust recomendado) y fijar versión mínima soportada (MSRV).
- Crear estructura de monorepo con workspace de Cargo (`lumen-lexer`, `lumen-parser`, `lumen-sema`, `lumen-ir`, `lumen-codegen`, `lumen-vm`, `lumen-cli` como crates separados).
- Configurar CI (GitHub Actions): build en Linux/macOS/Windows, `cargo test`, `cargo clippy --deny warnings`, `cargo fmt --check`.
- Definir convención de commits (Conventional Commits) y changelog automático.
- Crear `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, plantillas de issues/PR.
- Configurar `cargo bench` con `criterion` para benchmarking desde el día 1 (rendimiento se mide desde el inicio, no se añade al final).

Entregables: repo compilable con "hello world" en Rust, pipeline de CI en verde, documentación de arquitectura inicial (`ARCHITECTURE.md`).

Criterio de éxito: un colaborador nuevo puede clonar, compilar y correr los tests en menos de 10 minutos siguiendo el README.

Esfuerzo estimado: 3–5 días.

---

### Fase 1 — Especificación formal del lenguaje
**Objetivo**: escribir la gramática completa antes de programar el parser, para evitar rediseños costosos.

Tareas:
- Escribir la gramática en **EBNF** completa para v1.0 (expresiones, sentencias, declaraciones, precedencia y asociatividad de operadores).
- Especificar formalmente el sistema de tipos: reglas de compatibilidad, coerciones permitidas (¿existe coerción implícita numero↔texto? Recomendación: **no**, coherente con "nada ocurre implícitamente").
- Definir reglas de *scoping* (léxico, con sombreado de variables permitido o no — decidir y documentar).
- Especificar semántica de corto-circuito para `&&`/`||`.
- Definir el conjunto completo de errores léxicos, sintácticos y semánticos con códigos únicos (`E001`, `E002`...) para que cada error sea referenciable y documentado.
- Escribir un documento de "semántica operacional" informal: qué hace cada construcción paso a paso.

Entregables: `docs/spec/grammar.ebnf`, `docs/spec/type-system.md`, `docs/spec/error-codes.md`.

Criterio de éxito: la especificación cubre el 100% de los ejemplos de la sección "Especificación técnica" sin ambigüedades; revisada por al menos un tercero.

Esfuerzo estimado: 1–2 semanas.

---

### Fase 2 — Lexer (análisis léxico)
**Objetivo**: convertir código fuente `.nv` en una secuencia de tokens robusta.

Tareas:
- Definir el enum de `Token` (palabras clave, identificadores, literales numéricos/texto/booleanos, operadores, delimitadores, comentarios).
- Implementar el lexer manualmente (recomendado, para transparencia pedagógica) o con `logos` si se prioriza velocidad de desarrollo.
- Manejo de posiciones: cada token lleva línea y columna exactas, indispensable para los mensajes de error pedagógicos.
- Manejo de errores léxicos recuperables: el lexer no debe abortar en el primer carácter inválido; debe reportar todos los errores léxicos de una pasada.
- Soporte para comentarios de línea (`//`) y bloque (`/* */`, con anidamiento decidido explícitamente — recomendación: no anidados, más simple).
- Normalización de literales de texto (escapes: `\n`, `\t`, `\"`, `\\`).

Entregables: crate `lumen-lexer` con API `tokenize(source: &str) -> Vec<Token>` o iterador de tokens.

Criterio de éxito (tests):
- Suite de +150 casos unitarios cubriendo cada tipo de token.
- Fuzzing básico (entrada aleatoria de bytes) sin panics.
- Benchmark: tokenizar 100k líneas en <100ms en hardware modesto.

Esfuerzo estimado: 1–2 semanas.

---

### Fase 3 — Parser y AST
**Objetivo**: construir un árbol de sintaxis abstracta (AST) fiel a la gramática de la Fase 1.

Tareas:
- Definir los tipos del AST (`Expr`, `Stmt`, `Decl`, `Program`) como enums de Rust con posiciones de origen embebidas.
- Implementar parser recursivo descendente con **Pratt parsing** para expresiones (maneja precedencia de operadores de forma limpia y es fácil de explicar pedagógicamente).
- Implementar **recuperación de errores**: ante un error sintáctico, el parser debe sincronizar (ej. saltar hasta el siguiente `;` o `}`) y seguir reportando más errores en la misma pasada, en vez de detenerse en el primero.
- Serialización del AST a una representación textual/JSON para debugging y para el futuro modo `lumen disasm`/`--dump-ast`.
- Tests de "pretty-printing": el AST debe poder reconstruirse a código fuente equivalente (round-trip), útil para verificación y para futuras herramientas de formateo (`lumen fmt`).

Entregables: crate `lumen-parser`, comando interno `--dump-ast` en la CLI de desarrollo.

Criterio de éxito:
- Suite de tests con programas válidos e inválidos (snapshot testing del AST).
- Todos los ejemplos de la sección "Ejemplos de código" del README parsean correctamente.
- Errores sintácticos reportan línea/columna exactas y un mensaje pedagógico (no solo "unexpected token").

Esfuerzo estimado: 2–3 semanas.

---

### Fase 4 — Análisis semántico
**Objetivo**: validar que el programa tiene sentido más allá de la sintaxis: tipos, scopes, declaraciones.

Tareas:
- Implementar tabla de símbolos con scopes anidados (bloque, función, global).
- Chequeo de tipos estático: cada expresión recibe un tipo; se valida en asignaciones, llamadas a función, operadores, condiciones de `si`/`mientras` (deben ser `booleano`).
- Detección de: variables no declaradas, redeclaración en el mismo scope, funciones llamadas con número/tipo incorrecto de argumentos, `retornar` con tipo incompatible con la firma de la función, código inalcanzable después de `retornar` (warning pedagógico, no error).
- Generar el **AST tipado** (cada nodo anotado con su tipo resuelto) que alimentará al generador de IR.
- Motor de mensajes de error: centralizar todos los errores semánticos con su código (`E0xx`), plantilla de mensaje, y función de sugerencia (como en el ejemplo del README).

Entregables: crate `lumen-sema`, catálogo completo de errores semánticos con snippet de código + sugerencia para cada uno.

Criterio de éxito:
- Suite de tests: un caso positivo y al menos un caso negativo por cada regla semántica documentada en la Fase 1.
- 100% de los errores semánticos tienen mensaje pedagógico revisado por alguien no técnico (probar comprensibilidad real).

Esfuerzo estimado: 2–3 semanas.

---

### Fase 5 — Representación intermedia (IR)
**Objetivo**: desacoplar el frontend (AST tipado) del backend (bytecode) mediante una IR simple, para facilitar optimizaciones y futuros backends.

Tareas:
- Diseñar una IR de bajo nivel tipo "three-address code" o basada en bloques básicos (recomendado: three-address code por simplicidad pedagógica).
- Implementar el "lowering" de AST tipado → IR.
- Implementar un **CFG (grafo de flujo de control)** explícito por función, base para optimizaciones futuras y para el futuro visualizador educativo (Fase 14).
- Pase de optimización básica desde el día 1 (aunque v1.0 no lo exponga en CLI todavía): constant folding (`2 + 3` → `5` en tiempo de compilación) y eliminación de código muerto trivial.

Entregables: crate `lumen-ir`, dump textual de la IR (`--dump-ir`) para depuración y para material didáctico.

Criterio de éxito: la IR generada para los ejemplos del README es legible por un humano y corresponde exactamente a la semántica del AST original (test de equivalencia ejecutando ambos "intérpretes de IR" de referencia y comparando resultados).

Esfuerzo estimado: 2 semanas.

---

### Fase 6 — Generador de bytecode y formato `.nvc`
**Objetivo**: compilar la IR a un bytecode compacto, versionado y ejecutable por la VM.

Tareas:
- Diseñar el set de instrucciones de la VM stack-based (ej. `PUSH_NUM`, `PUSH_STR`, `ADD`, `SUB`, `CMP_LT`, `JMP`, `JMP_IF_FALSE`, `CALL`, `RET`, `LOAD_LOCAL`, `STORE_LOCAL`, `PRINT`, `HALT`). Documentar cada opcode con su efecto sobre el stack (tabla formal, entrada/salida).
- Definir el **formato binario `.nvc`**: header con magic number, versión de bytecode, tabla de constantes (strings, números), sección de código. Todo con especificación byte a byte documentada (`docs/spec/bytecode-format.md`) — esto es clave para la "transparencia técnica" prometida.
- Implementar el encoder (IR → bytes) y el decoder (bytes → estructura en memoria) simétricos y con tests de round-trip.
- Implementar `lumen build programa.nv` generando el `.nvc` real (mover de "EN PRUEBAS" a completado).
- Implementar el desensamblador (`lumen disasm`) que lee `.nvc` y muestra las instrucciones en formato legible con offsets — mueve este comando de "PROXIMAMENTE" a completado, y sirve directamente al objetivo pedagógico de "eliminar la magia".

Entregables: crate `lumen-codegen`, especificación de `docs/spec/bytecode-format.md`, comando `lumen build` y `lumen disasm` funcionales.

Criterio de éxito: todo bytecode generado se puede desensamblar y el resultado coincide 1:1 con la especificación; test de fuzzing sobre el decoder para asegurar que bytecode corrupto produce un error controlado, nunca un crash.

Esfuerzo estimado: 2–3 semanas.

---

### Fase 7 — Máquina virtual (VM)
**Objetivo**: ejecutar el bytecode de forma correcta, rápida y con gestión de memoria predecible.

Tareas:
- Implementar el bucle principal de la VM (fetch-decode-execute) sobre un stack de valores.
- Modelo de memoria: **stack de llamadas** (frames con locales y dirección de retorno) + **heap de arena** simple para `texto` y futuros tipos compuestos, con liberación determinística al cerrar el frame que los creó (documentar explícitamente las reglas de vida de estos valores — esto es enseñable y evita la necesidad de GC).
- Representación de valores: usar un `enum Value { Numero(f64), Texto(Rc<str>), Booleano(bool) }` o, para máximo rendimiento, un NaN-boxing si se quiere llevar el rendimiento al límite (opcional, posponer a Fase 13).
- Manejo de errores de ejecución (división por cero, índice fuera de rango en el futuro, desbordamiento de stack) con el mismo estilo de mensaje pedagógico que los errores de compilación, incluyendo el "stack trace" de llamadas de LÚMEN (no de Rust).
- Implementar `lumen run programa.nvc` (bytecode precompilado) — mueve este comando de "PROXIMAMENTE" a completado.
- Modo "run directo" (`lumen run programa.nv`) internamente encadena lexer→parser→sema→ir→codegen→vm en memoria, sin tocar disco (ya funciona hoy; formalizar como pipeline reusable).

Entregables: crate `lumen-vm`, especificación de la VM (`docs/spec/vm-spec.md`) con la tabla de opcodes y el modelo de memoria documentado.

Criterio de éxito:
- Suite de tests de ejecución end-to-end (programa fuente → salida esperada) para cada construcción del lenguaje.
- Benchmark de referencia (ej. Fibonacci recursivo, bucles de 10M iteraciones) documentado y repetible en CI para detectar regresiones de rendimiento.
- Cero panics de Rust ante cualquier entrada — todo error debe ser un `Result` manejado y convertido en mensaje de LÚMEN.

Esfuerzo estimado: 3–4 semanas.

---

### Fase 8 — CLI completa y experiencia de desarrollador
**Objetivo**: que las 5 subcomandos del README existan y sean consistentes.

Tareas:
- Consolidar `lumen run`, `lumen build`, `lumen check` (parsea + análisis semántico sin ejecutar, reporta todos los errores encontrados), `lumen disasm`, y añadir `lumen fmt` (formateador canónico, apoyándose en el round-trip del AST de la Fase 3) y `lumen repl` (bucle interactivo, valioso pedagógicamente para experimentar).
- Flags estándar: `--version`, `--help`, `--verbose` (adelanta parte de la v1.5 planeada), `--dump-ast`, `--dump-ir` para fines educativos.
- Códigos de salida consistentes (0 éxito, 1 error de usuario, 2 error interno) para integrarse bien con scripts y CI de los propios estudiantes.
- Empaquetado: binarios precompilados para Linux/macOS/Windows sin dependencias externas (coherente con "autonomía total"), publicados en GitHub Releases.

Entregables: `lumen-cli` completa, binarios de release, `docs/cli-reference.md`.

Criterio de éxito: un usuario nuevo instala el binario, corre los 3 ejemplos del README y el REPL sin fricción, en las tres plataformas soportadas.

Esfuerzo estimado: 1–2 semanas.

---

### Fase 9 — Sistema de tipos maduro y tipos compuestos
**Objetivo**: llevar el lenguaje más allá de lo mínimo pedagógico hacia expresividad real, manteniendo simplicidad.

Tareas:
- Separar `numero` en `entero` (i64) y `decimal` (f64) con reglas explícitas de conversión (`convertir_a_decimal(x)`, sin coerción implícita).
- Añadir **arrays** de tamaño fijo y dinámico (`arreglo[numero] de 10`), con chequeo de límites en tiempo de ejecución y mensajes de error pedagógicos ante acceso fuera de rango.
- Añadir **registros** (records/structs simples, sin métodos — mantiene la exclusión de POO pero permite agrupar datos): `registro Punto { numero x, numero y }`.
- Actualizar el análisis semántico, la IR y la VM para soportar estos tipos (impacto transversal, por eso se hace después de tener las fases 2–8 estables).
- Actualizar la especificación de errores con los nuevos casos (índice fuera de rango, campo inexistente en registro, etc.).

Entregables: especificación actualizada, soporte completo en el pipeline, ejemplos nuevos en `/examples`.

Criterio de éxito: reescribir al menos 3 programas de ejemplo "reales" (ej. ordenar una lista, calcular estadísticas básicas, un pequeño juego de texto) usando arrays y registros, todos con tests.

Esfuerzo estimado: 3–4 semanas.

---

### Fase 10 — Sistema de módulos
**Objetivo**: permitir dividir programas grandes en varios archivos, requisito de un lenguaje "maduro".

Tareas:
- Diseñar sintaxis de importación explícita (ej. `importar "utilidades.nv" como utilidades`) — nada implícito, coherente con la filosofía.
- Resolución de módulos: rutas relativas al archivo, detección de importaciones circulares con error pedagógico claro.
- Namespacing: cómo se referencian símbolos importados (`utilidades.suma(...)`), evitando colisiones de nombres.
- Compilación separada opcional: cada módulo puede compilarse a su propio `.nvc` y enlazarse, o compilarse todo junto en una sola unidad (decidir y documentar la estrategia; recomendación para v1.0 de módulos: compilación unificada, más simple; compilación separada se puede posponer a v3.0 tal como marcaba el roadmap original).

Entregables: especificación de módulos, soporte en parser/sema/codegen, ejemplos multi-archivo.

Criterio de éxito: un programa de ejemplo dividido en 3+ archivos compila y ejecuta correctamente, y los errores de import circular o módulo no encontrado son claros.

Esfuerzo estimado: 2–3 semanas.

---

### Fase 11 — Robustez de errores y experiencia pedagógica end-to-end
**Objetivo**: pulir la promesa central del proyecto — que ningún error deje al estudiante sin saber qué hacer.

Tareas:
- Auditoría completa: cada uno de los códigos de error (`Exxx`) definidos en la Fase 1 debe tener mensaje, ejemplo y sugerencia verificados.
- Sistema de "notas adicionales" en errores complejos (ej. cuando el error involucra dos ubicaciones en el código, mostrar ambas con contexto).
- Pruebas de usabilidad reales: sentar a 3–5 estudiantes principiantes frente a errores comunes y medir si entienden el problema sin ayuda externa; iterar mensajes según resultados.
- Colores y formato en terminal (con opción `--no-color` para entornos que no lo soportan, coherente con autonomía y accesibilidad).

Entregables: `docs/spec/error-codes.md` como catálogo final y estable, reporte de pruebas de usabilidad.

Criterio de éxito: ≥80% de los estudiantes en las pruebas de usabilidad resuelven el error sin ayuda externa a partir solo del mensaje.

Esfuerzo estimado: 2 semanas (más tiempo de coordinación para las pruebas con usuarios).

---

### Fase 12 — Testing y aseguramiento de calidad de nivel producción
**Objetivo**: que el compilador sea confiable como cualquier lenguaje "adulto", no un prototipo.

Tareas:
- Suite de **tests de conformidad del lenguaje** ("test suite" al estilo de los que usan lenguajes maduros): cientos de programas `.nv` de entrada con salida esperada exacta, corridos en CI en cada PR.
- **Fuzzing continuo** del lexer, parser y decoder de bytecode (ej. `cargo-fuzz`), corriendo automáticamente y reportando cualquier panic como bug bloqueante.
- **Property-based testing** (ej. `proptest`) para invariantes como "el AST reconstruido a fuente y vuelto a parsear produce el mismo AST".
- Cobertura de código medida (`tarpaulin` o similar) con umbral mínimo exigido en CI (ej. 85%).
- Tests de regresión: cada bug reportado por la comunidad se convierte primero en un test que falla, luego se corrige.

Entregables: pipeline de CI con fuzzing y cobertura, dashboard de cobertura público.

Criterio de éxito: cero panics conocidos ante entrada arbitraria; cobertura ≥85% sostenida.

Esfuerzo estimado: 2–3 semanas iniciales + mantenimiento continuo.

---

### Fase 13 — Rendimiento y optimización
**Objetivo**: cumplir de verdad la promesa de "alto rendimiento", con datos, no solo con la intención.

Tareas:
- Benchmarks de referencia contra intérpretes conocidos de complejidad similar (ej. comparar tiempos con un intérprete Python puro para el mismo algoritmo, como referencia relativa, no como competencia real).
- Optimizaciones en la IR: propagación de constantes, eliminación de subexpresiones comunes triviales, inlining de funciones pequeñas (opcional y documentado, para no romper la promesa de "nada oculto" — debe poder desactivarse con `--no-optimize` para fines didácticos).
- Optimizaciones en la VM: dispatch de instrucciones eficiente (computed goto o jump table si el lenguaje de implementación lo permite), representación de valores optimizada (NaN-boxing si se justifica con benchmarks).
- Perfilado de memoria: asegurar que el heap de arena no tiene fugas ni fragmentación excesiva en programas largos.
- Documentar cada optimización en `docs/performance.md`, explicando qué hace y por qué, manteniendo el espíritu educativo incluso en esta fase técnica.

Entregables: suite de benchmarks reproducible, reporte de rendimiento versionado (se re-corre en cada release para detectar regresiones).

Criterio de éxito: mejoras medibles y documentadas release a release, sin regresiones no explicadas.

Esfuerzo estimado: 3–4 semanas.

---

### Fase 14 — Herramientas educativas avanzadas
**Objetivo**: cumplir la visión de v2.0 del roadmap original — hacer visible lo invisible.

Tareas:
- **Modo de ejecución paso a paso**: `lumen run --debug` que pausa instrucción por instrucción, mostrando el stack, las variables locales y la línea fuente correspondiente.
- **Visualización del stack y memoria**: usar el CFG generado en la Fase 5 y el estado de la VM para renderizar (en terminal con TUI, o exportando a un formato que un futuro playground web pueda pintar) el estado de ejecución.
- **Debugger educativo** con breakpoints (`punto_de_ruptura` en el código o por línea vía CLI).
- **Editor con sintaxis resaltada**: definiciones de gramática TextMate/Tree-sitter para integración con editores populares (VS Code como prioridad, dado su uso extendido en entornos educativos).

Entregables: modo debug en la VM, extensión de VS Code, gramática Tree-sitter publicada.

Criterio de éxito: un instructor puede proyectar la ejecución paso a paso de un bucle en clase y los estudiantes ven exactamente cómo cambia el stack.

Esfuerzo estimado: 4–6 semanas.

---

### Fase 15 — Documentación y materiales didácticos completos
**Objetivo**: que el lenguaje pueda enseñarse sin depender del autor original.

Tareas:
- Manual del lenguaje completo (tutorial progresivo, de variables a funciones, con ejercicios al final de cada sección).
- Referencia técnica completa (cada palabra clave, cada operador, cada mensaje de error, indexado y buscable).
- Guía "cómo está hecho LÚMEN" narrando la arquitectura del compilador para quien quiera estudiarlo como currículum de compiladores.
- Banco de ejercicios graduados por dificultad, con soluciones.
- Guía de contribución técnica detallada para que terceros puedan añadir features siguiendo la filosofía del proyecto.

Entregables: sitio de documentación (estático, sin dependencias de servicios de terceros, coherente con "autonomía total"), publicado junto al repositorio.

Criterio de éxito: un instructor externo, sin contacto previo con el autor, puede dar un curso completo de un semestre usando solo esta documentación.

Esfuerzo estimado: 4–6 semanas (se puede paralelizar con otras fases).

---

### Fase 16 — Playground web y ecosistema
**Objetivo**: bajar la fricción de entrada a cero — probar LÚMEN sin instalar nada.

Tareas:
- Compilar la VM y el pipeline completo a **WebAssembly** (Rust lo permite de forma directa) para correr LÚMEN en el navegador.
- Playground interactivo: editor + botón "ejecutar" + panel de errores pedagógicos + (si la Fase 14 ya existe) visualización del stack, todo client-side sin backend, respetando "autonomía total" y minimizando costos de hosting.
- Explorar (como trabajo opcional y claramente delimitado, para no contradecir "sin librerías externas" del lenguaje en sí) un gestor de paquetes simple para compartir módulos de la comunidad, si la demanda lo justifica — se recomienda evaluarlo solo después de v2.0, no antes.

Entregables: playground desplegado, build de WASM en CI.

Criterio de éxito: cualquier persona con un navegador ejecuta su primer programa LÚMEN en menos de 30 segundos desde que entra al sitio.

Esfuerzo estimado: 3–5 semanas.

---

### Fase 17 — Estabilización v1.0 "madura" y mantenimiento continuo
**Objetivo**: declarar una versión 1.0 real: estable, documentada, con compromiso de compatibilidad.

Tareas:
- Congelar la gramática y el formato de bytecode para v1.0 (cambios futuros requieren bump de versión mayor y migración documentada).
- Política pública de *deprecation* (cómo y con cuánto aviso se retiran features).
- Auditoría de seguridad básica (aunque el lenguaje no se destine a producción crítica, un decoder de bytecode robusto ante archivos corruptos o maliciosos es buena práctica).
- Publicación formal: release notes completas, anuncio, checklist de "lenguaje maduro" (spec formal ✅, tests de conformidad ✅, tooling completo ✅, documentación completa ✅, benchmarks públicos ✅, comunidad con guía de contribución ✅).
- Definir cadencia de releases futuros (ej. minor cada trimestre, patch según necesidad).

Entregables: LÚMEN v1.0.0 etiquetado, anunciado, con todos los checkboxes de madurez cumplidos.

Criterio de éxito: el proyecto cumple, con evidencia verificable (tests, docs, benchmarks públicos), todos los puntos de la lista de madurez — no es una autodeclaración sin respaldo.

Esfuerzo estimado: 2 semanas de cierre + mantenimiento indefinido.

---

## Resumen de esfuerzo total estimado

| Fase | Duración estimada |
|---|---|
| 0. Cimientos | 3–5 días |
| 1. Especificación | 1–2 semanas |
| 2. Lexer | 1–2 semanas |
| 3. Parser/AST | 2–3 semanas |
| 4. Semántica | 2–3 semanas |
| 5. IR | 2 semanas |
| 6. Bytecode | 2–3 semanas |
| 7. VM | 3–4 semanas |
| 8. CLI | 1–2 semanas |
| 9. Tipos compuestos | 3–4 semanas |
| 10. Módulos | 2–3 semanas |
| 11. Errores pedagógicos | 2 semanas |
| 12. QA/Testing | 2–3 semanas + continuo |
| 13. Rendimiento | 3–4 semanas |
| 14. Herramientas educativas | 4–6 semanas |
| 15. Documentación | 4–6 semanas |
| 16. Playground web | 3–5 semanas |
| 17. Estabilización v1.0 | 2 semanas |
| **Total (part-time, una persona)** | **≈ 8–11 meses** hasta un v1.0 verdaderamente maduro |

*Nota: las fases 12 y 15 pueden y deben paralelizarse con las fases técnicas contiguas para acortar el calendario real. Si hay más de un colaborador, el frontend (fases 2–5) y el backend (fases 6–8) pueden avanzar en paralelo una vez cerrada la Fase 1.*

---

## Checklist de "lenguaje maduro" (criterio final de aceptación del proyecto)

- [ ] Gramática formal completa y congelada para v1.0
- [ ] Sistema de tipos documentado sin ambigüedades
- [ ] Lexer y parser con recuperación de errores (reportan múltiples errores por pasada)
- [ ] Análisis semántico completo con catálogo de errores pedagógicos verificado con usuarios reales
- [ ] IR con al menos optimizaciones básicas (constant folding, dead code elimination)
- [ ] Formato de bytecode versionado y documentado byte a byte
- [ ] VM con modelo de memoria documentado, sin GC, sin panics ante entrada arbitraria
- [ ] CLI completa: `run`, `build`, `check`, `disasm`, `fmt`, `repl`
- [ ] Tipos compuestos (arrays, registros) y separación entero/decimal
- [ ] Sistema de módulos funcional
- [ ] Suite de tests de conformidad + fuzzing continuo + cobertura ≥85%
- [ ] Benchmarks públicos y reproducibles, sin regresiones no explicadas
- [ ] Modo debug paso a paso y resaltado de sintaxis en al menos un editor popular
- [ ] Documentación completa: manual, referencia, guía de arquitectura, ejercicios
- [ ] Playground web funcional sin backend
- [ ] Política de versionado y deprecation públicada
- [ ] v1.0.0 etiquetada con release notes completas

---

## Licencia y derechos de autor

**MIT License**

Copyright © 2026 **Omar Palomares Velasco** — TriXxo Corp

Se concede permiso para usar, copiar, modificar y distribuir este software con fines educativos y comerciales, bajo los términos de la licencia MIT.

### Declaración de autoría

Este proyecto es obra original de **Omar Palomares Velasco**. El autor conserva la titularidad completa del lenguaje LÚMEN, incluyendo su diseño, especificación técnica, implementación, nombre comercial y todos los materiales asociados.

Ninguna institución educativa o tercero posee derechos de propiedad intelectual sobre este proyecto.

---

## Autor

**Omar Palomares Velasco**
Fundador y Presidente — **TriXxo Corp**
Estudiante de Ingeniería

**Contacto**: [Incluir método de contacto si deseas]
**Sitio web**: [Incluir URL si existe]

---

## Manifiesto LÚMEN

### La democratización del conocimiento

LÚMEN nace como respuesta directa a quienes alguna vez escucharon: **"eso no es para ti"**. No es una moda tecnológica ni un producto comercial más. Es una declaración de principios: **la programación puede y debe enseñarse con respeto, claridad y dignidad**.

### Homenaje a los maestros

Rendimos tributo a **Alejandro Taboada (Programación ATS)** y su lema fundacional:

> **"Si puedes imaginarlo, puedes programarlo."**

Su trabajo demostró que la programación es una puerta abierta para cualquier persona dispuesta a cruzarla. Su legado vive en LÚMEN: enseñar sin materias oscuras, sin jerga intimidante, con el convencimiento absoluto de que toda persona curiosa puede aprender.

### Principio inquebrantable

Compartimos el conocimiento como acto de generosidad. **El acceso al aprendizaje nunca se condiciona a capacidad económica**. Enseñamos porque creemos en el poder transformador del conocimiento.

### Nuestra promesa al estudiante

* **Claridad, no arrogancia**: explicamos conceptos con lenguaje accesible sin sacrificar rigor técnico
* **Razón, no memorización**: cada error explica el porqué, no solo señala el fallo
* **Guía, no abandono**: acompañamos paso a paso hasta lograr comprensión genuina
* **Dignidad, no condescendencia**: respetamos la inteligencia del estudiante

En LÚMEN, **el error es herramienta didáctica**; **la falla, una lección**. La única moneda válida es la **curiosidad**.

### Ingeniería con humanidad

LÚMEN conjuga **rigor técnico** y **empatía pedagógica**. Construimos un compilador profesional, una máquina virtual eficiente y herramientas robustas. Pero nuestro objetivo trasciende lo técnico: **hacer que aprender no intimide**, que la máquina deje de ser un misterio y que programar sea, fundamentalmente, **una experiencia creativa y liberadora**.

### Compromiso con el código abierto

Al liberar LÚMEN bajo licencia MIT, reafirmamos que **el conocimiento debe fluir libremente**. Invitamos a educadores, estudiantes y desarrolladores a usar, estudiar, modificar y mejorar LÚMEN.

---

## Estado del proyecto

**Fase actual**: Diseño e implementación inicial (Pre-alpha) — próximo hito: cierre de la **Fase 0** del roadmap (cimientos del proyecto).

Este repositorio documenta la evolución técnica y filosófica de LÚMEN. El lenguaje está en desarrollo activo, con especificación completa y primeras implementaciones del compilador en progreso.

### Contribuciones

LÚMEN es un proyecto de código abierto que acepta contribuciones. Si deseas participar:

1. Revisa la documentación técnica en `/docs`
2. Consulta los issues abiertos, organizados por fase del roadmap
3. Sigue las guías de contribución en `CONTRIBUTING.md`
4. Mantén el espíritu pedagógico en cada cambio

### Seguimiento del desarrollo

* **Repositorio**: [https://github.com/omarPVP123131/Lumen](https://github.com/omarPVP123131/Lumen)
* **Documentación**: [https://github.com/omarPVP123131/Lumen/tree/master/docs](https://github.com/omarPVP123131/Lumen/tree/master/docs)
* **Comunidad**: [PROXIMAMENTE]

---

## Filosofía final

LÚMEN no es solo un lenguaje de programación. Es una **declaración de que la tecnología debe ser accesible**, de que **aprender es un derecho**, y de que **la excelencia técnica y la pedagogía humanista pueden coexistir**.

Porque en LÚMEN creemos que **la luz del conocimiento debe brillar para todos**.

---

**TriXxo LÚMEN**
*Un lenguaje creado para quienes alguna vez escucharon que no podían.*

---

*Versión del documento: 2.0 — Roadmap técnico extendido*
*Última actualización: Julio 2026*