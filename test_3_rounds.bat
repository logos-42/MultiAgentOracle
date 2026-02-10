@echo off
chcp 65001 >nul
echo ============================================
echo  多智能体预言机系统 - 3轮测试
echo ============================================
echo.
echo 正在构建（使用单线程以减少内存占用）...
echo.

cd /d "d:\AI\预言机多智能体\MultiAgentOracle"

:: 使用单线程构建以减少内存占用
cargo build --example real_benchmark_experiment_10_agents --jobs 1

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [错误] 构建失败！可能是内存不足。
    echo.
    echo 建议解决方案：
    echo 1. 关闭其他占用内存的程序
    echo 2. 增加虚拟内存（页面文件）大小
    echo 3. 使用更小的测试用例
    echo.
    pause
    exit /b 1
)

echo.
echo ============================================
echo  构建成功！开始运行测试（3轮）
echo ============================================
echo.

cargo run --example real_benchmark_experiment_10_agents -- 3

echo.
echo ============================================
echo  测试完成！
echo ============================================
echo.
pause
