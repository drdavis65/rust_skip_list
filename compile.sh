#!/usr/bin/env bash
set -euo pipefail

SKIP_LIST_DIR="${SKIP_LIST_DIR:-$HOME/rust_skip_list}"
CUSTOM_RUSTC="${CUSTOM_RUSTC:-$HOME/rust_compilers/rust-modified-install/usr/local/bin/rustc}"
VANILLA_RUSTC="${VANILLA_RUSTC:-$HOME/rust_compilers/rust-vanilla-install/usr/local/bin/rustc}"
OPT_LEVELS=(0 1 2 3)

# Ensure cargo is on PATH (non-interactive shells won't source this for you)
if [ -f "$HOME/.cargo/env" ]; then
  . "$HOME/.cargo/env"
else
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Verify tools exist
if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo not found. Install rustup (curl https://sh.rustup.rs | sh) then re-run." >&2
  exit 1
fi

for R in "$CUSTOM_RUSTC" "$VANILLA_RUSTC"; do
  if [ ! -x "$R" ]; then
    echo "ERROR: rustc not found at: $R" >&2
    echo "Hint: If you used DESTDIR with x.py install, rustc ends up under <DESTDIR>/usr/local/bin/rustc." >&2
    exit 1
  fi
done

# Build
pushd "$SKIP_LIST_DIR" >/dev/null

# Create binaries directory
mkdir -p binaries

for opt in "${OPT_LEVELS[@]}"; do
  echo "========== Compiling Skip List Benchmark | Opt -O$opt =========="
  
  echo "Compiling C with -O$opt"
  pushd c >/dev/null
  clang -std=c99 -O"$opt" -DNDEBUG -lm -o "../binaries/skip_list_c_O$opt" main.c
  popd >/dev/null
  echo "✓ C binary: binaries/skip_list_c_O$opt"
  echo
  
  echo "Compiling Rust VANILLA (O$opt)"
  RUSTC="$VANILLA_RUSTC" RUSTFLAGS="-C opt-level=$opt" \
    cargo build --release --target-dir "target_vanilla_O$opt"
  cp "target_vanilla_O$opt/release/skip_list_rust" "binaries/skip_list_rust_vanilla_O$opt"
  echo "✓ Vanilla Rust: binaries/skip_list_rust_vanilla_O$opt"
  echo
  
  echo "Compiling Rust CUSTOM (O$opt)"
  RUSTC="$CUSTOM_RUSTC" RUSTFLAGS="-C opt-level=$opt" \
    cargo build --release --target-dir "target_custom_O$opt"
  cp "target_custom_O$opt/release/skip_list_rust" "binaries/skip_list_rust_custom_O$opt"
  echo "✓ Custom Rust: binaries/skip_list_rust_custom_O$opt"
  echo
done

popd >/dev/null

chmod +x perf_script.sh
mv perf_script.sh binaries
rm -r target*

echo "========== Compilation Complete =========="
echo "C binaries:      $SKIP_LIST_DIR/binaries/skip_list_c_O{0,1,2,3}"
echo "Vanilla Rust:    $SKIP_LIST_DIR/binaries/skip_list_rust_vanilla_O{0,1,2,3}"
echo "Custom Rust:     $SKIP_LIST_DIR/binaries/skip_list_rust_custom_O{0,1,2,3}"
