#!/bin/bash -e
cargo wasix build --no-default-features --release

PWD=$(pwd)
cd /prog/wasmer/lib/cli
cargo run --release --features compiler,cranelift \
  -- run --net --mapdir /public:/prog/deploy/wasmer-web/wapm/public /prog/static-web-server/target/wasm32-wasmer-wasi/release/static-web-server.wasm \
  -- -p 9080 --log-level warn
cd $PWD