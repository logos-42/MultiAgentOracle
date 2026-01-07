#!/bin/bash

# 简化的WSL部署脚本
echo "🚀 开始WSL部署流程"

# 检查WSL
if ! grep -q Microsoft /proc/version; then
    echo "错误: 请在WSL中运行此脚本"
    exit 1
fi

# 设置项目路径
echo "📁 设置项目路径..."
WINDOWS_PATH="/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"
LOCAL_PATH="$HOME/solana-oracle-deploy"

if [ ! -d "$WINDOWS_PATH" ]; then
    echo "错误: 未找到项目路径: $WINDOWS_PATH"
    echo "请确保项目在D:\\AI\\预言机多智能体\\MultiAgentOracle\\solana-oracle"
    exit 1
fi

# 复制项目到WSL
echo "复制项目到WSL..."
rm -rf "$LOCAL_PATH"
cp -r "$WINDOWS_PATH" "$LOCAL_PATH"
cd "$LOCAL_PATH"

echo "✅ 项目准备完成: $(pwd)"

# 1. 停止现有验证器
echo "🛑 停止现有验证器..."
pkill -f solana-test-validator 2>/dev/null || true
sleep 2

# 2. 启动本地测试网络
echo "🌐 启动本地测试网络..."
solana-test-validator --reset --quiet &
VALIDATOR_PID=$!
sleep 5

if ! ps -p $VALIDATOR_PID > /dev/null; then
    echo "错误: 无法启动测试验证器"
    exit 1
fi

echo "✅ 测试网络已启动 (PID: $VALIDATOR_PID)"

# 3. 配置网络
echo "🔧 配置网络..."
solana config set --url http://localhost:8899

# 4. 创建测试钱包
echo "💰 创建测试钱包..."
if [ ! -f "test-wallet.json" ]; then
    solana-keygen new --outfile test-wallet.json --no-passphrase --silent
fi
solana config set --keypair test-wallet.json

# 5. 获取测试SOL
echo "🪙 获取测试SOL..."
solana airdrop 10
BALANCE=$(solana balance)
echo "✅ 余额: $BALANCE"

# 6. 构建智能合约
echo "🔨 构建智能合约..."
if anchor build 2>/dev/null; then
    echo "✅ Anchor构建成功"
else
    echo "尝试cargo构建..."
    cd programs/solana-oracle
    if cargo build-sbf --sbf-out-dir ../../target/deploy 2>/dev/null; then
        echo "✅ Cargo构建成功"
        cd ../..
    else
        echo "❌ 构建失败"
        kill $VALIDATOR_PID
        exit 1
    fi
fi

# 7. 部署智能合约
echo "🚀 部署智能合约..."
PROGRAM_ID=$(solana-keygen pubkey target/deploy/solana_oracle-keypair.json 2>/dev/null || echo "未知")

if anchor deploy 2>/dev/null; then
    echo "✅ Anchor部署成功"
elif solana program deploy target/deploy/solana_oracle.so 2>/dev/null; then
    echo "✅ 手动部署成功"
else
    echo "❌ 部署失败"
    kill $VALIDATOR_PID
    exit 1
fi

# 8. 更新配置
echo "📝 更新配置..."
if [ -f "Anchor.toml" ]; then
    sed -i "s|solana_oracle = \".*\"|solana_oracle = \"$PROGRAM_ID\"|g" Anchor.toml
fi

# 9. 验证部署
echo "🔍 验证部署..."
if solana program show $PROGRAM_ID 2>/dev/null | grep -q "Program Id:"; then
    echo "✅ 部署验证成功"
else
    echo "⚠️  部署验证警告"
fi

# 显示结果
echo ""
echo "🎉 部署完成！"
echo "========================"
echo "程序ID: $PROGRAM_ID"
echo "网络: http://localhost:8899"
echo "钱包: test-wallet.json"
echo "余额: $BALANCE"
echo "验证器PID: $VALIDATOR_PID"
echo ""
echo "💡 下一步:"
echo "1. 更新Rust项目的程序ID"
echo "2. 运行测试: anchor test --skip-local-validator"
echo "3. 停止网络: kill $VALIDATOR_PID"
echo ""
echo "📋 保持此终端打开以运行测试网络"
echo "按 Ctrl+C 停止"

# 等待
trap "echo '停止验证器...'; kill $VALIDATOR_PID; echo '完成！'; exit 0" INT
while true; do sleep 10; done
