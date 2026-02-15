#!/bin/bash

REPO_API="https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest"
ZIP_NAME="tui-game-linux.tar.gz"
EXE_NAME="tui-game"
INSTALL_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_URL="https://github.com/MXBraisedFish/TUI-GAME"

echo "[1/8] Fetching latest release info from GitHub..."
LATEST_JSON=$(curl -s "$REPO_API")
DOWNLOAD_URL=$(echo "$LATEST_JSON" | grep -o "https://[^/\" ]*browser_download_url[^\" ]*$ZIP_NAME" | head -n 1 | sed "s/.*https/https/")

if [ -z "$DOWNLOAD_URL" ]; then
    echo "[ERROR] Failed to fetch release info. Please check your network."
    read -n 1 -s -r -p "Press any key to exit..."
    exit 1
fi

echo "[2/8] Downloading $ZIP_NAME..."
curl -L -o "$ZIP_NAME" "$DOWNLOAD_URL"
if [ $? -ne 0 ]; then
    echo "[ERROR] Download failed."
    read -n 1 -s -r -p "Press any key to exit..."
    exit 1
fi

echo "[3/8] Extracting files into: $INSTALL_DIR..."
tar -xzf "$ZIP_NAME" -C "$INSTALL_DIR"
rm "$ZIP_NAME"
chmod +x "$EXE_NAME"

echo "[4/8] Creating startup script (tg.sh)..."
cat <<EOF > "tg.sh"
#!/bin/bash
cd "\$(dirname "\$0")"
./$EXE_NAME
EOF
chmod +x "tg.sh"

echo "[5/8] Environment Variable (PATH) Settings..."
read -p "Do you want to add this directory to your PATH (~/.bashrc) automatically? (Y/N): " choice
if [[ "$choice" =~ ^[Yy]$ ]]; then
    if ! grep -q "$INSTALL_DIR" ~/.bashrc 2>/dev/null; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> ~/.bashrc
        echo "[SUCCESS] Path registered to ~/.bashrc."
    else
        echo "[INFO] Path already exists in ~/.bashrc."
    fi
    ADD_TO_PATH="Y"
else
    echo "[INFO] Skipped PATH registration."
    ADD_TO_PATH="N"
fi

echo "[6/8] Generating version.sh for updates..."
cat <<EOF > "version.sh"
#!/bin/bash
echo "[UPDATE] Updating TUI-GAME..."
LATEST_JSON=\$(curl -s "$REPO_API")
TAG_NAME=\$(echo "\$LATEST_JSON" | grep -o '"tag_name": "[^"]*"' | head -n 1 | cut -d'"' -f4)
URL=\$(echo "\$LATEST_JSON" | grep -o "https://[^/\" ]*browser_download_url[^\" ]*$ZIP_NAME" | head -n 1 | sed "s/.*https/https/")

curl -L -o "$ZIP_NAME" "\$URL"
tar -xzf "$ZIP_NAME" -C "."
rm "$ZIP_NAME"
chmod +x "$EXE_NAME"

mkdir -p tui-game-data
echo "{\"version\": \"\$TAG_NAME\"}" > ./tui-game-data/updater_cache.json
echo "[UPDATE] Update done."
read -p "Press [T] to restart, or any key to exit: " up_choice
if [[ "\$up_choice" =~ ^[Tt]$ ]]; then
    ./tg.sh
fi
EOF
chmod +x "version.sh"

echo "[7/8] Generating delete-tui-game.sh (Uninstaller)..."
cat <<EOF > "delete-tui-game.sh"
#!/bin/bash
read -p "Unnesting TUI GAME will delete all game files and save data. Continue? (Y/N): " confirm
if [[ ! "\$confirm" =~ ^[Yy]$ ]]; then exit 0; fi

echo "[CLEANING] Removing game files..."
rm -rf assets scripts tui-game-data
rm tg.sh $EXE_NAME version.sh 2>/dev/null

read -p "Do you want to remove the directory from ~/.bashrc? (Y/N): " reg_confirm
if [[ "\$reg_confirm" =~ ^[Yy]$ ]]; then
    sed -i "\|$INSTALL_DIR|d" ~/.bashrc
    echo "[SUCCESS] PATH cleaned."
else
    echo "[INFO] Please clean ~/.bashrc manually if needed."
fi
echo "Bye bye."
read -n 1 -s -r -p "Press any key to exit..."
rm -- "\$0"
EOF
chmod +x "delete-tui-game.sh"

echo "[8/8] Finalizing..."
echo "------------------------------------------------------"
echo "Have fun :)"
echo "If you like this project, please give a star to my repository!"
echo "Better ideas are always welcome."
echo ""
echo "Github: $REPO_URL"
echo "------------------------------------------------------"

if [ "$ADD_TO_PATH" == "Y" ]; then
    echo "[NOTICE] Success! You can now type 'tg.sh' in any NEW terminal to launch."
else
    echo "[NOTICE] Suggestion: Add $INSTALL_DIR to your PATH manually."
fi

echo ""
read -n 1 -s -r -p "Press any key to start the game and finish installation..."

rm -- "$0"
./tg.sh