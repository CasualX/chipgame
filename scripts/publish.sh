#!/usr/bin/env bash

set -euo pipefail

# Require a clean git checkout (no staged/unstaged changes; untracked OK)
if ! git diff-index --quiet HEAD --; then
	echo "Error: git working tree is dirty (staged or unstaged changes)." >&2
	echo "Please commit, stash, or discard changes before publishing." >&2
	exit 1
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
