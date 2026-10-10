#!/usr/bin/env bash
# Build and run the preserved playground, including on Apple Silicon.
# Prerequisites: rustup, Node.js, npm. Verified with Node 24.21.0.
set -euo pipefail
cd "$(dirname "$0")"

toolchain=nightly-2024-04-05
rustup toolchain install "$toolchain" --profile minimal --target wasm32-unknown-unknown
if [[ ! -x target/demo-tools/bin/wasm-bindgen ]] ||
   [[ "$(target/demo-tools/bin/wasm-bindgen --version)" != "wasm-bindgen 0.2.92" ]]; then
    cargo +"$toolchain" install wasm-bindgen-cli --version 0.2.92 --locked --root target/demo-tools
fi
cargo +"$toolchain" build --locked -p dada-web --target wasm32-unknown-unknown
target/demo-tools/bin/wasm-bindgen --target web --out-dir target/dada-web \
    --out-name dada_web target/wasm32-unknown-unknown/debug/dada_web.wasm
cat > target/dada-web/package.json <<'JSON'
{"name":"dada-web","version":"0.1.0","module":"dada_web.js","types":"dada_web.d.ts","sideEffects":["./snippets/*"]}
JSON
cd book
npm ci --no-audit --no-fund
exec npm start -- --host 127.0.0.1 --port "${PORT:-3000}" --no-open
