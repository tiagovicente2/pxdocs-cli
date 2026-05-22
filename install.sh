#!/usr/bin/env bash
set -euo pipefail

APP_NAME="pxdocs"
BIN_DIR="${PXDOCS_BIN_DIR:-$HOME/.local/bin}"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

log() { printf '[pxdocs] %s\n' "$*"; }
fail() { printf '[pxdocs] error: %s\n' "$*" >&2; exit 1; }

command -v cargo >/dev/null 2>&1 || fail "cargo is required"

cd "$REPO_DIR"
log "building release binary"
cargo build --release --bin "$APP_NAME"

mkdir -p "$BIN_DIR"
cp "$REPO_DIR/target/release/$APP_NAME" "$BIN_DIR/$APP_NAME"
chmod +x "$BIN_DIR/$APP_NAME"

log "installed: $BIN_DIR/$APP_NAME"
if ! command -v "$APP_NAME" >/dev/null 2>&1; then
  log "warning: $BIN_DIR is not in PATH"
  log "add this to your shell startup file: export PATH=\"$BIN_DIR:\$PATH\""
else
  log "available on PATH: $(command -v "$APP_NAME")"
fi
log "done"
