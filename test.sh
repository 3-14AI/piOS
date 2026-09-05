export XDG_RUNTIME_DIR=/tmp/xdg && mkdir -p $XDG_RUNTIME_DIR && chmod 700 $XDG_RUNTIME_DIR
RUSTC_BOOTSTRAP=1 cargo +1.95.0-x86_64-unknown-linux-gnu test --workspace --no-fail-fast || true
