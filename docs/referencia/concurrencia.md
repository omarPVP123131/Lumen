# LÚMEN — Referencia de Concurrencia (`stdlib/concurrencia.nv`)

Módulo de concurrencia de LÚMEN: hilos, mutex, canales, tareas, streams,
actores, generadores y paralelismo. API dual español/inglés.

**Uso:** `importar "concurrencia.nv";`

> ⚠️ **Importante (v3.94.23, QA bug B)**: las APIs de hilos/mutex reciben el
> **nombre de la función** a ejecutar como `texto`, resuelto por reflexión en
> runtime. El compilador **verifica en `check`** que el nombre (cuando es un
> literal) corresponda a una función definida — un typo produce el error
> **E042** en tiempo de compilación. Si el nombre se pasa por una variable, la
> resolución fallida devuelve `error("Función '...' no definida")` en runtime
> (nunca `void` silencioso).

---

## Hilos

| Función | Descripción |
|---|---|
| `hilo_lanzar(fn, a1, a2, a3, a4, a5)` | Lanza un hilo ejecutando `fn` con hasta 5 argumentos |
| `hilo_lanzar1(fn, a1)` | Lanza un hilo con 1 argumento |
| `thread_spawn` / `thread_spawn1` | Alias en inglés |
| `hilo_esperar(handle)` | Espera la finalización y devuelve el resultado |
| `thread_join(handle)` | Alias en inglés |

```lumen
importar "concurrencia.nv";

funcion cualquiera tarea(entero n) { retornar n * 2; }

funcion entero principal() {
    cualquiera handle = hilo_lanzar1("tarea", 10);
    cualquiera salida = hilo_esperar(handle);
    imprimir(a_texto(salida));   // 20
    retornar 0;
}
```

> Nota (QA bug K): el paralelismo real de hardware depende de los núcleos
> disponibles del entorno. En entornos de 1 core la ejecución puede ser
> secuencial; mide en hardware multi-core para verificar el speedup.

---

## Mutex

| Función | Descripción |
|---|---|
| `mutex_nuevo()` / `mutex_new()` | Crea un mutex (devuelve handle) |
| `mutex_bloquear(m, fn, a1)` / `mutex_lock(m, fn, a1)` | Bloquea `m` y ejecuta `fn(a1)` dentro |

```lumen
cualquiera m = mutex_nuevo();
cualquiera r = mutex_bloquear(m, "tarea", 5);
```

---

## RWLock

| Función | Descripción |
|---|---|
| `rwlock_nuevo()` / `rwlock_new()` | Crea un RWLock |
| `rwlock_leer(rw, fn, a1)` / `rwlock_read(...)` | Lectura bajo lock |
| `rwlock_escribir(rw, fn, a1)` / `rwlock_write(...)` | Escritura bajo lock |

---

## Canales

| Función | Descripción |
|---|---|
| `canal_nuevo()` / `channel_new()` | Crea un canal |
| `canal_enviar(ch, val)` / `channel_send(...)` | Envía un valor |
| `canal_recibir(ch)` / `channel_recv(...)` | Recibe (bloqueante) |
| `canal_seleccionar(rx1, rx2)` / `canal_select(...)` | Selecciona entre dos canales |

---

## Tareas (async) y Streams

| Función | Descripción |
|---|---|
| `tarea_lanzar(fn, a1, a2)` / `task_spawn(...)` | Lanza tarea async |
| `tarea_lanzar1(fn, a1)` / `task_spawn1(...)` | Tarea async con 1 arg |
| `tarea_esperar(id)` / `task_await(id)` | Espera resultado |
| `stream_desde(fuente)` / `stream_from(...)` | Crea un stream |
| `stream_mapear(s, fn)` / `stream_map(...)` | Mapea con `fn` |
| `stream_filtrar(s, fn)` / `stream_filter(...)` | Filtra con `fn` |
| `stream_colectar(s)` / `stream_collect(s)` | Colecta a lista |

---

## Actores

| Función | Descripción |
|---|---|
| `actor_nuevo()` / `actor_new()` | Crea un actor (mailbox) |
| `actor_enviar(addr, msg)` / `actor_send(...)` | Envía mensaje |
| `actor_recibir(addr)` / `actor_recv(...)` | Recibe mensaje |

---

## Generadores

| Función | Descripción |
|---|---|
| `generador_nuevo(fn)` / `generator_new(fn)` | Crea un generador |
| `generador_siguiente(gen, val)` / `generator_next(...)` | Siguiente valor |

---

## Paralelismo

| Función | Descripción |
|---|---|
| `par_mapear(lst, fn)` / `par_map(...)` | Map en paralelo |
| `par_unir(fn1, a1, fn2, a2)` / `par_join(...)` | Ejecuta dos funciones en paralelo |

---

## Utilidades

| Función | Descripción |
|---|---|
| `dormir(ms)` / `sleep(ms)` | Duerme el hilo `ms` milisegundos |
| `scope_nuevo()` / `scope_new()` | Crea un scope de concurrencia estructurada |
| `scope_lanzar(s, fn, a1)` / `scope_spawn(...)` | Lanza tarea dentro del scope |
| `scope_cancelar(s)` / `scope_cancel(s)` | Cancela el scope |
| `arc_nuevo(val)` / `arc_new(val)` | Referencia atómica compartida |
| `arc_obtener(h)` / `arc_get(h)` | Lee el valor |
| `arc_asignar(h, val)` / `arc_set(h, val)` | Escribe el valor |
