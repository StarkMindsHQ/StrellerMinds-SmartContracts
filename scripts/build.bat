@echo off
REM StrellerMinds Smart Contracts - Build Script (Windows)
REM This script builds all smart contracts for Windows environment

echo [BUILD] Building smart contracts...

REM Check for required tools
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: 'cargo' not found. Please install Rust: https://www.rust-lang.org/tools/install
    exit /b 1
)

where soroban >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: 'soroban' CLI not found. Please install Soroban CLI: https://soroban.stellar.org/docs/getting-started/installation
    exit /b 1
)

where wasm-opt >nul 2>nul
if %errorlevel% neq 0 (
    echo Warning: 'wasm-opt' not found. Fallback optimization will not be available. Install Binaryen: https://github.com/WebAssembly/binaryen
)

REM Print environment info
echo Environment Info:
rustc --version 2>nul || echo Rust: Not installed
cargo --version 2>nul || echo Cargo: Not installed
soroban --version 2>nul || echo Soroban: Not installed
wasm-opt --version 2>nul || echo wasm-opt: Not installed

REM Create target directory if it doesn't exist
if not exist "target\wasm32-unknown-unknown\release" mkdir "target\wasm32-unknown-unknown\release"

REM Logging setup
set LOGFILE=build.log
echo --- Build started at %date% %time% --- > "%LOGFILE%"

REM Build contracts
set success_contracts=
set failed_contracts=

if "%~1"=="" (
    echo Building all contracts...
    set contracts=contracts\*
) else (
    echo Building contract: %1
    set contracts=contracts\%1
)

for /d %%c in (%contracts%) do (
    set contract_name=%%~nc
    echo Building !contract_name! contract...
    
    cargo build --target wasm32-unknown-unknown --release -p !contract_name! >> "%LOGFILE%" 2>&1
    if %errorlevel% equ 0 (
        if exist "target\wasm32-unknown-unknown\release\!contract_name!.wasm" (
            echo Optimizing !contract_name!.wasm...
            soroban contract optimize --wasm "target\wasm32-unknown-unknown\release\!contract_name!.wasm" --wasm-out "target\wasm32-unknown-unknown\release\!contract_name!.optimized.wasm" >> "%LOGFILE%" 2>&1
            if %errorlevel% equ 0 (
                echo Optimization succeeded for !contract_name!
                set success_contracts=!success_contracts! !contract_name!
            ) else (
                echo Warning: soroban optimize failed for !contract_name!
                where wasm-opt >nul 2>nul
                if %errorlevel% equ 0 (
                    wasm-opt -Oz "target\wasm32-unknown-unknown\release\!contract_name!.wasm" -o "target\wasm32-unknown-unknown\release\!contract_name!.optimized.wasm" >> "%LOGFILE%" 2>&1
                    if %errorlevel% equ 0 (
                        echo Fallback optimization succeeded for !contract_name!
                        set success_contracts=!success_contracts! !contract_name!
                    ) else (
                        echo Error: Optimization failed for !contract_name!
                        set failed_contracts=!failed_contracts! !contract_name!
                    )
                ) else (
                    echo Error: wasm-opt not found for fallback optimization
                    set failed_contracts=!failed_contracts! !contract_name!
                )
            )
        ) else (
            echo Error: WASM file not found for !contract_name!
            set failed_contracts=!failed_contracts! !contract_name!
        )
    ) else (
        echo Error: Build failed for !contract_name!
        set failed_contracts=!failed_contracts! !contract_name!
    )
)

REM Print summary
echo.
echo Build completed!
if not "%success_contracts%"=="" (
    echo Contracts built and optimized successfully:%success_contracts%
)
if not "%failed_contracts%"=="" (
    echo Contracts with errors:%failed_contracts%
    echo See %LOGFILE% for details.
    exit /b 4
)

echo All contracts built successfully!
exit /b 0
