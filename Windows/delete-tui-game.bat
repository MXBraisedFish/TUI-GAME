@echo off
setlocal enabledelayedexpansion

echo [WARNING] This script will permanently delete game files and may remove saved data.

REM ----- Step 1: Confirmation -----
set /p CONFIRM="Are you sure you want to delete all game files? (Y/N): "
if /i not "!CONFIRM!"=="Y" (
    echo [INFO] Deletion cancelled by user.
    pause
    exit /b 0
)

REM ----- Step 2: Delete specific files and folders -----
echo [INFO] Deleting game files from current directory...

set "CUR_DIR=%~dp0"

REM Delete tui-game.exe
if exist "!CUR_DIR!tui-game.exe" (
    del /f /q "!CUR_DIR!tui-game.exe" 2>nul
    if !errorlevel! equ 0 ( echo [OK] Deleted tui-game.exe ) else ( echo [ERROR] Failed to delete tui-game.exe )
) else ( echo [INFO] tui-game.exe not found, skipping. )

REM Delete assets folder
if exist "!CUR_DIR!assets\" (
    rmdir /s /q "!CUR_DIR!assets" 2>nul
    if !errorlevel! equ 0 ( echo [OK] Deleted assets folder ) else ( echo [ERROR] Failed to delete assets folder )
) else ( echo [INFO] assets folder not found, skipping. )

REM Delete scripts folder
if exist "!CUR_DIR!scripts\" (
    rmdir /s /q "!CUR_DIR!scripts" 2>nul
    if !errorlevel! equ 0 ( echo [OK] Deleted scripts folder ) else ( echo [ERROR] Failed to delete scripts folder )
) else ( echo [INFO] scripts folder not found, skipping. )

REM Delete version.bat
if exist "!CUR_DIR!version.bat" (
    del /f /q "!CUR_DIR!version.bat" 2>nul
    if !errorlevel! equ 0 ( echo [OK] Deleted version.bat ) else ( echo [ERROR] Failed to delete version.bat )
) else ( echo [INFO] version.bat not found, skipping. )

REM Delete tg.bat (current script will be deleted later, so we skip deleting itself now)
if exist "!CUR_DIR!tg.bat" (
    del /f /q "!CUR_DIR!tg.bat" 2>nul
    if !errorlevel! equ 0 ( echo [OK] Deleted tg.bat ) else ( echo [ERROR] Failed to delete tg.bat )
) else ( echo [INFO] tg.bat not found, skipping. )

REM ----- Step 3: Registry cleanup (optional) -----
echo.
set /p REG_CONFIRM="Do you want to delete registry entries for this folder? (Y/N): "
if /i "!REG_CONFIRM!"=="Y" (
    echo [INFO] Removing registry entries...
    REM Here we assume registry files are .reg files in current directory.
    for %%f in ("!CUR_DIR!*.reg") do (
        del /f /q "%%f" 2>nul
        if !errorlevel! equ 0 ( echo [OK] Deleted %%~nxf ) else ( echo [ERROR] Failed to delete %%~nxf )
    )
    echo [INFO] Registry cleanup finished.
    set "REG_REMOVED=1"
) else (
    set "REG_REMOVED=0"
)

REM ----- Step 4: Final messages -----
echo.
echo [INFO] Goodbye!
if !REG_REMOVED! equ 0 (
    echo [REMINDER] Please manually clean up any registry entries for this folder if needed.
)

REM ----- Step 5: Wait for key press and self-delete -----
echo [INFO] Press any key to close this window and delete this script.
pause >nul

REM Delete this batch file itself (delete-tui-game.bat)
del /f /q "%~f0" 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Failed to delete this script. You may need to remove it manually.
    pause
)

exit /b 0