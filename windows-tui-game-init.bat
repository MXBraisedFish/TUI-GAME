@echo off
setlocal enabledelayedexpansion

echo [INFO] Starting TUI-GAME installation...

where curl >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] curl is required but was not found in PATH.
    pause
    exit /b 1
)

where powershell >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] PowerShell is required but was not found in PATH.
    pause
    exit /b 1
)

REM ----- Step 1: Fetch latest release info -----
echo [INFO] Fetching latest release information from GitHub...
set "API_URL=https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest"
set "TEMP_JSON=%temp%\tui_game_init_%RANDOM%.json"

curl -s -L -o "%TEMP_JSON%" "%API_URL%"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to download release information. Check your internet connection.
    pause
    exit /b 1
)

REM ----- Step 2: Extract Windows zip download URL -----
echo [INFO] Extracting download URL for Windows package...
set "WIN_ZIP_NAME=tui-game-windows.zip"
set "DOWNLOAD_URL="

for /f "usebackq delims=" %%i in (`powershell -command "& { $json = Get-Content '%TEMP_JSON%' | ConvertFrom-Json; $asset = $json.assets | Where-Object { $_.name -eq '%WIN_ZIP_NAME%' }; if ($asset) { $asset.browser_download_url } else { '' } }"`) do (
    set "DOWNLOAD_URL=%%i"
)

if "!DOWNLOAD_URL!"=="" (
    echo [ERROR] Could not find Windows asset '%WIN_ZIP_NAME%' in the latest release.
    del "%TEMP_JSON%"
    pause
    exit /b 1
)
echo [INFO] Download URL: !DOWNLOAD_URL!

REM ----- Step 3: Download the zip package -----
echo [INFO] Downloading game package...
set "TEMP_ZIP=%temp%\tui-game_%RANDOM%.zip"
curl -s -L -o "%TEMP_ZIP%" "!DOWNLOAD_URL!"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to download game package.
    del "%TEMP_JSON%"
    pause
    exit /b 1
)

REM ----- Step 4: Extract zip to current directory (overwrite) -----
echo [INFO] Extracting files to %CD% ...
powershell -command "& { Expand-Archive -Path '%TEMP_ZIP%' -DestinationPath '%CD%' -Force }"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to extract game package.
    del "%TEMP_JSON%"
    del "%TEMP_ZIP%"
    pause
    exit /b 1
)

REM ----- Clean up temporary files -----
del "%TEMP_JSON%"
del "%TEMP_ZIP%"
echo [INFO] Temporary files cleaned up.

REM ----- Step 5: Ask about adding installation folder to PATH -----
echo.
set /p ADD_PATH="Do you want to add the installation folder to your PATH environment variable? (Y/N): "
if /i "!ADD_PATH!"=="Y" (
    echo [INFO] Adding %~dp0 to user PATH...
    set "INSTALL_DIR=%~dp0"
    REM Remove trailing backslash
    if "!INSTALL_DIR:~-1!"=="\" set "INSTALL_DIR=!INSTALL_DIR:~0,-1!"

    REM Check if already in PATH
    echo !PATH! | findstr /i /c:"!INSTALL_DIR!" >nul
    if !errorlevel! equ 0 (
        echo [INFO] Directory already in PATH. Skipping.
    ) else (
        setx PATH "%PATH%;!INSTALL_DIR!"
        if !errorlevel! neq 0 (
            echo [WARNING] Failed to update PATH. You may need to add it manually.
        ) else (
            echo [SUCCESS] PATH updated. You can now use 'tg' command from any terminal (may need to reopen).
        )
    )
    set "REG_OPTION=yes"
) else (
    echo [INFO] Skipping PATH registration.
    set "REG_OPTION=no"
)

REM ----- Step 6: Final messages -----
echo.
echo [SUCCESS] TUI-GAME has been installed successfully!
echo =================================
echo Enjoy the game! :^)
echo.
echo If you like it, please give a star on GitHub: https://github.com/MXBraisedFish/TUI-GAME
echo Author: MXBraisedFish (MXFish)
echo =================================

if /i "!REG_OPTION!"=="yes" (
    echo [INFO] You can start the game by typing 'tg' in any terminal.
) else (
    echo [INFO] To start the game easily from anywhere, add this folder to your PATH, then you can use 'tg' command.
    echo [INFO] Current folder: %~dp0
)

echo.
echo [INFO] Press any key to exit and delete this installer.
pause >nul

REM ----- Delete this script itself -----
del "%~f0" >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Failed to delete installer. Please remove it manually.
    pause
)

exit /b 0
