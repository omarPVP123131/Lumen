# LÚMEN — El Lenguaje que Ilumina

**v1.6.0 — Documento de Posicionamiento y Visión**

> *"Programar no debería ser un lujo en inglés. Debería ser un derecho en tu idioma."*

---

## 🎯 Misión

Democratizar la educación en programación para los 500 millones de hispanohablantes del mundo, eliminando la barrera del inglés como requisito previo para aprender a programar.

LÚMEN existe porque creemos que **el idioma no debería determinar quién puede ser programador**. Mientras un adolescente en California escribe `if x > 0 { return true; }` a los 12 años, uno en México o Argentina debe primero aprender inglés. Eso es injusto. Eso lo cambiamos hoy.

---

## 💡 Lo Bueno

### 1. Español Nativo, Inglés Opcional

LÚMEN es el único lenguaje de programación moderno donde **el español es ciudadano de primera clase**, no un afterthought.

```nv
// Español — natural, sin fricción
si edad >= 18 {
    imprimir("Mayor de edad");
} sino {
    imprimir("Menor de edad");
}
```

Y si quieres inglés, solo agregas una línea:

```nv
importar ingles;   // Ahora puedes usar if/else/while/for también
```

**No existe otro lenguaje que haga esto.** Python, JavaScript, Rust — todos asumen inglés. LÚMEN te da la opción.

### 2. Compilador de Calidad Industrial

LÚMEN no es un intérprete de juguete. Tiene un pipeline de compilación completo:

```
Código Fuente (.nv)
  → Lexer (tokenización)
  → Parser (AST)
  → Análisis Semántico (verificación de tipos)
  → IR Intermedio (optimización)
  → Codegen (bytecode)
  → VM Stack-Based (ejecución)
```

- **353 tests** automatizados (0 fallos, 0 warnings)
- **Prop-testing** con miles de casos aleatorios
- **Constant folding**, **dead code elimination**, **shared pools**
- Build en **milisegundos** para archivos típicos

### 3. Mensajes de Error que Enseñan

Siguiendo la filosofía de Rust, cada error incluye:

```
  E031 No puedes asignar un valor de tipo 'Texto' a una variable de tipo 'Entero'
  --> programa.nv:3:12
   |
 2 | entero x = 0;
 3 | entero y = "hola";
   |            ^^^^^^
   |
   Ayuda: Usa un valor de tipo 'Entero' en lugar de 'Texto'
```

- **Código de error** único para buscar documentación
- **Preview de código** con línea anterior y siguiente
- **Subrayado** exacto del problema
- **Sugerencia** concreta de cómo arreglarlo
- **Colores ANSI** para legibilidad

### 4. Tipado Estático sin Dolor

Sistema de tipos robusto pero con inferencia:

```nv
x = 42;                    // El compilador deduce: entero
nombre = "Ana";            // El compilador deduce: texto
activo = verdadero;        // El compilador deduce: booleano
```

- **Type safety** en tiempo de compilación
- **Genéricos** con `funcion T id<T>(T v)`
- **Resultados** `Resultado<T, E>` para manejo de errores sin excepciones
- **Opciones** `Opcion<T>` para nulabilidad segura
- **Sum types** con `enum` (como Rust, Swift, Haskell)

### 5. Baterías Incluidas

No necesitas instalar 50 librerías para empezar:

| Incluido | Ejemplo |
|----------|---------|
| Formateador | `lumen fmt archivo.nv` |
| REPL | `lumen repl` |
| Test runner | `lumen test archivo.nv` |
| Módulos | `importar "libreria.nv"` |
| Scaffolding | `lumen new mi_proyecto` |
| Stdlib | `texto`, `matematicas`, `coleccion`, `fecha`, `archivos` |

### 6. Stack Traces en Runtime

Cuando algo falla, sabes exactamente dónde:

```
Error: Índice 5 fuera de rango (largo: 3)
Pila de llamadas:
  · procesar_lista
  · main
```

### 7. Conversiones Seguras

```nv
Resultado<entero, texto> r = a_entero("42");    // ✅ Éxito: 42
Resultado<entero, texto> r2 = a_entero("hola");  // ❌ Error: no es número
```

No más `parseInt()` que explota. Aquí siempre sabes si funcionó.

### 8. Interpolación de Strings

```nv
imprimir("Hola {nombre}, tienes {edad} años y mides {altura}m");
```

Adiós a `"Hola " + nombre + ", tienes " + str(edad) + " años"`.

### 9. 200 Ejemplos de Código Real

50 ejemplos junior (principiantes), 80 senior (avanzados), 70 de software real (apps, algoritmos, sistemas). Todos ejecutables, todos probados.

### 10. CI/CD Robusto

Pre-commit y pre-push hooks automáticos con PowerShell. Builds reproducibles. Tests en cada push. Cobertura con `llvm-cov`.

---

## ⚠️ Lo Malo (Honestidad Total)

### 1. Es Joven (v1.4.0)

LÚMEN tiene menos de 2 años. No está en producción en ningún lado. Úsalo para aprender, enseñar, prototipar — no para tu sistema bancario. Todavía.

### 2. Ecosistema Inexistente

No hay:
- Package registry (npm/pypi/crates.io equivalente)
- Librerías de terceros
- Framework web
- ORM / driver de base de datos
- SDK de cloud

**Por ahora, todo lo que necesitas lo construyes tú o está en la stdlib.** El package manager (`lumen install`) está en el roadmap.

### 3. Comunidad Pequeña

Somos un equipo chico. Si encuentras un bug, probablemente eres la primera persona en verlo. La buena noticia: lo arreglamos rápido.

### 4. Sin Compilación Nativa (AOT)

LÚMEN compila a bytecode que corre en su VM. No genera binarios nativos (todavía). WASM y LLVM están en el roadmap para v2.0.

### 5. Sin IDE/LSP Completo

Hay un LSP básico pero no tiene:
- Autocompletado avanzado
- Refactoring automático
- Debugging integrado
- Syntax highlighting en todos los editores

VS Code extension recomendada: usar resaltado genérico por ahora.

### 6. Features Avanzadas Pendientes

Lo que **no** tiene LÚMEN hoy:
- Async/await
- Pattern matching exhaustivo con guardas
- Traits / interfaces
- Closures con captura por referencia (solo por valor)
- Macros
- Operator overloading
- Unsafe / FFI a C

### 7. Solo ASCII en Strings

Sin soporte para Unicode avanzado en indexación (pero los strings se almacenan como UTF-8, y `s[i]` itera por code points).

### 8. Documentación en Crecimiento

LENGUAJE.md y HERRAMIENTAS.md son completos, pero no hay:
- Tutorial interactivo
- Libro digital
- Videos oficiales
- Cursos estructurados

---

## 🔭 Visión a Largo Plazo

### v1.5 — Estabilidad y Robustez (2026 Q3-Q4)
- Pattern matching exhaustivo
- Traits/Interfaces
- Garbage Collector
- `lumen doc` (generación de docs)

### v2.0 — Madurez y Producción (2027)
- Compilación AOT vía LLVM
- Target WASM (ejecución en navegador)
- Package manager (`lumen install`)
- LSP completo (autocompletado, go-to-def, refactor)
- Playground web interactivo

### v3.0 — Ecosistema (2028+)
- Framework web full-stack
- ORM / drivers de base de datos
- SDK de cloud (AWS, GCP, Azure)
- Librerías de machine learning
- Librerías de data science
- Comunidad autosustentable

---

## 🎓 Posicionamiento

### ¿Para quién es LÚMEN?

| Perfil | ¿LÚMEN es para ti? |
|--------|---------------------|
| **Estudiante hispanohablante** aprendiendo a programar | ✅ **Perfecto.** Sin barrera de idioma. |
| **Profesor de programación** en secundaria/universidad | ✅ **Ideal.** Diseñado para enseñar conceptos, no sintaxis inglesa. |
| **Autodidacta** que quiere crear scripts y herramientas | ✅ **Bueno.** Sintaxis clara, errores útiles, REPL. |
| **Desarrollador profesional** buscando producción | ⚠️ **Espera.** El ecosistema está en construcción. |
| **Startup** que necesita backend en producción | ❌ **No todavía.** Usa Rust, Go o Python por ahora. |
| **Investigador/científico** de datos | ❌ **No.** Python + NumPy/Pandas es mejor hoy. |

### ¿Contra quién compite LÚMEN?

| Lenguaje | Fortaleza | Debilidad vs LÚMEN |
|----------|-----------|---------------------|
| **Python** | Ecosistema masivo, librerías | Sintaxis inglesa obligatoria, sin tipos estáticos |
| **JavaScript** | Web nativo, ubicuo | Sintaxis inglesa, comportamiento impredecible |
| **Rust** | Performance, seguridad | Curva de aprendizaje brutal, sintaxis inglesa |
| **Go** | Simplicidad, concurrencia | Sintaxis inglesa, sin genéricos expresivos |
| **Swift** | Apple ecosystem, seguridad | Solo Apple, sintaxis inglesa |

**LÚMEN ocupa un nicho único**: primer lenguaje de programación para hispanohablantes con tipos estáticos, seguridad de memoria, y mensajes de error pedagógicos.

---

## 📣 Pitch de 30 Segundos

> "LÚMEN es un lenguaje de programación diseñado para que cualquier persona de habla hispana pueda aprender a programar sin saber inglés. Escribes `si`, `mientras`, `funcion` — como piensas. Tiene tipos estáticos como Rust, inferencia como TypeScript, mensajes de error que te enseñan, y todo viene incluido: formateador, REPL, tests, y 200 ejemplos de código real."

---

## 🏷️ Taglines

- *"Programar en tu idioma, compilar con esteroides."*
- *"El lenguaje que no te obliga a aprender inglés."*
- *"Rust para principiantes, en español."*
- *"Porque programar ya es difícil. El idioma no debería serlo."*
- *"LÚMEN: donde el código se lee como se piensa."*
- *"Typescript safety, Python simplicity, Spanish first."*

---

## 🔗 Links

- **Repositorio**: [github.com/anomalyco/LumenRust](https://github.com/anomalyco/LumenRust)
- **Manual**: `LENGUAJE.md` (en este repositorio)
- **Herramientas**: `HERRAMIENTAS.md` (en este repositorio)
- **Roadmap**: `docs/roadmap.md` (en este repositorio)
- **Reportar bugs**: GitHub Issues
- **Contribuir**: `docs/contributing.md`

---

## 🧠 Filosofía de Diseño

1. **El idioma no es la barrera.** Si un concepto se puede explicar en español, el código también.
2. **Los errores enseñan.** Cada mensaje de error debe ser una mini-lección.
3. **Baterías incluidas.** fmt, test, repl, docs — todo en la caja.
4. **Tipado sin ceremonia.** Seguridad de tipos sin 50 anotaciones.
5. **Transparencia.** El compilador explica qué hace y por qué.
6. **Honestidad.** Si algo no funciona, lo decimos. Sin marketing falso.

---

*LÚMEN v1.4.0 — Julio 2026 · Hecho con convicción, no con hype.*
