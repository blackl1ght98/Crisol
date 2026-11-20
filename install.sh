#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1090
        source "$HOME/.cargo/env"
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "No se encontro cargo. Instala Rust con:" >&2
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal" >&2
    exit 1
fi

cargo build --release --manifest-path "$ROOT/Cargo.toml"

install -Dm755 "$ROOT/target/release/crisol" "$HOME/.local/bin/crisol"
mkdir -p "$HOME/.local/share/crisol/recipes"
install -Dm644 "$ROOT/assets/crisol.svg" "$HOME/.local/share/icons/hicolor/scalable/apps/crisol.svg"

for recipe in "$ROOT"/recipes/*.recipe; do
    install -Dm644 "$recipe" "$HOME/.local/share/crisol/recipes/$(basename "$recipe")"
done

"$HOME/.local/bin/crisol" setup-desktop
"$HOME/.local/bin/crisol" doctor

echo
echo "Listo. Prueba: crisol install /ruta/a/programa.msi"
