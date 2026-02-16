#!/bin/bash
echo "[INFO] Starting TUI-GAME installation for macOS..."

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

# Step 3: Download package
echo "[INFO] Downloading game package..."
TEMP_ZIP=$(mktemp).zip
if ! curl -s -L -o "$TEMP_ZIP" "$DOWNLOAD_URL"; then
    echo "[ERROR] Failed to download game package."
    rm -f "$TEMP_ZIP"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 4: Extract to current directory
echo "[INFO] Extracting files to $SCRIPT_DIR ..."
if ! unzip -o "$TEMP_ZIP" -d "$SCRIPT_DIR"; then
    echo "[ERROR] Failed to extract game package."
    rm -f "$TEMP_ZIP"
    read -n1 -r -p "Press any key to exit..."
    exit 1
fi

# Step 5: Set execute permissions for binary and all shell scripts
chmod +x "$SCRIPT_DIR/tui-game" "$SCRIPT_DIR"/*.sh 2>/dev/null || true

# Clean up temporary file
rm -f "$TEMP_ZIP"
echo "[INFO] Temporary files cleaned up."

# Step 6: Ask about adding to PATH (via symlink)
echo
read -p "Do you want to create a symbolic link to 'tg.sh' in a directory in your PATH (e.g., /usr/local/bin)? (Y/N): " ADD_PATH
if [[ "$ADD_PATH" =~ ^[Yy]$ ]]; then
    echo "[INFO] Setting up command 'tg'..."
    # Try to create symlink in /usr/local/bin (may need sudo)
    TARGET_DIR="/usr/local/bin"
    LINK_PATH="$TARGET_DIR/tg"
    if [ -e "$LINK_PATH" ]; then
        echo "[INFO] $LINK_PATH already exists. Skipping."
    else
        # Attempt to create symlink (may fail due to permissions)
        if ln -s "$SCRIPT_DIR/tg.sh" "$LINK_PATH" 2>/dev/null; then
            echo "[SUCCESS] Created symlink: $LINK_PATH -> $SCRIPT_DIR/tg.sh"
        else
            echo "[WARNING] Failed to create symlink in $TARGET_DIR (permission denied). Trying ~/bin..."
            # Fallback to ~/bin
            mkdir -p "$HOME/bin"
            LINK_PATH="$HOME/bin/tg"
            if [ -e "$LINK_PATH" ]; then
                echo "[INFO] $LINK_PATH already exists. Skipping."
            else
                ln -s "$SCRIPT_DIR/tg.sh" "$LINK_PATH"
                if [ $? -eq 0 ]; then
                    echo "[SUCCESS] Created symlink: $LINK_PATH -> $SCRIPT_DIR/tg.sh"
                    # Ensure ~/bin is in PATH
                    if [[ ":$PATH:" != *":$HOME/bin:"* ]]; then
                        echo "[INFO] Note: $HOME/bin may not be in your PATH. Please add it manually."
                    fi
                else
                    echo "[ERROR] Failed to create symlink. You may need to create it manually."
                fi
            fi
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
echo "[INFO] Enjoy the game!"
echo "[INFO] If you like it, please give a star on GitHub: https://github.com/MXBraisedFish/TUI-GAME"
echo "[INFO] Author: MXBraisedFish (MXFish)"

if [ "$REG_OPTION" = "yes" ]; then
    echo "[INFO] You can start the game by typing 'tg' in any terminal (ensure the symlink directory is in PATH)."
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