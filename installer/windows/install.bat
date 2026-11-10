@echo off
setlocal
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "install.ps1"
if %ERRORLEVEL% equ 0 (
    echo.
    echo Installation finished! You can find DownSort on your Desktop and Start Menu.
    pause
) else (
    echo.
    echo Installation failed.
    pause
)
