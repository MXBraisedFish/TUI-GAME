@echo off
setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "BIN=%SCRIPT_DIR%tui-game.exe"

if not exist "!BIN!" (
    echo [ERROR] tui-game.exe not found: !BIN!
    exit /b 1
)

"!BIN!" %*
exit /b %errorlevel%
