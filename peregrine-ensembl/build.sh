#! /bin/bash

set -e

RUSTFLAGS="--cfg=console -cfg=force_show_incoming" wasm-pack build --target web --dev
#RUSTFLAGS="--cfg=console -cfg=force_show_incoming" wasm-pack build --target web --release
python3 -m http.server 8000
