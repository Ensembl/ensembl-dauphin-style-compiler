#! /bin/bash

RUSTFLAGS="--cfg=console --cfg=blackbox" wasm-pack build --target web --release
python3 -m http.server 8000

