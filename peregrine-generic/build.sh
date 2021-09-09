#! /bin/bash

set -e

#RUSTFLAGS="" wasm-pack build --target web 
#RUSTFLAGS="--cfg=console" wasm-pack build --target web --dev
#RUSTFLAGS="--cfg=console --cfg=debug_webgl" wasm-pack build --target web --dev
RUSTFLAGS="--cfg=console --cfg=force_show_incoming" wasm-pack build --target web --release
python3 server.py

