@echo off
setlocal enabledelayedexpansion

echo [WARNING] This will delete all game data, including saves and records.
echo [WARNING] Deletion is permanent and cannot be restored.
set /p CONTINUE_UNINSTALL="Continue? (Y/N): "
if /i not "!CONTINUE_UNINSTALL!"=="Y" (
    echo [INFO] Uninstall cancelled.
    echo [INFO] Press any key to exit.
    pause >nul
    goto SELF_DELETE
)

set "SCRIPT_DIR=%~dp0"
if "!SCRIPT_DIR:~-1!"=="\" set "SCRIPT_DIR=!SCRIPT_DIR:~0,-1!"
set "ENV_CLEANED=0"

set /p CLEAN_ENV="Use system cleanup for environment variable? (Y/N): "
if /i "!CLEAN_ENV!"=="Y" (
    set "TUI_GAME_INSTALL_DIR=!SCRIPT_DIR!"
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "$ErrorActionPreference = 'Stop';" ^
        "$target = [System.IO.Path]::GetFullPath($env:TUI_GAME_INSTALL_DIR).TrimEnd('\\');" ^
        "$userPath = [Environment]::GetEnvironmentVariable('Path','User');" ^
        "if ([string]::IsNullOrWhiteSpace($userPath)) { exit 10 }" ^
        "$parts = @();" ^
        "$removed = $false;" ^
        "foreach ($p in ($userPath -split ';')) {" ^
        "  if ([string]::IsNullOrWhiteSpace($p)) { continue }" ^
        "  try { $full = [System.IO.Path]::GetFullPath($p).TrimEnd('\\') } catch { $full = $p.TrimEnd('\\') }" ^
        "  if ($full -eq $target) { $removed = $true; continue }" ^
        "  $parts += $p" ^
        "}" ^
        "if (-not $removed) { exit 10 }" ^
        "$newPath = ($parts -join ';').Trim(';');" ^
        "[Environment]::SetEnvironmentVariable('Path', $newPath, 'User');" ^
        "exit 0"

    if !errorlevel! equ 0 (
        set "ENV_CLEANED=1"
        echo [INFO] Environment variable cleanup completed.
    ) else if !errorlevel! equ 10 (
        set "ENV_CLEANED=1"
        echo [INFO] Target path was not found in User PATH.
    ) else (
        echo [WARNING] Failed to clean environment variable automatically.
    )
) else (
    echo [INFO] Environment variable cleanup skipped.
)

call :delete_file "!SCRIPT_DIR!\version.bat"
call :delete_file "!SCRIPT_DIR!\tui-game.exe"
call :delete_dir "!SCRIPT_DIR!\assets"
call :delete_dir "!SCRIPT_DIR!\scripts"
call :delete_dir "!SCRIPT_DIR!\tui-game-data"

if "!ENV_CLEANED!"=="0" (
    echo [WARNING] Current environment variable was not cleaned. Manual cleanup is recommended.
)

echo.
echo Bye bye.
echo.
echo [INFO] Press any key to exit.
pause >nul

goto SELF_DELETE

:delete_file
set "TARGET_FILE=%~1"
if exist "!TARGET_FILE!" (
    del /f /q "!TARGET_FILE!" >nul 2>&1
)
exit /b 0

:delete_dir
set "TARGET_DIR=%~1"
if exist "!TARGET_DIR!" (
    rmdir /s /q "!TARGET_DIR!" >nul 2>&1
)
exit /b 0

:SELF_DELETE
set "SELF_BAT=%~f0"
start "" /b cmd /c "ping 127.0.0.1 -n 2 >nul & del /f /q ""%SELF_BAT%"" >nul 2>&1"
exit /b 0
