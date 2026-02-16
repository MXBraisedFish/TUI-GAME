#!/bin/bash
echo "[INFO] Starting TUI-GAME installation for Linux..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR" || exit 1

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

# Step 2: Extract Linux package download URL
echo "[INFO] Extracting download URL for Linux package..."
ASSET_NAME="tui-game-linux.tar.gz"
DOWNLOAD_URL=$(python3 -c "
import sys, json
try:
    with open('$TEMP_JSON') as f:
        data = json.load(f)
    for asset in data.get('assets', []):
        if asset.get('name') == '$ASSET_NAME':
            print(asset.get('browser_download_url', ''))
            break
except Exception:
    pass
")

if [ -z "$DOWNLOAD_URL" ]; then
    echo "[ERROR] Could not find Linux asset '$ASSET_NAME' in the latest release."
    rm -f "$TEMP_JSON"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi
echo "[INFO] Download URL: $DOWNLOAD_URL"
rm -f "$TEMP_JSON"

# Step 3: Download package
echo "[INFO] Downloading game package..."
TEMP_TGZ=$(mktemp).tar.gz
if ! curl -s -L -o "$TEMP_TGZ" "$DOWNLOAD_URL"; then
    echo "[ERROR] Failed to download game package."
    rm -f "$TEMP_TGZ"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 4: Extract to current directory
echo "[INFO] Extracting files to $SCRIPT_DIR ..."
if ! tar -xzf "$TEMP_TGZ" -C "$SCRIPT_DIR"; then
    echo "[ERROR] Failed to extract game package."
    rm -f "$TEMP_TGZ"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 5: Set execute permissions
chmod +x "$SCRIPT_DIR/tui-game" "$SCRIPT_DIR"/*.sh 2>/dev/null || true

# Clean up temporary file
rm -f "$TEMP_TGZ"
echo "[INFO] Temporary files cleaned up."

# Step 6: Ask about adding to PATH (via symlink)
echo
read -p "Do you want to create a symbolic link to 'tg.sh' in a directory in your PATH (e.g., ~/.local/bin)? (Y/N): " ADD_PATH
if [[ "$ADD_PATH" =~ ^[Yy]$ ]]; then
    echo "[INFO] Setting up command 'tg'..."
    TARGET_DIR="$HOME/.local/bin"
    mkdir -p "$TARGET_DIR"
    LINK_PATH="$TARGET_DIR/tg"
    if [ -e "$LINK_PATH" ]; then
        echo "[INFO] $LINK_PATH already exists. Skipping."
    else
        ln -s "$SCRIPT_DIR/tg.sh" "$LINK_PATH"
        if [ $? -eq 0 ]; then
            echo "[SUCCESS] Created symlink: $LINK_PATH -> $SCRIPT_DIR/tg.sh"
            # Ensure ~/.local/bin is in PATH
            if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
                echo "[INFO] Note: $HOME/.local/bin may not be in your PATH. Please add it manually."
            fi
        else
            echo "[ERROR] Failed to create symlink. You may need to run with appropriate permissions."
        fi
    fi
    REG_OPTION="yes"
else
    echo "[INFO] Skipping symlink creation."
    REG_OPTION="no"
fi

# Step 7: Final messages
echo
echo "[SUCCESS] TUI-GAME has been installed successfully!"
echo =================================
echo "[INFO] Enjoy the game! :)"
echo
echo "[INFO] If you like it, please give a star on GitHub: https://github.com/MXBraisedFish/TUI-GAME"
echo "[INFO] Author: MXBraisedFish (MXFish)"
echo =================================

if [ "$REG_OPTION" = "yes" ]; then
    echo "[INFO] You can start the game by typing 'tg' in any terminal (ensure ~/.local/bin is in PATH)."
else
    echo "[INFO] To start the game easily from anywhere, you can add this folder to your PATH or create a symlink, then use 'tg' command."
    echo "[INFO] Current folder: $SCRIPT_DIR"
fi

echo
echo "[INFO] Press any key to exit and delete this installer."
read -n1 -r

# Delete this script itself
rm -f "$0" && echo "[INFO] Installer removed." || echo "[ERROR] Failed to delete installer. Please remove it manually."

exit 0