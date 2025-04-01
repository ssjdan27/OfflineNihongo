#!/bin/bash

SOURCE_DIR="$HOME/Downloads/kanji"
DEST_DIR="$HOME/Repos/OfflineNihongo/src-tauri/data/kanji_svg"

mkdir -p "$DEST_DIR"

echo "Moving SVG files from '$SOURCE_DIR' to '$DEST_DIR'..."
find "$SOURCE_DIR" -type f -name "*.svg" -exec mv {} "$DEST_DIR" \;

echo "Done moving SVG files!"
