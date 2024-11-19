# bin/bash

# Clean the publish directory
rm -rf target/publish
mkdir -p target/publish

# Build the executables
cargo build --release --bin chipplay
cargo build --release --bin chipedit

# Copy the executables to the publish dir
cp target/release/chipplay target/publish
cp target/release/chipedit target/publish

# Copy the assets to the publish dir
cp -r data target/publish
cp -r levelsets target/publish

# Create folders for the save and replay files
mkdir -p target/publish/save/replay
