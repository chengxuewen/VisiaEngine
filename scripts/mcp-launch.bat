@echo off
REM mcp-launch.bat -- MCP launch wrapper (host-neutral, Windows).
REM
REM Resolves pixi (where pixi / %USERPROFILE%\.pixi\bin\pixi.exe), then runs the
REM pixi task against the repo manifest beside this script's parent dir.
REM
REM NOTE: authored on Linux; best-effort, NOT yet exercised on a real Windows box
REM (mirrors PolyOrch/scripts/pixi.bat + bootstrap.bat convention). Verify on a
REM Windows machine before relying on it (plan #7).
REM
REM Usage: mcp-launch.bat mcp-<name>
setlocal
set "DIR=%~dp0"
set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
if not defined PIXI (
    echo mcp-launch: pixi not found ^(provision via bootstrap^) 1>&2
    exit /b 127
)
"%PIXI%" run --manifest-path "%DIR%..\pixi.toml" %*
