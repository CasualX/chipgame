# bin/bash

# Build the executables
cargo build --release --bin chipplay
cargo build --release --bin chipedit

# Create a publish dir
mkdir -p target/publish

# Copy the executables to the publish dir
cp target/release/chipplay target/publish
cp target/release/chipedit target/publish

# Copy the assets to the publish dir
cp -r data target/publish

# Create folders for the save and replay files
mkdir -p target/publish/save/replay
