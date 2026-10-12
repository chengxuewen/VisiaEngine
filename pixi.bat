@echo off
REM pixi.bat -- VisiaEngine environment activation (Windows/cmd)
REM cmd has no source-eval model; the pixi subshell is the equivalent interactive form: enter an activated new shell, exit to leave
REM Note: not tested on Windows (best-effort, same clause as bootstrap.bat)

set "SCRIPT_DIR=%~dp0"
set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
if not defined PIXI (
    echo pixi not found -- run first: bootstrap.bat
    exit /b 1
)
"%PIXI%" shell --manifest-path "%SCRIPT_DIR%pixi.toml"
