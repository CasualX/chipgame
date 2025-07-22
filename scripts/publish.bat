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

rem Copy the assets to the publish dir
xcopy data target\publish\data /E /Y /I /Q
rem xcopy levelsets\cc1 target\publish\levelsets\cc1 /E /Y /I /Q
xcopy levelsets\cclp1 target\publish\levelsets\cclp1 /E /Y /I /Q
xcopy levelsets\cclp3 target\publish\levelsets\cclp3 /E /Y /I /Q
xcopy levelsets\cclp4 target\publish\levelsets\cclp4 /E /Y /I /Q
xcopy levelsets\cclp5 target\publish\levelsets\cclp5 /E /Y /I /Q
makurust levelsets\readme.md
move levelsets\readme.html target\publish\levelsets\readme.html

makurust chipgame.md
move chipgame.html target\publish\readme.html
