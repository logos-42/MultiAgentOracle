@echo off
chcp 65001 >nul
echo ==========================================
echo  多智能体预言机实验 - DeepSeek API
echo  配置: 10智能体 / 中位数ground_truth
echo ==========================================
echo.
echo 开始时间: %date% %time%
echo.

cargo run --example real_benchmark_experiment_10_agents --release

echo.
echo ==========================================
echo 实验完成时间: %date% %time%
echo ==========================================
pause
