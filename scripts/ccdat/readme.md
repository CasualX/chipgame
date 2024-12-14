Chip's Challenge Level Extractor
================================

Extract level data from .dat files and resave them in this project's format.

### Linux

```bash
# Select the .dat file, level pack location and level number
CCDAT=tmp/CCLP4/data/CCLP4.dat
LEVEL_PACK=levelsets/cclp4
N=1

function ccextract() {
	cargo run --bin ccdat_extract -- -f $CCDAT -n $1 > $LEVEL_PACK/lv/level$1.json
}
function ccedit() {
	cargo run --bin chipedit -- $LEVEL_PACK/lv/level$1.json
}

# Extract level and edit
clear && ccextract $N && ccedit $N

# Edit level
clear && ccedit $N
```

### Windows

```cmd
rem Select the .dat file, level pack location and level number
set CCDAT=tmp\CCLP1_zip\data\CCLP1.dat
set LEVEL_PACK=data\packs\cclp1
set N=24

rem Extract level and edit
cargo run --bin ccdat -- -f %CCDAT% -n %N% > %LEVEL_PACK%\lv\level%N%.json && cargo run --bin chipedit -- %LEVEL_PACK%\lv\level%N%.json

rem Edit level
cargo run --bin chipedit -- %LEVEL_PACK%\lv\level%N%.json
```

References
----------

* https://wiki.bitbusters.club/DAT
