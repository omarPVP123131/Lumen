# Migrar de la v2.4.x a la v3.0

La v3.0 corrige 153 bugs. La mayoría son transparentes: código que antes fallaba
ahora funciona. Pero **unos pocos arreglos rompen programas que antes compilaban
o se ejecutaban**, y esta página los recoge todos.

Es intencionado y es lo que justifica el cambio de versión mayor. En todos los
casos lo que se rompe es código que **ya estaba mal**: la v2.4.x lo aceptaba en
silencio y hacía algo distinto de lo que aparentaba.

---

## 1. Un bloque sin `{` ya es un error (BUG-151)

**Lo más probable que te afecte.**

Antes esto se ejecutaba, imprimía `hola` y `lumen check` lo daba por válido:

```lumen
si (1 == 2) cualquier_cosa { imprimir("hola"); }
```

La condición se descartaba entera y el bloque corría **igual, sin condición**.
Ahora es un error `E017`.

**Qué hacer.** Pasa `lumen check` a tus fuentes. Si aparece un E017, mira la
línea: casi siempre sobra un token entre la condición y el `{`, o falta el
propio `{`. Si el programa dependía de que ese bloque se ejecutase, **estaba
dependiendo de un bug**: decide si el bloque debe correr siempre (quítale el
`si`) o sólo bajo la condición (arregla la condición).

Afecta a `si`, `mientras`, `sino`, `para` y los cuerpos de función.

---

## 2. Las closures ahora capturan de verdad por referencia (BUG-148/149/150)

La semántica documentada siempre fue **captura por referencia**, pero la
implementación la perdía en varios casos. Lo que cambia:

```lumen
funcion vacio p() {
    entero x = 0;
    sea inc = funcion(entero n) { x = x + n; retornar x; };
    imprimir(inc(5));   // 5   — igual que antes
    imprimir(x);        // v2.4.x: 0     ← la mutación se perdía
                        // v3.0:   5     ← correcto
}
```

Además, devolver una closure desde una función ya no falla con
`Variable '__cap_1_n' no definida`, y el binario nativo se comporta igual que
la máquina virtual (antes divergían en silencio).

**Qué hacer.** Si tu código compensaba el bug —por ejemplo, devolviendo el
valor y reasignándolo a mano porque «las closures no mutaban»— esa compensación
ahora **duplica el efecto**. Busca closures que muten variables capturadas y
revisa si el llamador vuelve a aplicar el cambio.

**Limitación conocida.** Dos closures sobre una variable declarada en el nivel
superior del programa (fuera de toda función) no se propagan entre sí la última
mutación. Dentro de funciones —el patrón habitual de factoría y contador— el
comportamiento es correcto. Si te afecta, envuelve el código en una función.

---

## 3. `resultado` y `leer` son palabras reservadas (BUG-006)

```lumen
entero resultado = 5;   // v2.4.x: a veces colaba
                        // v3.0:   E011
```

**Qué hacer.** Renómbralas. `resultado` → `res`, `total`, `salida`;
`leer` → `obtener`, `cargar`, `ver`.

---

## 4. `prestado mut` ahora propaga las mutaciones (BUG-147)

Pasar `s.campo` o `l[i]` a un parámetro `prestado mut` **descartaba la
mutación en silencio**. Ahora funciona como siempre debió.

**Qué hacer.** Nada, salvo que tu código dependiera de que no se propagara. Si
llevabas una copia manual «porque no funcionaba», sobra.

---

## 5. La compilación AOT rechaza lo que no soporta (BUG-084/086)

El backend Cranelift antes aceptaba programas que no podía compilar y producía
binarios incorrectos. Ahora **rechaza la compilación** enumerando lo que le
falta.

**Qué hacer.** Si `lumen build --aot cranelift` empieza a fallar, el binario
que producía antes era incorrecto. Usa el backend de C (`--aot c`, el de
`--native`), que es el completo, o ejecuta con `lumen run`.

---

## Lista de verificación

```bash
# 1. ¿Compila todo?
for f in $(find . -name '*.nv'); do lumen check "$f" || echo "REVISAR: $f"; done

# 2. ¿Se comporta igual? Compara con tu versión anterior.
lumen run mi_programa.nv > nuevo.txt
diff viejo.txt nuevo.txt

# 3. Si usas AOT, verifica que el binario coincide con la VM.
lumen run p.nv > vm.txt
lumen build p.nv --native -o p_bin && ./p_bin > nat.txt
diff vm.txt nat.txt   # en v3.0 deben coincidir siempre
```

Si algo de la v2.4.x deja de funcionar y **no** aparece en esta página, es un
fallo de la migración y no una decisión de diseño: merece un issue.
