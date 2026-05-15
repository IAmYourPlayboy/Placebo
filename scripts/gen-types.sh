#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Locate cargo: prefer rustup-installed cargo at ~/.cargo/bin/cargo, fall back to PATH.
if [ -x "$HOME/.cargo/bin/cargo" ]; then
  CARGO="$HOME/.cargo/bin/cargo"
elif [ -x "$HOME/.cargo/bin/cargo.exe" ]; then
  CARGO="$HOME/.cargo/bin/cargo.exe"
else
  CARGO="cargo"
fi

echo "[gen-types] running cargo test export_bindings ..."
"$CARGO" test -p placebo-shared --features export-types export_bindings -- --nocapture || true

SRC="$ROOT/crates/placebo-shared/bindings"
DST="$ROOT/src/types/api"

if [ -d "$SRC" ]; then
  echo "[gen-types] copying bindings -> $DST"
  mkdir -p "$DST"
  # Remove stale .ts files (keep README.md).
  find "$DST" -type f -name "*.ts" -delete
  cp -R "$SRC"/. "$DST"/
else
  echo "[gen-types] no bindings/ yet, nothing to copy (OK on first run)."
fi

echo "[gen-types] done"
