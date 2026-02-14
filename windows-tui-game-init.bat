@echo off
setlocal enabledelayedexpansion

set "REPO_API=https://api.github.com/repos/MXBraisedFish/TUI-GAME/releases/latest"
set "ZIP_NAME=tui-game-windows.zip"
set "EXE_NAME=tui-game.exe"
set "INSTALL_DIR=%~dp0"
set "REPO_URL=https://github.com/MXBraisedFish/TUI-GAME"

echo [1/8] Fetching latest release info from GitHub...
powershell -NoProfile -Command "$ErrorActionPreference='Stop'; try { $resp = Invoke-RestMethod -Uri '%REPO_API%' -Headers @{'User-Agent'='TUI-Game-Init'}; $asset = $resp.assets | Where-Object { $_.name -eq '%ZIP_NAME%' }; if (-not $asset) { throw 'Asset not found' }; $asset.browser_download_url | Out-File -FilePath 'download_url.txt' -Encoding ascii } catch { exit 1 }"
if %ERRORLEVEL% neq 0 ( echo [ERROR] Failed to fetch release info. & pause & exit /b 1 )

set /p DOWNLOAD_URL=<download_url.txt
del download_url.txt

echo [2/8] Downloading %ZIP_NAME%...
powershell -NoProfile -Command "try { Invoke-WebRequest -Uri '%DOWNLOAD_URL%' -OutFile '%ZIP_NAME%' } catch { exit 1 }"
if %ERRORLEVEL% neq 0 ( echo [ERROR] Download failed. & pause & exit /b 1 )

echo [3/8] Unzipping files into: %INSTALL_DIR% [cite: 17]
powershell -NoProfile -Command "Expand-Archive -Path '%ZIP_NAME%' -DestinationPath '.' -Force"
del "%ZIP_NAME%"

echo [4/8] Creating startup script (tg.bat)...
(
    echo @echo off
    echo cd /d "%%~dp0"
    echo "%EXE_NAME%"
) > "tg.bat"

echo [5/8] Environment Variable (PATH) Settings...
set /p choice="Do you want to add this directory to your User PATH automatically? (Y/N): "
if /i "%choice%"=="Y" (
    powershell -NoProfile -Command "$path=[Environment]::GetEnvironmentVariable('PATH','User'); $dir='%INSTALL_DIR%'.TrimEnd('\'); if($path -split ';' -notcontains $dir){ [Environment]::SetEnvironmentVariable('PATH',$path+';'+$dir,'User'); }"
    echo [SUCCESS] Path registered.
) else (
    echo [INFO] Skipped PATH registration.
)

echo [6/8] Generating version.bat for updates...
(
    echo @echo off
    echo echo [UPDATE] Updating TUI-GAME...
    echo powershell -NoProfile -Command "$resp = Invoke-RestMethod -Uri '%REPO_API%' -Headers @{'User-Agent'='TUI-Updater'}; $url = ($resp.assets ^| Where-Object { $_.name -eq '%ZIP_NAME%' }).browser_download_url; Invoke-WebRequest -Uri $url -OutFile '%ZIP_NAME%'; if (-not (Test-Path 'tui-game-data')) { New-Item -ItemType Directory -Path 'tui-game-data' }; '{\"version\": \"' + $resp.tag_name + '\"}' ^| Out-File -FilePath './tui-game-data/updater_cache.json' -Encoding utf8"
    echo powershell -NoProfile -Command "Expand-Archive -Path '%ZIP_NAME%' -DestinationPath '.' -Force"
    echo del "%ZIP_NAME%"
    echo set /p choice="Update done. Press [T] to restart, or any key to exit: "
    echo if /i "%%choice%%"=="T" tg.bat
) > "version.bat"

echo [7/8] Generating delete-tui-game.bat (Uninstaller)... [cite: 24, 25]
echo @echo off > "delete-tui-game.bat"
echo set /p confirm="Unnesting TUI GAME will delete all game files and save data. Continue? (Y/N): " >> "delete-tui-game.bat"
echo if /i "%%confirm%%" neq "Y" exit /b >> "delete-tui-game.bat"
echo echo [CLEANING] Removing game files... >> "delete-tui-game.bat"
echo if exist "assets" rd /s /q "assets" >> "delete-tui-game.bat"
echo if exist "scripts" rd /s /q "scripts" >> "delete-tui-game.bat"
echo if exist "tui-game-data" rd /s /q "tui-game-data" >> "delete-tui-game.bat"
echo del "tg.bat" "tui-game.exe" "version.bat" 2^>nul >> "delete-tui-game.bat"
echo set /p reg_confirm="Do you want to remove the directory from system PATH environment? (Y/N): " >> "delete-tui-game.bat"
echo if /i "%%reg_confirm%%"=="Y" ( >> "delete-tui-game.bat"
echo     powershell -NoProfile -Command "$path=[Environment]::GetEnvironmentVariable('PATH','User'); $dir='%INSTALL_DIR%'.TrimEnd('\'); $newPath=($path -split ';' ^| Where-Object { $_ -ne $dir }) -join ';'; [Environment]::SetEnvironmentVariable('PATH',$newPath,'User');" >> "delete-tui-game.bat"
echo     echo [SUCCESS] PATH cleaned. >> "delete-tui-game.bat"
echo ) else ( >> "delete-tui-game.bat"
echo     echo [INFO] Please clean the registry/PATH manually if needed. >> "delete-tui-game.bat"
echo ) >> "delete-tui-game.bat"
echo echo Bye bye. >> "delete-tui-game.bat"
echo pause ^>nul >> "delete-tui-game.bat"
echo start /b "" cmd /c del "%%~f0" ^& exit >> "delete-tui-game.bat"

echo [8/8] Finalizing...
echo ------------------------------------------------------
echo Have fun :^)
echo If you like this project, please give a star to my repository! [cite: 26]
echo Better ideas are always welcome. [cite: 26]
echo.
echo Github: %REPO_URL% [cite: 27]
echo ------------------------------------------------------

if /i "%choice%"=="Y" echo [NOTICE] You can now type 'tg' in any NEW terminal to launch the game. [cite: 28]
if /i "%choice%"=="N" echo [NOTICE] Suggestion: Add %INSTALL_DIR% to your PATH manually for 'tg' command. [cite: 28]

echo.
echo Press any key to start the game and finish installation... 
pause >nul

start /b "" cmd /c "timeout /t 1 >nul & del /f /q "%~f0""
tg.bat