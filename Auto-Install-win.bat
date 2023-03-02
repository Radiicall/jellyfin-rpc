@echo off

REM Set paths
set EXE_PATH=%CD%\jellyflix-rpc\jellyflix-rpc.exe
set ENV_PATH=%CD%\jellyflix-rpc\.env

echo ===============================================================================
echo                        JELLYFLIX-RPC INSTALLATION 
echo ===============================================================================
echo.

REM Check if jellyflix-rpc.exe is present
if exist "%EXE_PATH%" (
    echo jellyflix-rpc.exe is already present. & timeout /t 3 /nobreak >nul
) else (
    REM Download and unzip jellyflix-rpc.zip from GitHub
    echo Downloading and unzipping jellyflix-rpc.zip from GitHub... & timeout /t 3 /nobreak >nul
    curl -L https://github.com/xenoncolt/jellyflix-rpc/releases/download/v0.2.0/jellyflix-rpc.zip -o jellyflix-rpc.zip
    powershell -Command "Expand-Archive -LiteralPath jellyflix-rpc.zip -DestinationPath ."
)

REM Check if .env file is present
if exist "%ENV_PATH%" (
    echo .env file is already present. & timeout /t 3 /nobreak >nul
) else (
    REM Copy example.env to .env
    copy jellyflix-rpc\example.env jellyflix-rpc\.env
    echo Created .env file from example.env. & timeout /t 3 /nobreak >nul
)

REM Check if NSSM is already installed
if exist "nssm-2.24\win64\nssm.exe" (
    echo NSSM is already installed. & timeout /t 3 /nobreak >nul
) else (
    REM Download NSSM installer
    echo Downloading and unzipping NSSM installer... & timeout /t 3 /nobreak >nul
    curl -L https://nssm.cc/release/nssm-2.24.zip -o nssm.zip

    REM Unzip NSSM
    powershell -Command "Expand-Archive -LiteralPath nssm.zip -DestinationPath ."
)

REM Install NSSM
echo Installing Jellyflix-rpc service... & timeout /t 3 /nobreak >nul
nssm-2.24\win64\nssm.exe install jellyflix-rpc "%EXE_PATH%"

REM Start the executable using NSSM
echo Starting Jellyflix-rpc service... & timeout /t 3 /nobreak >nul
set "psCommand=powershell -Command "Start-Process nssm-2.24\win64\nssm.exe -Verb RunAs -ArgumentList 'start','jellyflix-rpc'""
powershell -NoProfile -ExecutionPolicy Bypass -Command "%psCommand%"

REM Pause for 5 seconds
ping -n 5 127.0.0.1 > nul

REM Check if the service is running
tasklist /fi "imagename eq jellyflix-test.exe" | find ":" > nul
if %errorlevel%==0 (
    echo ===============================================================================
    echo                   JELLYFLIX-RPC SERVICE IS RUNNING
    echo ===============================================================================
) else (
    echo Jellyflix-rpc service failed to start.
)

echo.
echo ===============================================================================
echo                              INSTALLATION COMPLETE!
echo ===============================================================================
