@echo off

REM Check if running with administrator privileges
net session >nul 2>&1
if %errorlevel% == 0 (
    echo Running with administrator privileges
) else (
    echo ERROR: This batch file must be run with administrator privileges.
    echo Please right-click on the batch file and select "Run as administrator"
    pause >nul
    exit /b
)



echo ===============================================================================
echo                        JELLYFIN-RPC UNINSTALLATION  
echo ===============================================================================
echo.
timeout /t 3 >nul

REM set path
set NSSM_PATH=%APPDATA%\jellyfin-rpc\nssm-2.24\win64\nssm.exe
set MAIN_PATH=%APPDATA%\jellyfin-rpc



echo Stopping jellyfin-rpc from service & timeout /t 5 >nul
%NSSM_PATH% stop jellyfin-rpc

timeout /t 2 >nul

echo ===============================================================================
echo                           JELLYFIN-RPC STOPPED!  
echo ===============================================================================
echo. 


set /p =Hit ENTER to continue uninstallation...
echo Removing jellyfin-rpc from service...
%NSSM_PATH% remove jellyfin-rpc

timeout /t 5 >nul

echo Removing jellyfin-rpc folder...
rd /s /q "%APPDATA%\jellyfin-rpc"
echo jellyfin-rpc folder removed successfully.

echo.
echo ===============================================================================
echo                         UNINSTALLATION COMPLETE!
echo =============================================================================== 
pause >nul

