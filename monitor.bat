@echo off
title Voxtral Live Transcription
echo.
echo  ========================================
echo   Voxtral Live Transcription Monitor
echo  ========================================
echo   Output: C:\workspace\voxtral\output\transcription.log
echo   Press Ctrl+C to stop.
echo.

:: Create the file if it doesn't exist yet
if not exist "C:\workspace\voxtral\output\transcription.log" (
    echo   Waiting for first transcription...
    echo.
)

powershell -NoProfile -Command ^
  "Get-Content 'C:\workspace\voxtral\output\transcription.log' -Wait -Tail 30 -ErrorAction SilentlyContinue"
