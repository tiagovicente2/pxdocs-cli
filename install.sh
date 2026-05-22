#!/usr/bin/env bash
set -euo pipefail

REPO="${PXDOCS_REPO:-tiagovicente2/pxdocs-cli}"
APP_NAME="pxdocs"
BIN_DIR="${PXDOCS_BIN_DIR:-$HOME/.local/bin}"

log() { printf '[pxdocs] %s\n' "$*"; }
fail() { printf '[pxdocs] error: %s\n' "$*" >&2; exit 1; }

command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v tar >/dev/null 2>&1 || fail "tar is required"

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$os" in
  linux) platform="linux" ;;
  darwin) platform="macos" ;;
  *) fail "unsupported OS: $os" ;;
esac
case "$arch" in
  x86_64|amd64) arch="x64" ;;
  arm64|aarch64) arch="arm64" ;;
  *) fail "unsupported architecture: $arch" ;;
esac

artifact="${APP_NAME}-${platform}-${arch}.tar.gz"
url="https://github.com/${REPO}/releases/latest/download/${artifact}"
checksum_url="https://github.com/${REPO}/releases/latest/download/SHA256SUMS"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

log "downloading ${url}"
curl -fL "$url" -o "$tmp_dir/$artifact"

if curl -fsL "$checksum_url" -o "$tmp_dir/SHA256SUMS"; then
  expected_checksum="$(awk -v artifact="$artifact" '$2 == artifact { print $1 }' "$tmp_dir/SHA256SUMS")"
  if [[ -n "$expected_checksum" ]]; then
    if command -v sha256sum >/dev/null 2>&1; then
      actual_checksum="$(sha256sum "$tmp_dir/$artifact" | awk '{ print $1 }')"
    else
      actual_checksum="$(shasum -a 256 "$tmp_dir/$artifact" | awk '{ print $1 }')"
    fi
    [[ "$actual_checksum" == "$expected_checksum" ]] || fail "checksum verification failed for $artifact"
    log "verified checksum for $artifact"
  else
    log "checksum file did not include $artifact; skipping verification"
  fi
else
  log "checksums unavailable; skipping verification"
fi

extract_dir="$tmp_dir/extract"
mkdir -p "$extract_dir"
tar -xzf "$tmp_dir/$artifact" -C "$extract_dir" --strip-components=1

launcher="$extract_dir/$APP_NAME"
if [[ ! -x "$launcher" ]]; then
  launcher="$(find "$extract_dir" -maxdepth 2 -type f -perm -111 -name "$APP_NAME" | head -n 1 || true)"
fi
[[ -n "$launcher" && -x "$launcher" ]] || fail "executable not found in $artifact"

mkdir -p "$BIN_DIR"
cp "$launcher" "$BIN_DIR/$APP_NAME"
chmod +x "$BIN_DIR/$APP_NAME"

log "installed: $BIN_DIR/$APP_NAME"
if ! command -v "$APP_NAME" >/dev/null 2>&1; then
  log "warning: $BIN_DIR is not in PATH"
fi
log "done"
