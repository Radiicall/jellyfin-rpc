@echo off

REM Set paths
set EXE_PATH=%CD%\jellyfin-rpc\jellyfin-rpc.exe
set JSON_PATH=%CD%\jellyfin-rpc\main.json
set DOWNLOAD_URL=https://github.com/Radiicall/jellyfin-rpc/releases/latest/download/jellyfin-rpc.exe
set DOWNLOAD_DIR=jellyfin-rpc


echo ===============================================================================
echo                        JELLYFIN-RPC INSTALLATION 
echo ===============================================================================
echo.

REM Check if jellyfin-rpc folder exist
if not exist "%DOWNLOAD_DIR%" mkdir "%DOWNLOAD_DIR%"

REM Check if jellyfin-rpc.exe is present
if exist "%EXE_PATH%" (
    echo jellyfin-rpc.exe is already present. & timeout /t 3 /nobreak >nul
) else (
    REM Downloading jellyfin-rpc binary
    echo Downloading jellyfin-rpc binary from GitHub... & timeout /t 3 /nobreak >nul
    curl -L %DOWNLOAD_URL% -o "%DOWNLOAD_DIR%\jellyfin-rpc.exe"
)

REM Check if main.json is present
if exist "%JSON_PATH%" (
    echo main.json file is already present & timeout /t 3 /nobreak >nul
) else (
    echo   
    set /p SAY=Make a main.json file in %DOWNLOAD_DIR% folder and Hit ENTER to continue...
)

REM Check if NSSM is already installed
if exist "%DOWNLOAD_DIR%\nssm-2.24\win64\nssm.exe" (
    echo NSSM is already installed. & timeout /t 3 /nobreak >nul
) else (
    REM Download NSSM installer
    echo Downloading and unzipping NSSM installer... & timeout /t 3 /nobreak >nul
    curl -L https://nssm.cc/release/nssm-2.24.zip -o nssm.zip

    REM Unzip NSSM
    powershell -Command "Expand-Archive -LiteralPath nssm.zip -DestinationPath ."
    move /Y "nssm-2.24" "%DOWNLOAD_DIR%\"
    echo Deleting unnecessary nssm.zip file
    del "nssm.zip"
)

REM Install NSSM
echo Installing jellyfin-rpc service... & timeout /t 3 /nobreak >nul
%DOWNLOAD_DIR%\nssm-2.24\win64\nssm.exe install jellyfin-rpc "%EXE_PATH%"

REM Start the executable using NSSM
echo Starting jellyfin-rpc service... & timeout /t 3 /nobreak >nul
set "psCommand=powershell -Command "Start-Process %DOWNLOAD_DIR%\nssm-2.24\win64\nssm.exe -Verb RunAs -ArgumentList 'start','jellyfin-rpc'""
powershell -NoProfile -ExecutionPolicy Bypass -Command "%psCommand%"

REM Pause for 5 seconds
ping -n 5 127.0.0.1 > nul

REM Coded by xenoncolt.tk

REM Check if the service is running
tasklist /fi "imagename eq jellyfin-rpc.exe" | find ":" > nul
if %errorlevel%==0 (
    echo ===============================================================================
    echo                   JELLYFIN-RPC SERVICE IS RUNNING
    echo ===============================================================================
) else (
    echo jellyfin-rpc service failed to start.
)

echo.
echo ===============================================================================
echo                              INSTALLATION COMPLETE!
echo ===============================================================================
