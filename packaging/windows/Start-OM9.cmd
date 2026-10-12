@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Launch-OM9.ps1"
if errorlevel 1 pause
