@echo off
REM Configuration Validation Script
echo Validating JSON configurations...
echo.

set CONFIG_COUNT=0
set VALID_COUNT=0

for %%f in (examples\configs\*.json) do (
    set /a CONFIG_COUNT+=1
    echo Checking %%f...
    
    REM Basic JSON syntax check using Python if available
    python -m json.tool "%%f" > nul 2>&1
    if %ERRORLEVEL% EQU 0 (
        echo   ✓ Valid JSON syntax
        set /a VALID_COUNT+=1
    ) else (
        echo   ✗ Invalid JSON syntax
    )
    echo.
)

echo ========================================
echo Validation Results:
echo   Total configs: %CONFIG_COUNT%
echo   Valid configs: %VALID_COUNT%
echo ========================================

if %VALID_COUNT% EQU %CONFIG_COUNT% (
    echo ✓ All configurations are valid!
    exit /b 0
) else (
    echo ✗ Some configurations have errors.
    exit /b 1
)
