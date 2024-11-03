Chip's Challenge Level Extractor
================================

Workflow

```bash
CCDAT=data/CCLP1_zip/data/CCLP1.dat
LEVEL_PACK=data/cclp1

# Select level to edit
N=24

# Extract level and edit
clear && cargo run --bin ccdat -- -f $CCDAT -n $N > $LEVEL_PACK/lv/level$N.json && cargo run --bin edit -- $LEVEL_PACK/lv/level$N.json

# Edit level
clear && cargo run --bin edit -- $LEVEL_PACK/lv/level$N.json
```
