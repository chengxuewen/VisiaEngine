@echo off
REM bootstrap.bat — VisiaEngine 开发环境首次初始化（Windows；幂等）
REM 用法: bootstrap.bat
REM D5: 全 pixi 管理（conda-forge 单源，rust 含在内）。Windows 逃生舱（rustup+MSVC）仅文档记载，不入本脚本。
REM 注: 本文件在 Linux 开发机上编写，未经 Windows 实测——首台 Windows 机器上跑通前视为 best-effort

setlocal
set "SCRIPT_DIR=%~dp0"
set "PIXI_PIN=0.78.0"

set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"

if defined PIXI (
    echo [1/2] pixi 已就绪: %PIXI%
) else (
    echo [1/2] 未找到 pixi，安装 v%PIXI_PIN% ...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "$env:PIXI_VERSION='v%PIXI_PIN%'; iex ((Invoke-WebRequest -UseBasicParsing https://pixi.sh/install.ps1).Content)"
    if errorlevel 1 (
        echo ERROR: pixi 安装失败 >&2
        exit /b 1
    )
    set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
)

echo [2/2] pixi install（首次求解含下载，可能数分钟）...
"%PIXI%" install --manifest-path "%SCRIPT_DIR%pixi.toml"
if errorlevel 1 (
    echo ERROR: pixi install 失败 >&2
    exit /b 1
)

echo.
echo 环境就绪。日常激活: pixi.bat   （或 %PIXI% run ^<task^>）
endlocal
