#! /bin/bash

set -e

#RUSTFLAGS="" wasm-pack build --target web 
RUSTFLAGS="--cfg=console --cfg=blackbox" wasm-pack build --target web --dev
#RUSTFLAGS="--cfg=console --cfg=blackbox --cfg=debug_webgl" wasm-pack build --target web --dev
#RUSTFLAGS="--cfg=console" wasm-pack build --target web --release
python3 -m http.server 8000