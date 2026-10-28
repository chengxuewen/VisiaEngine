@echo off
rem visiaengine.bat -- execution front door for Windows.
rem
rem BEST-EFFORT: authored on Linux, NOT yet exercised on a real Windows machine
rem (same status as pixi.bat / bootstrap.bat). If it misbehaves, `pixi run <task>`
rem remains the supported path.
rem
rem Pure passthrough:  visiaengine.bat <task> [args...]   ==   pixi run <task> [args...]
rem No task table here on purpose -- pixi.toml is the single source of task names.
rem Not source-able; use pixi.bat when you want an activated sub-shell.
rem
rem Exit codes owned by this file: -h/--help -> 0, no args -> 2, pixi missing -> 127.

setlocal
cd /d "%~dp0"

if "%~1"=="" goto usage2
if /i "%~1"=="-h" goto usage0
if /i "%~1"=="--help" goto usage0

rem Resolve pixi the same way scripts\mcp-launch.bat does: PATH first, then %USERPROFILE%.
set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
if not defined PIXI (
    echo visiaengine.bat: pixi not found ^(run bootstrap.bat^) 1>&2
    exit /b 127
)

"%PIXI%" run %*
exit /b %ERRORLEVEL%

:usage0
echo usage: visiaengine.bat ^<task^> [args...]
echo.
echo common:
echo   visiaengine.bat mcp status      MCP: pin/installed versions + index state
echo   visiaengine.bat mcp index       build ^(first run^) or rebuild .codegraph
echo   visiaengine.bat gate-agents     single-source consistency gate
echo   visiaengine.bat ci              the whole CI chain
echo   pixi task list                  every task this workspace defines
echo.
echo pixi run ^<task^> is always equivalent.
exit /b 0

:usage2
echo usage: visiaengine.bat ^<task^> [args...]   ^(see: visiaengine.bat -h^) 1>&2
exit /b 2
