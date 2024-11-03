Chip's Challenge Level Extractor
================================

Extract level data from .dat files and resave them in this project's format.

### Linux

```bash
# Select the .dat file, level pack location and level number
CCDAT=tmp/CCLP1_zip/data/CCLP1.dat
LEVEL_PACK=data/packs/cclp1
N=24

# Extract level and edit
clear && cargo run --bin ccdat -- -f $CCDAT -n $N > $LEVEL_PACK/lv/level$N.json && cargo run --bin edit -- $LEVEL_PACK/lv/level$N.json

# Edit level
clear && cargo run --bin edit -- $LEVEL_PACK/lv/level$N.json
```

### Windows

```cmd
rem Select the .dat file, level pack location and level number
set CCDAT=tmp\CCLP1_zip\data\CCLP1.dat
set LEVEL_PACK=data\packs\cclp1
set N=24

rem Extract level and edit
cargo run --bin ccdat -- -f %CCDAT% -n %N% > %LEVEL_PACK%\lv\level%N%.json && cargo run --bin edit -- %LEVEL_PACK%\lv\level%N%.json

rem Edit level
cargo run --bin edit -- %LEVEL_PACK%\lv\level%N%.json
```
