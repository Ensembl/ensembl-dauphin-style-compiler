#! /bin/bash

set -e

RUSTFLAGS="--cfg=console" wasm-pack build --target web --dev
#RUSTFLAGS="--cfg=console" wasm-pack build --target web --release
python3 -m http.server 8000
