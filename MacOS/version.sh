#!/bin/bash
echo "[INFO] Starting update process..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR" || { echo "[ERROR] Failed to change to script directory."; exit 1; }

# Step 1: Fetch latest release info
echo "[INFO] Fetching latest release information from GitHub..."
API_URL="https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest"
TEMP_JSON=$(mktemp)

if ! curl -s -L -o "$TEMP_JSON" "$API_URL"; then
    echo "[ERROR] Failed to download release information. Check your internet connection."
    rm -f "$TEMP_JSON"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 2: Extract macOS asset download URL using awk
echo "[INFO] Extracting download URL for macOS package..."
ASSET_NAME="tui-game-macos.zip"
DOWNLOAD_URL=$(awk -v name="$ASSET_NAME" '
    $0 ~ "\"name\": \"" name "\"" {found=1}
    found && $0 ~ "\"browser_download_url\"" {
        gsub(/[",]/, "")
        split($0, a, ": ")
        url=a[2]
        gsub(/^[ \t]+|[ \t]+$/, "", url)
        print url
        exit
    }
' "$TEMP_JSON")

if [ -z "$DOWNLOAD_URL" ]; then
    echo "[ERROR] Could not find macOS asset '$ASSET_NAME' in the latest release."
    rm -f "$TEMP_JSON"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi
echo "[INFO] Download URL: $DOWNLOAD_URL"
rm -f "$TEMP_JSON"

# Step 3: Download the package
echo "[INFO] Downloading update package..."
TEMP_ZIP=$(mktemp).zip
if ! curl -s -L -o "$TEMP_ZIP" "$DOWNLOAD_URL"; then
    echo "[ERROR] Failed to download update package."
    rm -f "$TEMP_ZIP"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 4: Extract to current directory, overwriting
echo "[INFO] Extracting update to current directory (overwriting files)..."
if ! unzip -o "$TEMP_ZIP" -d "$SCRIPT_DIR"; then
    echo "[ERROR] Failed to extract update package."
    rm -f "$TEMP_ZIP"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 5: Clean up temporary files
rm -f "$TEMP_ZIP"
echo "[INFO] Temporary files cleaned up."

# Step 6: Ensure extracted files have execute permissions (tui-game and scripts)
chmod +x "$SCRIPT_DIR/tui-game" "$SCRIPT_DIR"/*.sh 2>/dev/null || true

echo "[SUCCESS] Update completed successfully!"
echo "[INFO] Press any key to exit."
read -n1 -r
exit 0