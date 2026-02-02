@echo off
chcp 65001 >nul
echo ╔══════════════════════════════════════════════════════════╗
echo ║     多智能体预言机系统 - 论文级实验测试                   ║
echo ╚══════════════════════════════════════════════════════════╝
echo.

:: 设置输出目录
set OUTPUT_DIR=%~dp0output
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

echo 📁 输出目录: %OUTPUT_DIR%
echo.

:: 运行实验
echo 🔬 步骤1: 运行基准测试实验...
cargo run --example paper_benchmark_experiment --release

if errorlevel 1 (
    echo ❌ 实验运行失败
    exit /b 1
)

echo.
echo ✅ 实验数据收集完成！
echo.

:: 查找最新的实验目录
for /f "delims=" %%i in ('dir /b /ad /o-d "%OUTPUT_DIR%\experiment_*" 2^>nul') do (
    set LATEST_EXP=%%i
    goto :found
)

echo ❌ 未找到实验输出目录
exit /b 1

:found
echo 📊 步骤2: 生成论文图表...
echo    实验目录: %LATEST_EXP%
echo.

:: 检查Python是否可用
python --version >nul 2>&1
if errorlevel 1 (
    echo ⚠️  未找到Python，跳过图表生成
    echo    请手动运行: python examples\generate_paper_plots.py %OUTPUT_DIR%\%LATEST_EXP%
) else (
    python examples\generate_paper_plots.py "%OUTPUT_DIR%\%LATEST_EXP%"
)

echo.
echo ╔══════════════════════════════════════════════════════════╗
echo ║                    实验执行完成                          ║
echo ╚══════════════════════════════════════════════════════════╝
echo.
echo 📁 实验数据位置: %OUTPUT_DIR%\%LATEST_EXP%
echo.
echo 生成的文件:
echo   - full_report.json      完整实验数据（JSON）
echo   - raw_data.csv          原始数据（CSV）
echo   - summary.md            实验摘要（Markdown）
echo   - tables.tex            LaTeX表格代码
echo   - plot_*.csv            图表数据
echo   - fig_*.png/pdf         论文图表
echo.
echo 📝 论文写作建议:
echo   1. 使用 tables.tex 中的LaTeX代码插入表格
echo   2. 使用 fig_*.png 或 fig_*.pdf 插入图表
echo   3. 使用 summary.md 作为实验章节草稿
echo   4. 使用 raw_data.csv 进行进一步分析
echo.
pause
