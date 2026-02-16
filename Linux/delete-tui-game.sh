#!/bin/bash
echo "[WARNING] This script will permanently delete game files and may remove saved data."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR" || exit 1

# Step 1: Confirmation
read -p "Are you sure you want to delete all game files? (Y/N): " CONFIRM
if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    echo "[INFO] Deletion cancelled by user."
    read -n1 -r -p "Press any key to exit..."
    exit 0
fi

# Step 2: Delete specific files and folders
echo "[INFO] Deleting game files from current directory..."

# tui-game
if [ -f "tui-game" ]; then
    rm -f "tui-game" && echo "[OK] Deleted tui-game" || echo "[ERROR] Failed to delete tui-game"
else
    echo "[INFO] tui-game not found, skipping."
fi

# assets folder
if [ -d "assets" ]; then
    rm -rf "assets" && echo "[OK] Deleted assets folder" || echo "[ERROR] Failed to delete assets folder"
else
    echo "[INFO] assets folder not found, skipping."
fi

# scripts folder
if [ -d "scripts" ]; then
    rm -rf "scripts" && echo "[OK] Deleted scripts folder" || echo "[ERROR] Failed to delete scripts folder"
else
    echo "[INFO] scripts folder not found, skipping."
fi

# version.sh
if [ -f "version.sh" ]; then
    rm -f "version.sh" && echo "[OK] Deleted version.sh" || echo "[ERROR] Failed to delete version.sh"
else
    echo "[INFO] version.sh not found, skipping."
fi

# tg.sh (current script will be deleted later)
if [ -f "tg.sh" ]; then
    rm -f "tg.sh" && echo "[OK] Deleted tg.sh" || echo "[ERROR] Failed to delete tg.sh"
else
    echo "[INFO] tg.sh not found, skipping."
fi

# Step 3: System integration cleanup (optional)
echo
read -p "Do you want to remove any system integration (e.g., PATH symlinks, desktop entries) for this game? (Y/N): " REG_CONFIRM
if [[ "$REG_CONFIRM" =~ ^[Yy]$ ]]; then
    echo "[INFO] Removing system integration..."
    # Check for common symlink locations
    SYMLINK_PATHS=("/usr/local/bin/tg" "$HOME/.local/bin/tg" "/usr/bin/tg")
    for link in "${SYMLINK_PATHS[@]}"; do
        if [ -L "$link" ] && [ "$(readlink -f "$link")" = "$SCRIPT_DIR/tg.sh" ]; then
            rm -f "$link" 2>/dev/null && echo "[OK] Removed $link" || echo "[ERROR] Failed to remove $link"
        fi
    done
    # Check for .desktop files
    DESKTOP_FILE="$HOME/.local/share/applications/tui-game.desktop"
    if [ -f "$DESKTOP_FILE" ]; then
        rm -f "$DESKTOP_FILE" && echo "[OK] Removed desktop entry" || echo "[ERROR] Failed to remove desktop entry"
    fi
    echo "[INFO] System integration cleanup finished."
    REG_REMOVED=1
else
    REG_REMOVED=0
fi

# Step 4: Final messages
echo
echo "[INFO] Goodbye!"
if [ $REG_REMOVED -eq 0 ]; then
    echo "[REMINDER] Please manually remove any system integration (symlinks, desktop entries) if needed."
fi

# Step 5: Wait for key press and self-delete
echo "[INFO] Press any key to close this window and delete this script."
read -n1 -r

# Delete this script itself
rm -f "$0" && echo "[INFO] Self-deletion successful." || echo "[ERROR] Failed to delete this script. Please remove it manually."

exit 0