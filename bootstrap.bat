@echo off
REM bootstrap.bat -- first-time initialization of the VisiaEngine dev environment (Windows; idempotent)
REM Usage: bootstrap.bat
REM D5: managed entirely by pixi (conda-forge single source, rust included). The Windows escape hatch (rustup+MSVC) is documented only; it never enters this script.
REM Note: this file was written on a Linux dev machine and has never been tested on Windows -- treat as best-effort until it passes on the first Windows box

setlocal
set "SCRIPT_DIR=%~dp0"
set "PIXI_PIN=0.78.0"

set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"

if defined PIXI (
    echo [1/2] pixi already present: %PIXI%
) else (
    echo [1/2] pixi not found; installing v%PIXI_PIN% ...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "$env:PIXI_VERSION='v%PIXI_PIN%'; iex ((Invoke-WebRequest -UseBasicParsing https://pixi.sh/install.ps1).Content)"
    if errorlevel 1 (
        echo ERROR: pixi install failed >&2
        exit /b 1
    )
    set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
)

echo [2/2] pixi install (first solve includes downloads; may take minutes)...
"%PIXI%" install --manifest-path "%SCRIPT_DIR%pixi.toml"
if errorlevel 1 (
    echo ERROR: pixi install failed >&2
    exit /b 1
)

echo.
echo Environment ready. Daily activation: pixi.bat   (or %PIXI% run ^<task^>)
endlocal
