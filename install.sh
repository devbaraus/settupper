#!/usr/bin/env sh
set -eu

REPO="${SETTUPPER_REPO:-devbaraus/settupper}"
VERSION="${SETTUPPER_VERSION:-}"
INSTALL_ROOT="${SETTUPPER_INSTALL_ROOT:-$HOME/.local}"
BIN_DIR="$INSTALL_ROOT/bin"
BINARY_NAME="settupper"

err() {
    echo "erro: $*" >&2
    exit 1
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "$1 nao encontrado."
}

tmpdir=""
cleanup() {
    if [ -n "$tmpdir" ] && [ -d "$tmpdir" ]; then
        rm -rf "$tmpdir"
    fi
}
trap cleanup EXIT INT TERM

detect_platform() {
    os="$(uname -s 2>/dev/null || true)"
    arch="$(uname -m 2>/dev/null || true)"

    case "$os" in
        Linux) os_name="linux" ;;
        Darwin) os_name="macos" ;;
        *) err "sistema operacional nao suportado: ${os:-desconhecido}" ;;
    esac

    case "$arch" in
        x86_64|amd64) arch_name="x86_64" ;;
        arm64|aarch64) arch_name="aarch64" ;;
        *) err "arquitetura nao suportada: ${arch:-desconhecida}" ;;
    esac

    ARCHIVE="settupper-${os_name}-${arch_name}.tar.gz"
}

download() {
    url="$1"
    output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fL --retry 3 --proto '=https' --tlsv1.2 -o "$output" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$output" "$url"
    else
        err "curl ou wget nao encontrado."
    fi
}

latest_tag() {
    if command -v curl >/dev/null 2>&1; then
        latest_url="$(curl -fsIL -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest")"
    elif command -v wget >/dev/null 2>&1; then
        latest_url="$(wget --server-response --max-redirect=10 --spider "https://github.com/$REPO/releases/latest" 2>&1 | awk '/^  Location: / { url=$2 } END { print url }' | tr -d '\r')"
    else
        err "curl ou wget nao encontrado."
    fi

    tag="${latest_url##*/}"
    [ -n "$tag" ] && [ "$tag" != "latest" ] || err "nao foi possivel descobrir a ultima release em https://github.com/$REPO"
    printf '%s\n' "$tag"
}

detect_platform
need_cmd tar

if [ -z "$VERSION" ]; then
    VERSION="$(latest_tag)"
fi

tmpdir="$(mktemp -d)"
archive_path="$tmpdir/$ARCHIVE"
download_url="https://github.com/$REPO/releases/download/$VERSION/$ARCHIVE"

echo "Baixando settupper $VERSION ($ARCHIVE)..."
download "$download_url" "$archive_path"

tar -xzf "$archive_path" -C "$tmpdir"
[ -f "$tmpdir/$BINARY_NAME" ] || err "binario $BINARY_NAME nao encontrado no arquivo $ARCHIVE"

mkdir -p "$BIN_DIR"
install -m 0755 "$tmpdir/$BINARY_NAME" "$BIN_DIR/$BINARY_NAME"

echo
echo "Settupper instalado em: $BIN_DIR/$BINARY_NAME"

case ":$PATH:" in
    *":$BIN_DIR:"*)
        echo "Execute com: settupper"
        ;;
    *)
        echo "Adicione ao PATH para executar apenas com 'settupper':"
        echo "  export PATH=\"$BIN_DIR:\$PATH\""
        ;;
esac
