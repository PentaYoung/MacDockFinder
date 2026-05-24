#!/bin/bash
# Build MacDockFinder for Windows (cross-compile from WSL/Linux)
# Requires: x86_64-pc-windows-gnu rust target + mingw-w64 toolchain

set -e

. "$HOME/.cargo/env"

echo "=== Step 1: Ensure Windows target is installed ==="
rustup target add x86_64-pc-windows-gnu 2>/dev/null || true

echo "=== Step 2: Check mingw-w64 toolchain ==="
if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
  echo "mingw-w64 not found. Install it:"
  echo "  sudo apt install mingw-w64"
  echo "Or use the pre-extracted toolchain at /tmp/mingw-prefix"
  if [ -d /tmp/mingw-prefix/usr/bin ]; then
    export PATH="/tmp/mingw-prefix/usr/bin:$PATH"
    echo "Using toolchain at /tmp/mingw-prefix"
  else
    exit 1
  fi
fi

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== Step 3: Build frontend ==="
cd "$PROJECT_ROOT"
npm run build

echo "=== Step 4: Build with tauri build (production) ==="
cd "$PROJECT_ROOT"
export PATH="/tmp/mingw-prefix/usr/bin:$PATH"
npx tauri build --no-bundle --target x86_64-pc-windows-gnu

echo ""
echo "=== Done ==="
echo "Binary: $PROJECT_ROOT/src-tauri/target/x86_64-pc-windows-gnu/release/macdockfinder.exe"
ls -lh "$PROJECT_ROOT/src-tauri/target/x86_64-pc-windows-gnu/release/macdockfinder.exe"
