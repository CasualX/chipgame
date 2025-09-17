@echo off

rem Clean the publish directory
rmdir /S /Q target\publish
mkdir target\publish

rem Build the executables
cargo build --release --bin chipplay
cargo build --release --bin chipedit

rem Copy the executables to the publish directory
copy target\release\chipplay.exe target\publish
copy target\release\chipedit.exe target\publish

rem Package the assets
pakscmd target/publish/data.paks 0 new
pakscmd target/publish/data.paks 0 copy "" data

rem Package the levelsets
mkdir target\publish\levelsets
cargo run --release --bin packset levelsets\cclp1 target\publish\levelsets\cclp1.paks
cargo run --release --bin packset levelsets\cclp3 target\publish\levelsets\cclp3.paks
cargo run --release --bin packset levelsets\cclp4 target\publish\levelsets\cclp4.paks
cargo run --release --bin packset levelsets\cclp5 target\publish\levelsets\cclp5.paks

rem Create the save dir
mkdir target\publish\save

makurust levelsets\readme.md
move levelsets\readme.html target\publish\levelsets\readme.html

makurust chipgame.md
move chipgame.html target\publish\readme.html
