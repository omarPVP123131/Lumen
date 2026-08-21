# Distribución de LÚMEN v3.0

Cómo se construye y se publica un release, y por qué está montado así.

---

## 1. Qué produce cada workflow

Hay **dos**, deliberadamente separados:

| Workflow | Cuándo | Qué hace |
|---|---|---|
| `ci.yml` | cada push y PR | verifica: formato, clippy, 704 pruebas en 3 SO, fuzzers, ejemplos, stdlib, instalación limpia |
| `release.yml` | al empujar un tag `vX.Y.Z` | compila los 4 artefactos, los verifica y publica el release |

**Por qué separados.** Si un test rojo bloquea la publicación desde el mismo
job, cuesta distinguir «el código está mal» de «el empaquetado está mal». Con
dos workflows, un fallo dice de inmediato en cuál de las dos cosas estás.

El release **no se fía** de que CI haya pasado: vuelve a correr la suite en
cada runner antes de subir nada. Publicar un binario que no pasa sus propios
tests no debería ser posible por descuido.

---

## 2. Artefactos y por qué esos

| Artefacto | Target | Nota |
|---|---|---|
| `lumen-x86_64-unknown-linux-musl.tar.gz` | musl estático | corre en **cualquier** distro |
| `lumen-aarch64-unknown-linux-musl.tar.gz` | musl estático ARM64 | servidores ARM, Raspberry Pi |
| `lumen-universal-apple-darwin.tar.gz` | universal | Apple Silicon **e** Intel |
| `lumen-x86_64-pc-windows-msvc.zip` | MSVC | |

### Linux: musl, no glibc

Un binario compilado en `ubuntu-latest` con glibc enlaza contra símbolos como
`GLIBC_2.34` y **no arranca** en Ubuntu 20.04, Debian 11, RHEL 8 ni Amazon
Linux 2. El usuario ve `version 'GLIBC_2.34' not found` y se va.

musl produce un binario estático sin esa dependencia. El workflow lo verifica
explícitamente en cada release:

```bash
file target/.../lumen | grep -q "static"
test "$(strings target/.../lumen | grep -c GLIBC_)" -eq 0
```

Si alguien cambia el target a glibc por comodidad, el release falla ahí mismo
en vez de publicar algo que se rompe en máquinas ajenas.

### macOS: universal, no sólo ARM

`macos-latest` es ARM64. Compilar sólo ahí deja fuera a todos los Mac Intel. Se
compilan las dos arquitecturas y se unen con `lipo`.

---

## 3. La stdlib viaja dentro del binario

Este era el bloqueante real, y no lo arreglaba ningún workflow (BUG-152).

El CLI resolvía los `importar` **buscando ficheros en disco**. Un binario
instalado fuera del repo no encontraba **ninguno** de los 69 módulos: el
programa más simple con `importar "texto";` fallaba nada más instalar.

Ahora `lumen-sema` —el crate que resuelve los imports— embebe la stdlib en
tiempo de compilación mediante su `build.rs`. Consecuencias:

- **Todo consumidor lo hereda**: CLI, WASM y la API embebible. No hubo que
  parchear cada uno por separado.
- **El disco sigue teniendo prioridad**: `-L mi_stdlib/` y una `stdlib/` local
  siguen ganando. Lo embebido es el último recurso, así que quien desarrolle
  sobre el repo ve sus cambios al instante.
- **Recompilar actualiza la stdlib embebida** automáticamente; no hay una copia
  que se quede vieja.

Coste: el binario pasa de ~18,2 MB a ~18,7 MB. Barato por no tener que acertar
con una ruta de instalación en cuatro sistemas operativos.

El tarball incluye además `stdlib/` como fuente: sirve de referencia legible y
permite sobrescribir módulos con `-L`.

**Regresión imposible en silencio**: el job `instalacion-limpia` copia el
binario a un directorio vacío, sin repo alrededor, y comprueba que los 69
módulos importan. Si alguien deshace el embebido, el CI se pone rojo.

---

## 4. `build --native` necesita un compilador de C

`lumen build --native` genera C y llama a `gcc`/`clang`. En Linux y macOS suele
haber uno; en Windows, la máquina del usuario final probablemente no.

No es un bug —es la naturaleza de la compilación AOT— pero **el mensaje sí lo
era**: decía «Instala GCC», que en macOS y Windows ni siquiera es el consejo
correcto. Ahora detecta el sistema y dice qué instalar en él, y recuerda que
`lumen run` no necesita nada (BUG-153).

Comportamiento garantizado por CI: código de salida ≠ 0, **ningún binario a
medias en disco**, y un mensaje que menciona el compilador de C.

---

## 5. Publicar sin ser el dueño del repo

El repositorio original es de **`omarPVP123131`**. Un workflow que cree
releases necesita permiso de escritura ahí (`permissions: contents: write`),
así que hay dos caminos:

### Opción A — Pull request al upstream (recomendada)

Se abre un PR con los parches, los workflows y la documentación. Al fusionarse,
`ci.yml` empieza a correr en cada PR del proyecto y el mantenedor publica
cuando quiera empujando un tag:

```bash
git tag v3.0.0
git push origin v3.0.0
```

Ventajas: los usuarios lo reciben desde el sitio oficial, y el proyecto se
queda con la red de seguridad (fuzzers, prueba de instalación limpia).

### Opción B — Fork propio

Si el upstream no responde o prefiere otro rumbo, un fork publica sus propios
releases sin tocar nada del original. Requiere dejar **muy claro en el README**
que es un fork no oficial y de qué commit parte, para no confundir a nadie
sobre cuál es el proyecto de referencia.

En ambos casos: `GITHUB_TOKEN` es suficiente, **no hacen falta secretos
adicionales**. Nada del workflow requiere credenciales de terceros.

---

## 6. Publicar un release, paso a paso

```bash
# 1. La versión del workspace y el tag deben coincidir.
grep '^version' Cargo.toml

# 2. Comprobar en local lo mismo que comprobará el CI.
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --release --workspace
bash gen/allex.sh
python3 gen/fuzz7.py 240 && bash gen/fz11.sh
python3 gen/fuzz8.py 120 && bash gen/fz12.sh

# 3. Tag y push: el workflow hace el resto.
git tag v3.0.0 && git push origin v3.0.0
```

El release sale con notas generadas, los cuatro artefactos y un único
`SHA256SUMS.txt`.

### Verificación por el usuario

```bash
sha256sum -c SHA256SUMS.txt --ignore-missing
```

---

## 7. Lo que sigue sin cubrirse

Honestidad sobre los límites de este montaje:

- **Sin firma ni notarización.** macOS mostrará el aviso de Gatekeeper y
  Windows el de SmartScreen. Resolverlo exige certificados de pago y una cuenta
  de desarrollador de Apple; es una decisión del proyecto, no algo que un
  workflow arregle.
- **Sin gestores de paquetes** (Homebrew, winget, AUR, `cargo install`). Son el
  siguiente paso natural una vez haya releases estables con URLs fijas.
- **ARM64 de Linux se compila cruzado y no se testea**: los runners no emulan
  ARM. La suite lo cubre en x86_64 y macOS ARM, que juntos ejercitan tanto la
  arquitectura como el sistema, pero la combinación exacta no.
