#!/bin/bash
set -u

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
BIN="$SCRIPT_DIR/tui-game"

if [ ! -x "$BIN" ]; then
    echo "[ERROR] Executable not found or not executable: $BIN"
    exit 1
fi

"$BIN" "$@"
exit $?
