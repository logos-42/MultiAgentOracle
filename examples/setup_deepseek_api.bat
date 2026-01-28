@echo off
REM 设置 DeepSeek API 密钥
REM 使用方法: setup_api_keys.bat YOUR_DEEPSEEK_API_KEY

if "%1"=="" (
    echo 用法: setup_api_keys.bat YOUR_DEEPSEEK_API_KEY
    echo.
    echo 示例:
    echo   setup_api_keys.bat sk-xxxxxxxxxxxxxxxxxxxxx
    echo.
    echo 或者手动设置环境变量:
    echo   set DEEPSEEK_API_KEY=your_key_here
    pause
    exit /b 1
)

set DEEPSEEK_API_KEY=%1
echo ✅ 已设置 DEEPSEEK_API_KEY
echo.
echo 现在可以使用以下命令运行实验:
echo   cargo run --example zk_fingerprint_experiment -- --use-api
echo.
echo 或使用其他提供商:
echo   --provider openai --model gpt-4
echo   --provider anthropic --model claude-3-opus-20240229
echo.
pause
