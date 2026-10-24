@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

echo ========================================
echo  Opifex v1.1.0 - Agent Teams (Test Runner)
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

echo Choose run mode:
echo   [1] Dev mode (fast, hot-reload, for testing)
echo   [2] Release build (full build, for distribution testing)
echo   [3] Dev mode + start Notifier companion
echo.
set /p MODE="Enter choice (1-3): "

if "%MODE%"=="1" goto dev
if "%MODE%"=="2" goto release
if "%MODE%"=="3" goto dev_notifier

:dev
echo.
echo [1/2] Building frontend...
call npm run build
if errorlevel 1 (
    echo ERROR: Frontend build failed.
    pause
    exit /b 1
)

echo.
echo [2/2] Launching Opifex dev mode...
echo Starting Vite dev server in background...
start "Vite Dev" cmd /k "npm run dev"
timeout /t 5 >nul
echo Launching Tauri application...
cd /d "%~dp0src-tauri"
cargo run
if errorlevel 1 (
    echo ERROR: Failed to launch Opifex dev.
    pause
    exit /b 1
)
goto end

:release
echo.
echo [1/3] Building frontend...
call npm run build
if errorlevel 1 (
    echo ERROR: Frontend build failed.
    pause
    exit /b 1
)

echo.
echo [2/3] Building Tauri application...
cd /d "%~dp0src-tauri"
cargo tauri build
if errorlevel 1 (
    echo ERROR: Tauri build failed.
    pause
    exit /b 1
)

echo.
echo [3/3] Launching Opifex...
set "APP_DIR=%~dp0src-tauri\target\release"
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
goto end

:dev_notifier
echo.
echo [1/2] Building frontend...
call npm run build
if errorlevel 1 (
    echo ERROR: Frontend build failed.
    pause
    exit /b 1
)

echo.
echo [2/2] Launching Opifex dev + Notifier companion...
start "Notifier" cmd /k "cd /d D:\src\notifier && npm start"
timeout /t 3 >nul
echo Starting Vite dev server in background...
start "Vite Dev" cmd /k "npm run dev"
timeout /t 5 >nul
echo Launching Tauri application...
cd /d "%~dp0src-tauri"
cargo run
if errorlevel 1 (
    echo ERROR: Failed to launch Opifex dev.
    pause
    exit /b 1
)

:end
echo.
echo Done.
pause
