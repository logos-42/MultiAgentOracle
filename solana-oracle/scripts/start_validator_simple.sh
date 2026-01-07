#!/bin/bash

# 简单验证器启动脚本
echo "🔧 启动Solana本地验证器"

# 设置环境
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 停止现有验证器
echo "停止现有验证器..."
pkill -f solana-test-validator 2>/dev/null || true
sleep 2

# 检查WSL资源
echo "检查WSL资源..."
echo "内存: $(free -h | grep Mem | awk '{print $2}')"
echo "磁盘: $(df -h / | tail -1 | awk '{print $4}') 可用"

# 尝试启动验证器
echo "启动验证器..."
echo "命令: solana-test-validator --reset --log -r"

# 在后台启动并记录日志
solana-test-validator --reset --log > validator.log 2>&1 &
VALIDATOR_PID=$!

echo "验证器PID: $VALIDATOR_PID"
echo "等待启动..."
sleep 8

# 检查是否运行
if ps -p $VALIDATOR_PID > /dev/null; then
    echo "✅ 验证器进程正在运行 (PID: $VALIDATOR_PID)"
else
    echo "❌ 验证器进程已退出"
    echo "查看日志..."
    tail -20 validator.log 2>/dev/null || echo "无日志文件"
    exit 1
fi

# 检查网络连接
echo "检查网络连接..."
for i in {1..5}; do
    if solana cluster-version --url http://localhost:8899 2>&1 | grep -q "1."; then
        echo "✅ 验证器响应正常"
        break
    fi
    echo "尝试 $i/5..."
    sleep 2
done

if solana cluster-version --url http://localhost:8899 2>&1 | grep -q "1."; then
    echo "🎉 验证器启动成功！"
    echo "RPC URL: http://localhost:8899"
    echo "日志文件: validator.log"
    echo ""
    echo "保持验证器运行，按 Ctrl+C 停止"
    
    # 显示日志尾部
    echo "=== 最近日志 ==="
    tail -10 validator.log 2>/dev/null || echo "无日志"
    
    # 等待用户中断
    trap "echo '停止验证器...'; kill $VALIDATOR_PID 2>/dev/null; echo '完成！'; exit 0" INT
    while true; do sleep 10; done
else
    echo "❌ 验证器未响应"
    echo "查看详细日志..."
    tail -50 validator.log 2>/dev/null || echo "无日志文件"
    kill $VALIDATOR_PID 2>/dev/null
    exit 1
fi
