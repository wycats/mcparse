#!/bin/bash

# Update Mermaid
# Directory to store the mermaid script
DEST_DIR="docs/book/js"
mkdir -p "$DEST_DIR"

# URL for the latest mermaid.min.js (using jsdelivr to get the latest version)
MERMAID_URL="https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js"

echo "Fetching latest mermaid.min.js from $MERMAID_URL..."
curl -L "$MERMAID_URL" -o "$DEST_DIR/mermaid.min.js"

if [ $? -eq 0 ]; then
  echo "Successfully updated mermaid.min.js in $DEST_DIR"
else
  echo "Failed to download mermaid.min.js"
  exit 1
fi
