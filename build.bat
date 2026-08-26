@echo off
REM RustFS Launcher Build Script for Windows
REM Downloads required binary files for Windows platform before building

setlocal enabledelayedexpansion

set BINARIES_DIR=src-tauri\binaries
set TEMP_DIR=temp_downloads

REM Create directories
if not exist "%BINARIES_DIR%" mkdir "%BINARIES_DIR%"
if not exist "%TEMP_DIR%" mkdir "%TEMP_DIR%"

REM Detect architecture. ARM64 Windows falls back to the x86_64 binary
REM because upstream does not yet publish native ARM64 builds.
set ARCH=%PROCESSOR_ARCHITECTURE%
if defined PROCESSOR_ARCHITEW6432 set ARCH=%PROCESSOR_ARCHITEW6432%
if /I "%ARCH%"=="AMD64" set ARCH=x86_64
if /I "%ARCH%"=="x86" set ARCH=x86_64
if /I "%ARCH%"=="ARM64" set ARCH=x86_64

echo Detected platform: Windows %PROCESSOR_ARCHITECTURE% (using %ARCH% binary)
echo Downloading RustFS binary for Windows platform...

echo Resolving latest RustFS version...
for /f "usebackq delims=" %%i in (`powershell -NoProfile -Command "(Invoke-RestMethod -Uri 'https://version.rustfs.com/latest.json').tag"`) do set RUSTFS_RELEASE_TAG=%%i
if "%RUSTFS_RELEASE_TAG%"=="" for /f "usebackq delims=" %%i in (`powershell -NoProfile -Command "(Invoke-RestMethod -Uri 'https://version.rustfs.com/latest.json').version"`) do set RUSTFS_RELEASE_TAG=%%i
if "%RUSTFS_RELEASE_TAG%"=="" (
    echo ✗ Error: Failed to resolve RustFS version from latest.json
    exit /b 1
)

set RUSTFS_ASSET_VERSION=%RUSTFS_RELEASE_TAG%
if not "%RUSTFS_ASSET_VERSION:~0,1%"=="v" set RUSTFS_ASSET_VERSION=v%RUSTFS_ASSET_VERSION%
echo Latest RustFS version: %RUSTFS_RELEASE_TAG%

REM Download Windows binary only
set WINDOWS_X86_64_URL=https://github.com/rustfs/rustfs/releases/download/%RUSTFS_RELEASE_TAG%/rustfs-windows-x86_64-%RUSTFS_ASSET_VERSION%.zip

if "%ARCH%"=="x86_64" (
    echo Downloading for Windows x86_64...
    call :download_binary "%WINDOWS_X86_64_URL%" "rustfs-windows-x86_64" "rustfs-windows-x86_64.exe"
) else (
    echo ✗ Error: Unsupported Windows architecture: %ARCH%
    echo Only x86_64 is supported
    exit /b 1
)

REM Clean up temporary files
echo Cleaning up temporary files...
if exist "%TEMP_DIR%" rmdir /s /q "%TEMP_DIR%"

echo Binary downloaded successfully for Windows %ARCH%!
echo You can now run: cargo tauri build
goto :eof

:download_binary
set url=%~1
set filename=%~2
set target_name=%~3

echo Downloading %filename%...

REM Download using curl (available in Windows 10+)
curl -fL --retry 3 --retry-delay 5 -H "Cache-Control: no-cache" -o "%TEMP_DIR%\%filename%.zip" "%url%"
if errorlevel 1 (
    echo ✗ Error: Failed to download %filename%
    exit /b 1
)

echo Extracting %filename%...
REM Extract using PowerShell
powershell -command "Expand-Archive -Path '%TEMP_DIR%\%filename%.zip' -DestinationPath '%TEMP_DIR%\%filename%' -Force"

REM Find and copy the binary
if exist "%TEMP_DIR%\%filename%\rustfs.exe" (
    copy "%TEMP_DIR%\%filename%\rustfs.exe" "%BINARIES_DIR%\%target_name%"
) else if exist "%TEMP_DIR%\%filename%\rustfs" (
    copy "%TEMP_DIR%\%filename%\rustfs" "%BINARIES_DIR%\%target_name%"
) else (
    echo ✗ Error: Binary not found in extracted files
    dir "%TEMP_DIR%\%filename%\"
    exit /b 1
)

echo ✓ %target_name% downloaded and extracted successfully
goto :eof
