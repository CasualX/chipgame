#!/usr/bin/env bash

set -euo pipefail

cargo build --release -p chipwasm --target=wasm32-unknown-unknown

cp target/wasm32-unknown-unknown/release/chipwasm.wasm chiphtml/chipwasm.wasm
