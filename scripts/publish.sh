#!/usr/bin/env bash

set -euo pipefail

allow_dirty=0

while [[ $# -gt 0 ]]; do
	case "$1" in
		--allow-dirty)
			allow_dirty=1
			shift
			;;
		-h|--help)
			echo "Usage: $(basename "$0") [--allow-dirty]" >&2
			exit 0
			;;
		*)
			echo "Error: unknown argument: $1" >&2
			echo "Usage: $(basename "$0") [--allow-dirty]" >&2
			exit 2
			;;
	esac
done

# Require a clean git checkout (no staged/unstaged changes; untracked OK)
if [[ $allow_dirty -eq 0 ]]; then
	if ! git diff-index --quiet HEAD --; then
		echo "Error: git working tree is dirty (staged or unstaged changes)." >&2
		echo "Re-run with --allow-dirty to bypass this check." >&2
		echo "(Tip: untracked files are already allowed.)" >&2
		exit 1
	fi
fi

# Clean the publish directory
rm -rf target/publish
mkdir -p target/publish

# Build the executables
cargo build --release --bin chipplay
cargo build --release --bin chipedit

# Copy the executables to the publish dir
cp target/release/chipplay target/publish
cp target/release/chipedit target/publish

# Copy the config
cp chipgame.ini target/publish

# Package the assets
pakscmd target/publish/data.paks 0 new
pakscmd target/publish/data.paks 0 copy "" data

# Package the levelsets
mkdir -p target/publish/levelsets
cargo run --bin packset levelsets/cclp1 target/publish/levelsets/cclp1.paks
cargo run --bin packset levelsets/cclp2 target/publish/levelsets/cclp2.paks
cargo run --bin packset levelsets/cclp3 target/publish/levelsets/cclp3.paks
cargo run --bin packset levelsets/cclp4 target/publish/levelsets/cclp4.paks
cargo run --bin packset levelsets/cclp5 target/publish/levelsets/cclp5.paks

# Create the save dir
mkdir -p target/publish/save

# Generate the documentation
cargo run --bin makedocs

# Zip it all up
rm -f target/chipgame.zip
( cd target/publish && zip -r ../chipgame.zip . )
