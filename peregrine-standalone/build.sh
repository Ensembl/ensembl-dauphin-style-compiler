#! /bin/bash

set -e

#RUSTFLAGS="" wasm-pack build --target web 
RUSTFLAGS="--cfg=console --cfg=blackbox" wasm-pack build --target web --dev
python3 -m http.server 8000