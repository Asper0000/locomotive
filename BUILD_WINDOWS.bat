@echo off
setlocal

rem CMD does not execute .ps1 files directly. Always invoke PowerShell explicitly.
set "SCRIPT_DIR=%~dp0"
where powershell.exe >nul 2>nul
if errorlevel 1 (
    echo ERROR: Windows PowerShell was not found.
    exit /b 1
)

powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%BUILD_WINDOWS.ps1" %*
set "BUILD_EXIT_CODE=%ERRORLEVEL%"

if not "%BUILD_EXIT_CODE%"=="0" (
    echo.
    echo Build failed with exit code %BUILD_EXIT_CODE%.
    echo Check the error above and see BUILD_INSTRUCTIONS.md.
)

exit /b %BUILD_EXIT_CODE%
