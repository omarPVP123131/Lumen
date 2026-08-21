# Fuzzers y comprobaciones de LÚMEN

Los tres fuzzers usan un **oráculo independiente en Python**, no comparan LÚMEN
consigo mismo: los bugs de estas zonas (BUG-147..151) no eran divergencias
entre backends, sino que ambos backends coincidían en equivocarse.

| Script | Genera con | Qué comprueba |
|---|---|---|
| `fz10.sh` | `fuzz6.py` | structs, listas y `prestado mut` — VM y nativo contra el valor calculado en Python |
| `fz11.sh` | `fuzz7.py` | closures y captura de variables (BUG-148/149/150) |
| `fz12.sh` | `fuzz8.py` | **rechazo**: el código inválido debe rechazarse, no ejecutarse (BUG-151) |
| `fz13.py` | (autónomo) | **regex**: `__regex_coincide` y `__regex_reemplazar` deben dar lo mismo en la VM y en el binario nativo (BUG-166/167) |
| `allex.sh` | — | todos los ejemplos de `examples/` se ejecutan sin fallar |

## Uso

```bash
python3 gen/fuzz7.py 240   # genera el corpus
bash gen/fz11.sh           # lo ejecuta
```

Las rutas son relocalizables: funcionan desde cualquier directorio. Se pueden
fijar `LUMEN_RAIZ` (raíz del repo) y `LUMEN_BIN` (binario a probar) si hace
falta apuntar a otro sitio.

## Cómo leer la salida

`fz13.py` es autónomo (genera y ejecuta en una sola pasada) y admite número de
casos y semilla: `python3 gen/fz13.py 250 77`. Termina en `diffs=N`, que debe
ser 0; si el binario nativo muere, informa de su código de salida.

`fz10`/`fz11` terminan en `oraculo=N diffs=N crashes=N`. Los tres deben ser 0:
`oraculo` es el número de casos en que LÚMEN discrepa del resultado esperado,
`diffs` los que discrepan entre la VM y el binario nativo.

`fz12` termina en `acepta_run=N acepta_check=N ejecuta_cuerpo=N`, también todos
0. El tercero es el importante: un programa inválido que informa del error pero
ya ha ejecutado medio cuerpo es tan peligroso como el que no informa.

Los corpus (`gen/fz*/`) no se versionan: se regeneran con el `.py`
correspondiente, que lleva semilla fija para que sean reproducibles.
