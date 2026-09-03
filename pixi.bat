@echo off
REM pixi.bat — VisiaEngine 环境激活（Windows/cmd）
REM cmd 无 source-eval 模型，pixi 子 shell 是对等交互形态: 进入一个已激活的新 shell，exit 退出
REM 注: 未经 Windows 实测（best-effort，同 bootstrap.bat 条款）

set "SCRIPT_DIR=%~dp0"
set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
if not defined PIXI (
    echo 未找到 pixi — 先执行: bootstrap.bat
    exit /b 1
)
"%PIXI%" shell --manifest-path "%SCRIPT_DIR%pixi.toml"
