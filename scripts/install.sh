#!/bin/sh
# install.sh — Instalador de LÚMEN para Linux/macOS
# Uso: curl -fsSL https://raw.githubusercontent.com/omarPVP123131/Lumen/main/scripts/install.sh | sh

set -e
# La versión se puede fijar con LUMEN_VERSION=v3.0.0 sh install.sh
VERSION="${LUMEN_VERSION:-3.0.0}"
REPO="omarPVP123131/Lumen"

# Detectar OS y arquitectura
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)  TARGET="x86_64-unknown-linux-gnu" ;;
    darwin) TARGET="x86_64-apple-darwin" ;;
    *)      echo "OS no soportado: $OS"; exit 1 ;;
esac

case "$ARCH" in
    aarch64|arm64)
        case "$OS" in
            linux)  TARGET="aarch64-unknown-linux-gnu" ;;
            darwin) TARGET="aarch64-apple-darwin" ;;
        esac ;;
esac

BIN_DIR="${HOME}/.lumen/bin"
mkdir -p "$BIN_DIR"

echo "LÚMEN v${VERSION} Installer (${TARGET})"
echo "======================================"

# Descargar release
URL="https://github.com/${REPO}/releases/download/v${VERSION}/lumen-${TARGET}.tar.gz"
TMP_FILE="/tmp/lumen-${VERSION}.tar.gz"

echo "  Descargando ${URL} ..."
if curl -fsSL "$URL" -o "$TMP_FILE" 2>/dev/null; then
    echo "  ✓ Descargado"

    # Verificar integridad si el release publica SHA256SUMS.txt. Un binario
    # corrupto o manipulado no debe instalarse en silencio.
    SUMS_URL="https://github.com/${REPO}/releases/download/v${VERSION}/SHA256SUMS.txt"
    if curl -fsSL "$SUMS_URL" -o /tmp/lumen-sums.txt 2>/dev/null; then
        if command -v sha256sum >/dev/null 2>&1; then
            REAL=$(sha256sum "$TMP_FILE" | cut -d" " -f1)
        elif command -v shasum >/dev/null 2>&1; then
            REAL=$(shasum -a 256 "$TMP_FILE" | cut -d" " -f1)
        else
            REAL=""
        fi
        ESPERADO=$(grep "lumen-${TARGET}.tar.gz" /tmp/lumen-sums.txt 2>/dev/null | cut -d" " -f1)
        if [ -n "$REAL" ] && [ -n "$ESPERADO" ] && [ "$REAL" != "$ESPERADO" ]; then
            echo "  ✗ SHA256 no coincide — descarga corrupta o manipulada."
            echo "    esperado: $ESPERADO"
            echo "    obtenido: $REAL"
            rm -f "$TMP_FILE"
            exit 1
        fi
        [ -n "$REAL" ] && [ -n "$ESPERADO" ] && echo "  ✓ SHA256 verificado"
        rm -f /tmp/lumen-sums.txt
    fi

    tar -xzf "$TMP_FILE" -C "$BIN_DIR" 2>/dev/null || cp "$TMP_FILE" "$BIN_DIR/lumen"
    chmod +x "$BIN_DIR/lumen"
else
    echo "  ! No se encontró release para v${VERSION}"
    echo "  Construyendo desde fuente..."
    
    if ! command -v cargo >/dev/null 2>&1; then
        echo "  ✗ Necesitas Rust: https://rustup.rs"
        exit 1
    fi
    
    SRC_DIR="/tmp/lumen-build"
    rm -rf "$SRC_DIR"
    git clone "https://github.com/${REPO}.git" --branch "v${VERSION}" "$SRC_DIR" 2>/dev/null || \
    git clone "https://github.com/${REPO}.git" "$SRC_DIR"
    
    cd "$SRC_DIR"
    cargo build --release
    cp "target/release/lumen" "$BIN_DIR/lumen"
    cd /tmp
    rm -rf "$SRC_DIR"
    echo "  ✓ Compilado desde fuente"
fi

# Agregar al PATH
case "$SHELL" in
    */bash)
        if ! grep -q ".lumen/bin" "${HOME}/.bashrc" 2>/dev/null; then
            echo "export PATH=\"\$PATH:${BIN_DIR}\"" >> "${HOME}/.bashrc"
            echo "  ✓ Agregado a ~/.bashrc"
        fi ;;
    */zsh)
        if ! grep -q ".lumen/bin" "${HOME}/.zshrc" 2>/dev/null; then
            echo "export PATH=\"\$PATH:${BIN_DIR}\"" >> "${HOME}/.zshrc"
            echo "  ✓ Agregado a ~/.zshrc"
        fi ;;
esac

export PATH="$PATH:${BIN_DIR}"
echo ""
echo "  LÚMEN instalado en: ${BIN_DIR}/lumen" 
echo "  Prueba: lumen run examples/hello.nv"
