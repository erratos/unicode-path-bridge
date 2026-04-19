@echo off
setlocal

set CSC=C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe

if not exist "%CSC%" (
    echo ERROR: .NET Framework 4.x compiler not found at:
    echo   %CSC%
    echo.
    echo Make sure .NET Framework 4.x is installed.
    exit /b 1
)

echo Building Unicode Path Bridge...
"%CSC%" /nologo /target:winexe /win32manifest:src\ubp.manifest /out:ubp.exe src\ubp.cs

if %ERRORLEVEL% EQU 0 (
    echo.
    echo   OK: ubp.exe created successfully.
    echo.
    echo   Next steps:
    echo     1. Copy ubp.exe to a permanent location (e.g. C:\Tools\)
    echo     2. Set up a registry entry — see examples\ folder
    echo     3. Right-click a file to test
) else (
    echo.
    echo   ERROR: Build failed.
    exit /b 1
)
