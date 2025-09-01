#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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
"$SCRIPT_DIR/createpak.sh" target/publish/data.paks 0 "" data/

# Package the levelsets
mkdir -p target/publish/levelsets
"$SCRIPT_DIR/createpak.sh" target/publish/levelsets/cclp1.paks 0 "" levelsets/cclp1/
"$SCRIPT_DIR/createpak.sh" target/publish/levelsets/cclp3.paks 0 "" levelsets/cclp3/
"$SCRIPT_DIR/createpak.sh" target/publish/levelsets/cclp4.paks 0 "" levelsets/cclp4/
"$SCRIPT_DIR/createpak.sh" target/publish/levelsets/cclp5.paks 0 "" levelsets/cclp5/

# Create the save dir
mkdir -p target/publish/save

makurust levelsets/readme.md
mv levelsets/readme.html target/publish/levelsets

makurust chipgame.md
mv chipgame.html target/publish/readme.html
