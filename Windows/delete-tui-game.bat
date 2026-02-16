@echo off
setlocal enabledelayedexpansion

echo [WARNING] This script will permanently delete TUI-GAME runtime files.
echo [WARNING] Save data under "tui-game-data" will also be removed.
echo.
set /p CONFIRM="Continue uninstall? (Y/N): "
if /i not "!CONFIRM!"=="Y" (
    echo [INFO] Uninstall cancelled.
    pause
    exit /b 0
)

set "SCRIPT_DIR=%~dp0"
if "!SCRIPT_DIR:~-1!"=="\" set "SCRIPT_DIR=!SCRIPT_DIR:~0,-1!"

echo [INFO] Working directory: !SCRIPT_DIR!

set "HAS_ERROR=0"
set "EXE=!SCRIPT_DIR!\tui-game.exe"
set "DATA_DIR=!SCRIPT_DIR!\tui-game-data"
set "VERSION_BAT=!SCRIPT_DIR!\version.bat"
set "TG_BAT=!SCRIPT_DIR!\tg.bat"
set "ASSETS_DIR=!SCRIPT_DIR!\assets"
set "SCRIPTS_DIR=!SCRIPT_DIR!\scripts"

call :delete_file "!EXE!"
call :delete_dir "!DATA_DIR!"
call :delete_file "!VERSION_BAT!"
call :delete_file "!TG_BAT!"
call :delete_dir "!ASSETS_DIR!"
call :delete_dir "!SCRIPTS_DIR!"

echo.
set /p CLEAN_PATH="Remove this install directory from User PATH? (Y/N): "
if /i "!CLEAN_PATH!"=="Y" (
    echo [INFO] Cleaning User PATH...
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "$target = [System.IO.Path]::GetFullPath('%SCRIPT_DIR%').TrimEnd('\');" ^
        "$userPath = [Environment]::GetEnvironmentVariable('Path','User');" ^
        "if ([string]::IsNullOrWhiteSpace($userPath)) { exit 0 }" ^
        "$parts = @();" ^
        "foreach ($p in ($userPath -split ';')) {" ^
        "  if ([string]::IsNullOrWhiteSpace($p)) { continue }" ^
        "  try { $full = [System.IO.Path]::GetFullPath($p).TrimEnd('\') } catch { $full = $p.TrimEnd('\') }" ^
        "  if ($full -ne $target) { $parts += $p }" ^
        "}" ^
        "$newPath = ($parts -join ';').Trim(';');" ^
        "[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')"
    if errorlevel 1 (
        echo [WARNING] Failed to update User PATH automatically.
    ) else (
        echo [OK] User PATH updated.
    )
) else (
    echo [INFO] Skipped PATH cleanup.
)

echo.
if "!HAS_ERROR!"=="1" (
    echo [WARNING] Uninstall completed with errors. Some files may remain.
) else (
    echo [SUCCESS] Uninstall completed.
)

echo [INFO] Press any key to exit and remove this script.
pause >nul
del /f /q "%~f0" >nul 2>&1
exit /b 0

:delete_file
set "TARGET=%~1"
if exist "!TARGET!" (
    del /f /q "!TARGET!" >nul 2>&1
    if errorlevel 1 (
        echo [ERROR] Failed to delete file: !TARGET!
        set "HAS_ERROR=1"
    ) else (
        echo [OK] Deleted file: !TARGET!
    )
) else (
    echo [INFO] File not found, skip: !TARGET!
)
exit /b 0

:delete_dir
set "TARGET=%~1"
if exist "!TARGET!" (
    rmdir /s /q "!TARGET!" >nul 2>&1
    if errorlevel 1 (
        echo [ERROR] Failed to delete folder: !TARGET!
        set "HAS_ERROR=1"
    ) else (
        echo [OK] Deleted folder: !TARGET!
    )
) else (
    echo [INFO] Folder not found, skip: !TARGET!
)
exit /b 0
