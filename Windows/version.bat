@echo off
setlocal enabledelayedexpansion

echo [INFO] Starting update process...

REM ----- Step 1: Fetch latest release info -----
echo [INFO] Fetching latest release information from GitHub...
set "API_URL=https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest"
set "TEMP_JSON=%temp%\latest_release.json"

curl -s -L -o "%TEMP_JSON%" "%API_URL%"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to download release information. Check your internet connection.
    pause
    exit /b 1
)

REM ----- Step 2: Extract Windows asset download URL -----
echo [INFO] Extracting download URL for Windows package...
set "WIN_ZIP_NAME=tui-game-windows.zip"
set "DOWNLOAD_URL="

REM Use PowerShell to parse JSON and extract the browser_download_url for the windows zip
for /f "usebackq delims=" %%i in (`powershell -command "& { $json = Get-Content '%TEMP_JSON%' | ConvertFrom-Json; $asset = $json.assets | Where-Object { $_.name -eq '%WIN_ZIP_NAME%' }; if ($asset) { $asset.browser_download_url } else { '' } }"`) do (
    set "DOWNLOAD_URL=%%i"
)

if "!DOWNLOAD_URL!"=="" (
    echo [ERROR] Could not find Windows asset '%WIN_ZIP_NAME%' in the latest release.
    del "%TEMP_JSON%" 2>nul
    pause
    exit /b 1
)

echo [INFO] Download URL: !DOWNLOAD_URL!

REM ----- Step 3: Download the zip package -----
echo [INFO] Downloading update package...
set "TEMP_ZIP=%temp%\tui-game-update.zip"
curl -s -L -o "%TEMP_ZIP%" "!DOWNLOAD_URL!"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to download update package.
    del "%TEMP_JSON%" 2>nul
    pause
    exit /b 1
)

REM ----- Step 4: Extract the zip to current directory, overwriting files -----
echo [INFO] Extracting update to current directory (overwriting files)...
REM Use PowerShell Expand-Archive (available on most Windows 10/11)
powershell -command "& { Expand-Archive -Path '%TEMP_ZIP%' -DestinationPath '%~dp0' -Force }"
if %errorlevel% neq 0 (
    echo [ERROR] Failed to extract the update package.
    del "%TEMP_JSON%" 2>nul
    del "%TEMP_ZIP%" 2>nul
    pause
    exit /b 1
)

REM ----- Step 5: Clean up temporary files -----
del "%TEMP_JSON%" 2>nul
del "%TEMP_ZIP%" 2>nul
echo [INFO] Temporary files cleaned up.

echo [SUCCESS] Update completed successfully!
echo [INFO] Press any key to exit.
pause >nul
exit /b 0