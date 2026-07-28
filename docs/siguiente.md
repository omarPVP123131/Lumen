# Propuesta de Siguientes Bloques — Roadmap LÚMEN

Clasificados por **complejidad** (implementación) e **impacto** (utilidad para el usuario).

---

## 🟢 Prioridad Alta — Fácil + Alto Impacto

| Bloque | Complejidad | Impacto | Razón |
|--------|-------------|---------|-------|
| **261-270 DevOps** | 🟢 Baja | 🔥 Muy Alto | CI/CD, releases automáticos, tests en GitHub Actions. Ya hay CI básico, solo falta pulirlo. |
| **371-400 Comunidad & Docs** | 🟢 Baja | 🔥 Muy Alto | Documentación, tutoriales, web. Lo que más necesita un lenguaje nuevo. |
| **231-250 Portabilidad** | 🟡 Media | 🔥 Alto | Linux/macOS: `tui_core.nv` ya tiene soporte parcial. Hace falta probar y corregir. |

## 🟡 Prioridad Media — Moderado + Alto Impacto

| Bloque | Complejidad | Impacto | Razón |
|--------|-------------|---------|-------|
| **171-190 GUI Nativa** | 🔴 Muy Alta | 🔥 Muy Alto | Ventanas HWND, botones, inputs. Win32 API FFI (~20 APIs diferentes). Mucho código pero killer feature. |
| **191-210 Gráficos & Juegos** | 🔴 Muy Alta | 🔥 Alto | SDL2 FFI: sprites, input, sonido. Similar complejidad a GUI. |
| **251-260 Native Embed** | 🔴 Muy Alta | 🔥 Muy Alto | `extern C`, `unsafe`, inline ASM. Cambios profundos en el compilador (lexer→parser→sema→codegen). Feature revolucionario. |

## 🔵 Prioridad Baja — Muy Complejo + Nicho

| Bloque | Complejidad | Impacto | Razón |
|--------|-------------|---------|-------|
| **211-230 WebAssembly** | 🔴 Muy Alta | 🟡 Medio | Compilar LÚMEN a WASM requiere backend nuevo. Útil pero nicho. |
| **291-320 AI/ML** | 🔴 Muy Alta | 🟡 Medio | Depende de bindings a librerías C (llama.cpp, tensorflow). Nicho. |
| **321-340 Data Science** | 🔴 Muy Alta | 🟢 Bajo | Similar a AI/ML. Más útil cuando el lenguaje tenga usuarios. |
| **341-370 Cloud** | 🔴 Muy Alta | 🟢 Bajo | HTTP server, despliegue. Depende de tener ecosistema maduro. |
| **271-290 Seguridad** | 🔴 Muy Alta | 🟡 Medio | Criptografía, sandboxing, auditoría. Ya tenemos crypto vía BCrypt. |

---

## 🎯 Recomendación

### Sprint actual: **GUI Nativa (171-190)**
Por qué:
- Mayor impacto visible: ventanas nativas es un hito
- Ya tenemos toda la infraestructura FFI lista
- Usa las mismas técnicas que `tui_core.nv` pero con `user32.dll` en vez de `kernel32.dll`
- Demo impresionante para mostrar el lenguaje

### Siguiente: **Native Embed (251-260)**
Por qué:
- Feature diferenciadora: ningún otro lenguaje ofrece `extern C` + `unsafe` + inline ASM
- Ya sentamos las bases con bitwise ops y `sino si`
- Permite escribir código de bajo nivel SIN salir de LÚMEN

### Luego: **DevOps (261-270) + Docs (371-400)**
Por qué:
- Mantenimiento: CI/CD robusto + documentación atraen contribuidores
- Bajo esfuerzo, alto retorno

---

¿Quieres que arranque con **GUI Nativa (171-190)** o prefieres otro bloque?
