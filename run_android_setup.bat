@echo off
setlocal

rem ==== 1. Verify Prerequisites ==== 
rem Install missing tools via winget (auto‑accept agreements)
where git >nul 2>&1
if errorlevel 1 (
    winget install --id Git.Git -e --accept-source-agreements --accept-package-agreements
)

where node >nul 2>&1
if errorlevel 1 (
    winget install --id OpenJS.Nodejs -e --accept-source-agreements --accept-package-agreements
)

where java >nul 2>&1
if errorlevel 1 (
    winget install --id EclipseAdoptium.Temurin.11 -e --accept-source-agreements --accept-package-agreements
)

where adb >nul 2>&1
if errorlevel 1 (
    echo Installing Android Studio (auto‑accept agreements)…
    winget install --id Google.AndroidStudio -e --accept-source-agreements --accept-package-agreements
)

rem ==== 2. Repository Setup ==== 
call scripts\setup.sh
if %errorlevel% neq 0 (
    echo Setup script failed. Exiting.
    exit /b %errorlevel%
)

rem ==== 3. Android Build & Install ==== 
cd mobile-app\android
call gradlew clean assembleDebug installDebug
if %errorlevel% neq 0 (
    echo Android build failed. Exiting.
    exit /b %errorlevel%
)

rem ==== 4. (Optional) Run Unit Tests ==== 
rem call gradlew test

rem ==== 5. (Optional) Run Instrumented Tests ==== 
rem call gradlew connectedAndroidTest

endlocal
