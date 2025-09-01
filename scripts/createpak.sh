#!/usr/bin/env bash
# Generalized PAK packer with natural file sort
# Usage: ./createpak.sh <PAKFILE> <KEY> <TARGET> <SOURCE>

set -euo pipefail

if [ $# -ne 4 ]; then
  echo "Usage: $0 <PAKFILE> <KEY> <TARGET> <SOURCE>" >&2
  exit 1
fi

PAKFILE="$1"
KEY="$2"
TARGET="$3"
SOURCE="$4"

# Verify source exists
if [ ! -d "$SOURCE" ]; then
  echo "Error: Source folder '$SOURCE' does not exist." >&2
  exit 1
fi

# Normalize TARGET (strip any trailing slash)
TARGET="${TARGET%/}"

# Make PAKFILE absolute
PAKFILE="$(cd "$(dirname -- "$PAKFILE")"; pwd -P)/$(basename -- "$PAKFILE")"

# Start a fresh PAKFILE
PAKStool "$PAKFILE" "$KEY" new

# Work inside SOURCE so we get relative paths
cd "$SOURCE"

# Natural sort the file list (-V = version sort)
while IFS= read -r relpath; do
  # Strip leading "./"
  relpath="${relpath#./}"

  # Skip empty
  [ -z "$relpath" ] && continue

  # Build paks path
  if [ -n "$TARGET" ]; then
    pakpath="$TARGET/$relpath"
  else
    pakpath="$relpath"
  fi

  echo "Adding $pakpath..."
  PAKStool "$PAKFILE" "$KEY" add "$pakpath" < "$relpath"
done < <(find . -type f | sort -V)
