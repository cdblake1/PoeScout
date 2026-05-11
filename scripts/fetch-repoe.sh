#!/usr/bin/env bash
set -euo pipefail

DATA_DIR="${1:-data}"
BASE_URL="https://raw.githubusercontent.com/brather1ng/RePoE/master/RePoE/data"

FILES=(
  "mods.json"
  "base_items.json"
  "fossils.json"
  "essences.json"
)

mkdir -p "$DATA_DIR"

for file in "${FILES[@]}"; do
  if [ -f "$DATA_DIR/$file" ]; then
    echo "Already exists: $file"
  else
    echo "Downloading $file..."
    curl -sL "$BASE_URL/$file" -o "$DATA_DIR/$file"
    echo "Downloaded $file ($(wc -c < "$DATA_DIR/$file") bytes)"
  fi
done

echo "Done. Data files in $DATA_DIR/"
