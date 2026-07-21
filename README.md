# LÚMEN — Lenguaje de Programación Nativo en Español

[![CI](https://github.com/omarPVP123131/Lumen/actions/workflows/ci.yml/badge.svg)](https://github.com/omarPVP123131/Lumen/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-1.2.0-orange)
![Tests](https://img.shields.io/badge/tests-294-brightgreen)

LÚMEN es un lenguaje de programación educativo de alto rendimiento con sintaxis nativa en español (y soporte completo para inglés). Cuenta con un pipeline de compilación completo escrito desde cero en Rust: Lexer → Parser → IR → Optimizador → Bytecode → VM.

## 🚀 Inicio Rápido

### Instalación (desde binario)
1. Ve a la sección de [Releases](https://github.com/omarPVP123131/Lumen/releases).
2. Descarga el ejecutable para tu plataforma (Windows, Linux o macOS).
3. Añade el ejecutable a tu `PATH` o ejecútalo directamente:
   ```bash
   ./lumen run mi_programa.nv
   ```

### Compilación desde fuente
Si tienes Rust instalado:
```bash
git clone https://github.com/omarPVP123131/Lumen.git
cd Lumen
cargo build --release
./target/release/lumen --help
```

## ✨ Características Principales

LÚMEN combina la legibilidad del español con la potencia de los lenguajes modernos:

- **🏠 Sintaxis Dual**: Programa en español (`si`, `mientras`, `funcion`) o inglés (`if`, `while`, `function`).
- **🛡️ Tipado Estático Moderno**: `entero`, `decimal`, `texto`, `booleano`, `lista<T>`, `opcion<T>`, `resultado<T, E>`.
- **🧩 Tipos Compuestos**:
  - **Estructuras**: Objetos con campos nombrados.
  - **Enums (Tipos Suma)**: Variantes con datos asociados (`enum Color { Rojo, Verde(entero) }`).
  - **Tuplas**: Agrupación heterogénea de valores `(1, "hola")`.
- **🔍 Pattern Matching**: Desempaquetado potente con `elegir` / `match`.
- **🧬 Genéricos**: Funciones y estructuras reutilizables con parámetros de tipo `<T>`.
- **📦 Módulos**: Sistema de importación robusto con `importar`.
- **⚡ Optimizado**: Bytecode compacto y VM de alta velocidad con cache de funciones y pooling de constantes.

## 💻 El Lenguaje en un Vistazo

### Funciones y Genéricos
```lumen
// Función genérica que funciona con cualquier tipo
funcion T identidad<T>(T valor) {
    retornar valor;
}

imprimir(identidad<entero>(42));
imprimir(identidad<texto>("Hola LÚMEN"));
```

### Enums y Pattern Matching
```lumen
enum Forma {
    Circulo(decimal),      // radio
    Rectangulo(decimal, decimal) // base, altura
}

funcion decimal area(Forma f) {
    elegir f {
        caso Forma::Circulo(r) { retornar 3.14159 * r * r; }
        caso Forma::Rectangulo(b, h) { retornar b * h; }
    }
}

Forma mi_forma = Forma::Circulo(5.0);
imprimir("Área: ", area(mi_forma));
```

### Tuplas y Destructuración
```lumen
// Declaración con destructuración
entero x, texto etiqueta = (100, "Coordenada X");

// Asignación múltiple
x, etiqueta = (200, "Nueva Coordenada");

imprimir(etiqueta, ": ", x);
```

## 🛠️ Herramientas de Línea de Comandos

| Comando | Acción |
|---------|--------|
| `lumen run <archivo>` | Ejecuta un archivo fuente `.nv` o bytecode `.nvc` |
| `lumen build <archivo>` | Compila el código fuente a bytecode optimizado `.nvc` |
| `lumen check <archivo>` | Realiza análisis léxico, sintáctico y semántico sin ejecutar |
| `lumen disasm <archivo>` | Muestra las instrucciones de bajo nivel (bytecode) del programa |

## 📚 Documentación

Para profundizar en LÚMEN, consulta nuestra documentación detallada:

- [📖 Guía del Lenguaje](docs/language.md) — Tutorial completo de sintaxis y tipos.
- [⚙️ Referencia de la CLI](docs/cli.md) — Todos los flags y comandos.
- [🏗️ Arquitectura Interna](docs/architecture.md) — Cómo funciona el compilador y la VM.
- [🗺️ Roadmap](docs/roadmap.md) — El camino hacia la versión 2.0.

## ❤️ Contribuir

LÚMEN es un proyecto abierto. Si quieres reportar un error o sugerir una mejora, por favor abre un *Issue* o envía un *Pull Request*. Consulta [CONTRIBUTING.md](docs/contributing.md) para más detalles.

---

MIT License — © 2026 Omar Palomares Velasco
