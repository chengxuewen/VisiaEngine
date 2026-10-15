@echo off
REM bootstrap.bat -- first-time initialization of the VisiaEngine dev environment (Windows; idempotent)
REM Usage: bootstrap.bat
REM D5: managed entirely by pixi (conda-forge single source, rust included). The Windows escape hatch (rustup+MSVC) is documented only; it never enters this script.
REM T-B2 pattern (mirrors bootstrap.sh): the pixi install stages delegate to the PolyOrch
REM standard pixi-init (submodule scripts\pixi-init.bat, manifest-parameterized); this
REM script keeps only what is VisiaEngine-specific: the submodule prerequisite and the verify smoke.
REM Note: this file was written on a Linux dev machine and has never been tested on Windows -- treat as best-effort until it passes on the first Windows box

setlocal
set "SCRIPT_DIR=%~dp0"
set "PIXI_INIT=%SCRIPT_DIR%PolyOrch\scripts\pixi-init.bat"
set "MANIFEST=%SCRIPT_DIR%pixi.toml"

REM --- [0/3] submodule prerequisite (cold clones); idempotent ---
if not exist "%PIXI_INIT%" if not exist "%SCRIPT_DIR%PolyOrch\pixi.toml" git -C "%SCRIPT_DIR%." submodule update --init PolyOrch
if not exist "%PIXI_INIT%" (
    echo ERROR: missing %PIXI_INIT% after submodule init >&2
    exit /b 1
)

REM --- [1/3] pixi + environment via the PolyOrch standard bootstrap (incl. its smoke) ---
call "%PIXI_INIT%" "%MANIFEST%"
if errorlevel 1 exit /b 1

REM activation trio comes from the supplier; root copies are generated content
copy /Y "%SCRIPT_DIR%PolyOrch\scripts\pixi.sh"   "%SCRIPT_DIR%." >nul
copy /Y "%SCRIPT_DIR%PolyOrch\scripts\pixi.bat"  "%SCRIPT_DIR%." >nul
copy /Y "%SCRIPT_DIR%PolyOrch\scripts\pixi.ps1"  "%SCRIPT_DIR%." >nul

REM --- [3/3] host smoke (parity with bootstrap.sh; [2/3] cargo-mirror is Unix-side only) ---
echo [3/3] Smoke: pixi run verify ...
set "PIXI=pixi"
where pixi >nul 2>nul || set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
"%PIXI%" run --manifest-path "%MANIFEST%" verify
if errorlevel 1 ( echo ERROR: verify failed >&2 & exit /b 1 )

echo.
echo =============== VisiaEngine environment ready ===============
echo   Daily activation:  pixi.bat  (cmd; child shell)  or  . \pixi.ps1  (PowerShell)
echo   One-shot commands: pixi run ^<task^>     (all tasks: pixi task list)
echo   Handy tasks:       pixi run verify (toolchain smoke) / pixi run lint (clippy -D warnings)
echo   Guide:             docs\env.md
echo ============================================================
endlocal
