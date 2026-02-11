@echo off
chcp 65001 >nul
echo ==========================================
echo  多智能体预言机实验 - 完整版谱分析
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
