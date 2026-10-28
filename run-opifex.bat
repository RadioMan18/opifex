@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

echo ========================================
echo  Opifex - Agent Builder (Smoke Test)
echo ========================================
echo.

:: Check for Node.js
where node.exe >nul 2>&1
if errorlevel 1 (
    echo ERROR: Node.js not found. Please install Node.js first.
    pause
    exit /b 1
)

:: Check for Rust/Cargo
where cargo.exe >nul 2>&1
if errorlevel 1 (
    echo ERROR: Rust/Cargo not found. Please install Rust first.
    pause
    exit /b 1
)

echo [1/3] Cleaning previous build...
cargo clean >nul 2>&1
npm run build
if errorlevel 1 (
    echo ERROR: Frontend build failed.
    pause
    exit /b 1
)

echo.
echo [2/3] Building Tauri application...
cargo tauri build --manifest-path src-tauri/Cargo.toml
if errorlevel 1 (
    echo ERROR: Tauri build failed.
    pause
    exit /b 1
)

echo.
echo [3/3] Launching Opifex...
set "APP_DIR=%~dp0target\release"
set "APP_EXE=%APP_DIR%\app.exe"
if not exist "%APP_EXE%" (
    echo ERROR: Built application not found at:
    echo %APP_EXE%
    pause
    exit /b 1
)

start "" /D "%APP_DIR%" "%APP_EXE%"
if errorlevel 1 (
    echo ERROR: Failed to launch Opifex.
    pause
    exit /b 1
)

echo.
echo Opifex was launched. Check your taskbar for the window.
echo.
pause