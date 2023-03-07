@echo off

REM Set paths
set EXE_PATH=%APPDATA%\jellyfin-rpc\jellyfin-rpc.exe
set JSON_PATH=%APPDATA%\jellyfin-rpc\main.json
set DOWNLOAD_URL=https://github.com/Radiicall/jellyfin-rpc/releases/latest/download/jellyfin-rpc.exe
set DOWNLOAD_DIR=%APPDATA%\jellyfin-rpc

REM set
set JELLYFIN_URL=https://example.com
set JELLYFIN_API_KEY=abcdef0123456789
set JELLYFIN_USERNAME=admin
set DISCORD_APPLICATION_ID=1053747938519679018
set DISCORD_ENABLE_IMAGES=false

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

rem Prompt the user for input
set /p JELLYFIN_URL=Enter Jellyfin URL "[%JELLYFIN_URL%]": 
set /p JELLYFIN_API_KEY=Enter Jellyfin API key "[%JELLYFIN_API_KEY%]": 
set /p JELLYFIN_USERNAME=Enter Jellyfin username "[%JELLYFIN_USERNAME%]": 
set /p DISCORD_APPLICATION_ID=Enter Discord application ID "[%DISCORD_APPLICATION_ID%]": 
set /p IMGUR_CLIENT_ID=Enter Imgur client ID (Leave empty if not using) "[%IMGUR_CLIENT_ID%]": 
set /p IMAGES_ENABLE_IMAGES=Enable images (true/false) [%IMAGES_ENABLE_IMAGES%]:
set /p IMAGES_IMGUR_IMAGES=Enable images from Imgur (true/false) [%IMAGES_IMGUR_IMAGES%]:



rem Output the JSON data to the file
echo { > main.json
echo     "Jellyfin": { >> main.json
echo         "URL": "%JELLYFIN_URL%", >> main.json
echo         "API_KEY": "%JELLYFIN_API_KEY%", >> main.json
echo         "USERNAME": "%JELLYFIN_USERNAME%" >> main.json
echo     }, >> main.json
echo     "Discord": { >> main.json
echo         "APPLICATION_ID": "%DISCORD_APPLICATION_ID%" >> main.json
echo     }, >> main.json
echo     "Imgur": { >> main.json
echo         "CLIENT_ID": "%IMGUR_CLIENT_ID%" >> main.json
echo     }, >> main.json
echo     "Images": { >> main.json
echo         "ENABLE_IMAGES": %IMAGES_ENABLE_IMAGES%, >> main.json
echo         "IMGUR_IMAGES": %IMAGES_IMGUR_IMAGES% >> main.json
echo     } >> main.json
echo } >> main.json

REM Check if main.json is present
if exist "%JSON_PATH%" (
    echo main.json file is already present & timeout /t 3 /nobreak >nul
    del "main.json"
) else (
    move "main.json" "%DOWNLOAD_DIR%\" >nul
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
    move /Y "nssm-2.24" "%DOWNLOAD_DIR%\" >nul
    echo Deleting unnecessary nssm.zip file
    del "nssm.zip"
)

REM Install NSSM
echo Installing jellyfin-rpc service... & timeout /t 3 /nobreak >nul
%DOWNLOAD_DIR%\nssm-2.24\win64\nssm.exe install jellyfin-rpc "%EXE_PATH%" "-c %JSON_PATH% -i %DOWNLOAD_DIR%\urls.json"

REM Start the executable using NSSM
echo Starting jellyfin-rpc service... & timeout /t 3 /nobreak >nul
set "psCommand=powershell -Command "Start-Process %DOWNLOAD_DIR%\nssm-2.24\win64\nssm.exe -Verb RunAs -ArgumentList 'start','jellyfin-rpc'""
powershell -NoProfile -ExecutionPolicy Bypass -Command "%psCommand%"


REM Coded by xenoncolt.tk

REM Check if the service is running
tasklist /fi "imagename eq jellyfin-rpc.exe" | find ":" > nul
if %errorlevel%==0 (
    echo ===============================================================================
    echo                      JELLYFIN-RPC SERVICE IS RUNNING
    echo ===============================================================================
) else (
    echo jellyfin-rpc service failed to start.
)
timeout /t 5

echo.
echo ===============================================================================
echo                            INSTALLATION COMPLETE!
echo ===============================================================================
pause >nul
