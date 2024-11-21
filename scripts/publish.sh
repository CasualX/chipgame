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

# Copy the levelsets to the publish dir
mkdir -p target/publish/levelsets
cp -r levelsets/cc1 target/publish/levelsets/cc1
cp -r levelsets/cclp1 target/publish/levelsets/cclp1
makurust levelsets/readme.md
mv levelsets/readme.html target/publish/levelsets

makurust chipgame.md
mv chipgame.html target/publish/readme.html

# Create folders for the save and replay files
mkdir -p target/publish/save/replay
