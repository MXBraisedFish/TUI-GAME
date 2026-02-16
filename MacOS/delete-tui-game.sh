#!/bin/bash
set -u

echo "[WARNING] This script will permanently delete TUI-GAME runtime files."
echo "[WARNING] Save data in tui-game-data will also be removed."
echo
read -r -p "Continue uninstall? (Y/N): " CONFIRM
if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    echo "[INFO] Uninstall cancelled."
    read -n1 -r -p "Press any key to exit..."
    exit 0
fi

SOURCE="${BASH_SOURCE[0]}"
while [ -L "$SOURCE" ]; do
    DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
    TARGET="$(readlink "$SOURCE")"
    if [[ "$TARGET" != /* ]]; then
        SOURCE="$DIR/$TARGET"
    else
        SOURCE="$TARGET"
    fi
done
SCRIPT_DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
cd "$SCRIPT_DIR" || exit 1

HAS_ERROR=0

delete_file() {
    local file="$1"
    if [ -f "$file" ]; then
        if rm -f "$file"; then
            echo "[OK] Deleted file: $file"
        else
            echo "[ERROR] Failed to delete file: $file"
            HAS_ERROR=1
        fi
    else
        echo "[INFO] File not found, skip: $file"
    fi
}

delete_dir() {
    local dir="$1"
    if [ -d "$dir" ]; then
        if rm -rf "$dir"; then
            echo "[OK] Deleted folder: $dir"
        else
            echo "[ERROR] Failed to delete folder: $dir"
            HAS_ERROR=1
        fi
    else
        echo "[INFO] Folder not found, skip: $dir"
    fi
}

echo "[INFO] Working directory: $SCRIPT_DIR"
delete_file "$SCRIPT_DIR/tui-game"
delete_dir "$SCRIPT_DIR/tui-game-data"
delete_file "$SCRIPT_DIR/version.sh"
delete_file "$SCRIPT_DIR/tg.sh"
delete_dir "$SCRIPT_DIR/assets"
delete_dir "$SCRIPT_DIR/scripts"

echo
read -r -p "Remove system integration (tg symlink/app link)? (Y/N): " CLEAN_SYS
if [[ "$CLEAN_SYS" =~ ^[Yy]$ ]]; then
    echo "[INFO] Cleaning system integration..."
    expected="$SCRIPT_DIR/tg.sh"
    for link in "/usr/local/bin/tg" "$HOME/bin/tg" "$HOME/.local/bin/tg"; do
        if [ -L "$link" ]; then
            target="$(readlink "$link" 2>/dev/null || true)"
            if [ -z "$target" ]; then
                continue
            fi
            if [[ "$target" != /* ]]; then
                target="$(cd -P "$(dirname "$link")" && pwd)/$target"
            fi
            target_dir="$(cd -P "$(dirname "$target")" 2>/dev/null && pwd || true)"
            if [ -z "$target_dir" ]; then
                continue
            fi
            resolved="$target_dir/$(basename "$target")"
            if [ "$resolved" = "$expected" ]; then
                rm -f "$link" && echo "[OK] Removed symlink: $link" || echo "[WARNING] Failed to remove symlink: $link"
            fi
        fi
    done
    app_link="$HOME/Applications/TUI-GAME"
    if [ -L "$app_link" ]; then
        rm -f "$app_link" && echo "[OK] Removed application link." || echo "[WARNING] Failed to remove application link."
    fi
else
    echo "[INFO] Skipped system integration cleanup."
fi

echo
if [ "$HAS_ERROR" -eq 1 ]; then
    echo "[WARNING] Uninstall completed with errors. Some files may remain."
else
    echo "[SUCCESS] Uninstall completed."
fi

echo "[INFO] Press any key to exit and remove this script."
read -n1 -r
rm -f "$0"
exit 0
