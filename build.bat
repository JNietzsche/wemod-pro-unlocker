@echo off
setlocal enabledelayedexpansion

echo ========================================
echo  Building WeMod Pro Unlocker Artifacts
echo ========================================

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build.ps1"
if %ERRORLEVEL% neq 0 (
    echo.
    echo Build failed with error code %ERRORLEVEL%
    exit /b %ERRORLEVEL%
)

endlocal
