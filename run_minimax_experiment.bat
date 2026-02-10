@echo off
chcp 65001 > nul
echo ========================================
echo 使用 Minimax API 运行 25 轮实验
echo ========================================
echo.
echo 开始时间: %date% %time%
echo.
echo 实验预计耗时: 约 100-120 分钟
echo 实验结果将保存到: experiments\output\real_experiment_*\ 目录
echo.
echo 注意: 请勿关闭此窗口，否则实验将被中断
echo ========================================
echo.

cd /d "d:\AI\预言机多智能体\MultiAgentOracle"
cargo run --example real_benchmark_experiment_10_agents 25

echo.
echo ========================================
echo 实验完成或被中断
echo 结束时间: %date% %time%
echo ========================================
echo.
echo 请检查: experiments\output\ 目录中的最新实验结果
pause
