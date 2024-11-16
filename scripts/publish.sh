# bin/bash

# Build the executables
cargo build --release --bin play
cargo build --release --bin edit

# Create a publish dir
mkdir -p target/publish

# Copy the executables to the publish dir
cp target/release/play target/publish
cp target/release/edit target/publish

# Copy the assets to the publish dir
cp -r data target/publish

# Create folders for the save and replay files
mkdir -p target/publish/save
mkdir -p target/publish/replay
