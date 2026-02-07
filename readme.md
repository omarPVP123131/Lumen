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

### Explícitamente excluido (decisión pedagógica)

* **POO**: se introduce complejidad conceptual prematura
* **Concurrencia**: requiere modelo mental avanzado
* **Garbage Collection**: se prefiere gestión manual educativa
* **Sistema de módulos**: fase posterior
* **Librerías externas**: evita dependencias y complejidad

Estas exclusiones son **intencionales** y responden a la filosofía pedagógica de LÚMEN: dominar lo fundamental antes de lo avanzado.

---

## Hoja de ruta

### Versión 1.0 — Fundamentos (En desarrollo)
- [x] Especificación del lenguaje
- [ ] Implementación del lexer
- [ ] Implementación del parser y AST
- [ ] Análisis semántico
- [ ] Generación de bytecode
- [ ] Máquina virtual funcional
- [ ] CLI básica
- [ ] Suite de pruebas
- [ ] Documentación del lenguaje

### Versión 1.5 — Herramientas de desarrollo
- [ ] Modo verbose con información de compilación
- [ ] Desensamblador de bytecode
- [ ] Optimizaciones básicas del compilador
- [ ] Manejo de errores mejorado

### Versión 2.0 — Herramientas educativas
- [ ] Modo de ejecución paso a paso
- [ ] Visualización gráfica del stack y memoria
- [ ] Debugger educativo
- [ ] Editor con sintaxis resaltada
- [ ] Playground web interactivo
- [ ] Materiales didácticos (ejercicios, tutoriales)

### Versión 3.0 — Expansión controlada
- [ ] Sistema básico de módulos
- [ ] Tipos de datos compuestos (arrays, registros)
- [ ] Gestión de memoria avanzada
- [ ] Optimizaciones de rendimiento

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

**Sitio web**: [https://portfolio-react-nine-xi.vercel.app/]

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

**Fase actual**: Diseño e implementación inicial (Pre-alpha)

Este repositorio documenta la evolución técnica y filosófica de LÚMEN. El lenguaje está en desarrollo activo, con especificación completa y primeras implementaciones del compilador en progreso.

### Contribuciones

LÚMEN es un proyecto de código abierto que acepta contribuciones. Si deseas participar:

1. Revisa la documentación técnica en `/docs`
2. Consulta los issues abiertos
3. Sigue las guías de contribución en `CONTRIBUTING.md`
4. Mantén el espíritu pedagógico en cada cambio

### Seguimiento del desarrollo

* **Repositorio**: [[URL del repositorio]](https://github.com/omarPVP123131/Lumen)
* **Documentación**: [[URL de la documentación](https://github.com/omarPVP123131/Lumen/tree/master/docs)]
* **Comunidad**: [PROXIMAMENTE]

---

## Filosofía final

LÚMEN no es solo un lenguaje de programación. Es una **declaración de que la tecnología debe ser accesible**, de que **aprender es un derecho**, y de que **la excelencia técnica y la pedagogía humanista pueden coexistir**.

Porque en LÚMEN creemos que **la luz del conocimiento debe brillar para todos**.

---

**TriXxo LÚMEN**  
*Un lenguaje creado para quienes alguna vez escucharon que no podían.*

---

*Versión del documento: 1.0*  
*Última actualización: Febrero 2026*