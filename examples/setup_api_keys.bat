@echo off
REM Windows API密钥配置脚本
REM 设置真实API测试所需的环境变量

echo ========================================
echo 🔑 设置API密钥（真实数据测试）
echo ========================================
echo.

REM 提示用户输入API密钥
echo 请输入你的API密钥（如果没有，按回车跳过）:
echo.

set /p ALPHA_VANTAGE_KEY="Alpha Vantage API Key (股票数据): "
set /p OPENWEATHER_KEY="OpenWeatherMap API Key (天气数据): "
set /p EXCHANGERATE_KEY="ExchangeRate API Key (外汇数据): "

REM 设置环境变量
if not "%ALPHA_VANTAGE_KEY%"=="" (
    setx ALPHA_VANTAGE_API_KEY "%ALPHA_VANTAGE_KEY%"
    echo ✅ Alpha Vantage API Key 已设置
) else (
    echo ⚠️  Alpha Vantage 使用demo模式
)

if not "%OPENWEATHER_KEY%"=="" (
    setx OPENWEATHER_API_KEY "%OPENWEATHER_KEY%"
    echo ✅ OpenWeatherMap API Key 已设置
) else (
    echo ⚠️  OpenWeatherMap 使用demo模式
)

if not "%EXCHANGERATE_KEY%"=="" (
    setx EXCHANGERATE_API_KEY "%EXCHANGERATE_KEY%"
    echo ✅ ExchangeRate API Key 已设置
) else (
    echo ⚠️  ExchangeRate API 使用demo模式
)

echo.
echo ========================================
echo 💡 提示：
echo    重启终端后环境变量生效
echo    或使用 PowerShell 立即生效:
echo    $env:ALPHA_VANTAGE_API_KEY="your_key"
echo ========================================

pause
