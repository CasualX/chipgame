#!/usr/bin/env bash

# Clean the publish directory
rm -rf target/publish
mkdir -p target/publish

# Build the executables
cargo build --release --bin chipplay
cargo build --release --bin chipedit

# Copy the executables to the publish dir
cp target/release/chipplay target/publish
cp target/release/chipedit target/publish

# Package the assets
PAKStool target/publish/data.paks 0 new
PAKStool target/publish/data.paks 0 copy "" data

# Package the levelsets
mkdir -p target/publish/levelsets
cargo run --bin packset levelsets/cclp1 target/publish/levelsets/cclp1.paks
cargo run --bin packset levelsets/cclp3 target/publish/levelsets/cclp3.paks
cargo run --bin packset levelsets/cclp4 target/publish/levelsets/cclp4.paks
cargo run --bin packset levelsets/cclp5 target/publish/levelsets/cclp5.paks

# Create the save dir
mkdir -p target/publish/save

makurust levelsets/readme.md
mv levelsets/readme.html target/publish/levelsets

makurust chipgame.md
mv chipgame.html target/publish/readme.html
