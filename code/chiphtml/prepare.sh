#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"

cd "$repo_root"

rm -rf target/publish
mkdir -p target/publish

pakscmd target/publish/data.paks 0 new
pakscmd target/publish/data.paks 0 copy "" data

mkdir -p target/publish/levelsets
cargo run --bin packset levelsets/cclp1 target/publish/levelsets/cclp1.paks
cargo run --bin packset levelsets/cclp2 target/publish/levelsets/cclp2.paks
cargo run --bin packset levelsets/cclp3 target/publish/levelsets/cclp3.paks
cargo run --bin packset levelsets/cclp4 target/publish/levelsets/cclp4.paks
cargo run --bin packset levelsets/cclp5 target/publish/levelsets/cclp5.paks