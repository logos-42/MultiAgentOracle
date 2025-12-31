#!/bin/bash

# WSL部署脚本 - 使用Windows安装的工具
echo "🚀 开始WSL部署（使用Windows工具）"

# 检查WSL环境
if ! grep -q Microsoft /proc/version; then
    echo "❌ 此脚本需要在WSL环境中运行"
    exit 1
fi

echo "✅ 检测到WSL环境"

# Windows工具路径
WINDOWS_SOLANA="/mnt/d/APPs/solana-release-x86_64-pc-windows-msvc/solana-release/bin/solana.exe"
WINDOWS_ANCHOR="/mnt/c/Users/Mechrevo/.cargo/bin/anchor.exe"
WINDOWS_CARGO="/mnt/c/Users/Mechrevo/.cargo/bin/cargo.exe"
WINDOWS_CARGO_HOME="/mnt/c/Users/Mechrevo/.cargo"

# 检查Windows工具是否存在
echo "🔧 检查Windows工具..."
check_windows_tool() {
    if [ -f "$1" ]; then
        echo "✅ 找到: $1"
        return 0
    else
        echo "❌ 未找到: $1"
        return 1
    fi
}

check_windows_tool "$WINDOWS_SOLANA"
check_windows_tool "$WINDOWS_ANCHOR"
check_windows_tool "$WINDOWS_CARGO"

# 设置项目路径
echo "📁 设置项目路径..."
PROJECT_PATH="/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"
if [ ! -d "$PROJECT_PATH" ]; then
    echo "❌ 未找到项目路径: $PROJECT_PATH"
    exit 1
fi

cd "$PROJECT_PATH"
echo "✅ 项目目录: $(pwd)"

# 设置环境变量
echo "⚙️  设置环境变量..."
export CARGO_HOME="$WINDOWS_CARGO_HOME"
export PATH="$PATH:$(dirname "$WINDOWS_SOLANA" | sed 's|/mnt/||' | sed 's|/|\\\\|g'):/mnt/c/Users/Mechrevo/.cargo/bin"

# 创建别名函数来调用Windows工具
solana() {
    "$WINDOWS_SOLANA" "$@"
}

anchor() {
    "$WINDOWS_ANCHOR" "$@"
}

cargo() {
    "$WINDOWS_CARGO" "$@"
}

# 验证工具
echo "🔍 验证工具版本..."
solana --version
anchor --version
cargo --version

# 1. 停止现有验证器
echo "🛑 停止现有验证器..."
pkill -f solana-test-validator 2>/dev/null || true
sleep 2

# 2. 启动本地测试网络
echo "🌐 启动本地测试网络..."
# 注意：solana-test-validator需要在WSL中安装，或者使用Windows版本
# 这里我们假设已经在WSL中安装了solana-test-validator
if command -v solana-test-validator &> /dev/null; then
    solana-test-validator --reset --quiet &
    VALIDATOR_PID=$!
    sleep 5
    
    if ps -p $VALIDATOR_PID > /dev/null; then
        echo "✅ 测试网络已启动 (PID: $VALIDATOR_PID)"
    else
        echo "❌ 无法启动测试验证器"
        echo "💡 提示：请在WSL中安装solana-test-validator"
        echo "   运行: sh -c \"\$(curl -sSfL https://release.solana.com/stable/install)\""
        exit 1
    fi
else
    echo "❌ 未找到solana-test-validator"
    echo "💡 请在WSL中安装："
    echo "   sh -c \"\$(curl -sSfL https://release.solana.com/stable/install)\""
    exit 1
fi

# 3. 配置网络
echo "🔧 配置网络..."
solana config set --url http://localhost:8899

# 4. 创建测试钱包
echo "💰 创建测试钱包..."
if [ ! -f "test-wallet.json" ]; then
    solana-keygen new --outfile test-wallet.json --no-passphrase --silent
    echo "✅ 创建新的测试钱包"
else
    echo "✅ 使用现有测试钱包"
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
    echo "⚠️  Anchor构建失败，尝试其他方法..."
    
    # 检查是否已构建
    if [ -f "target/deploy/solana_oracle.so" ]; then
        echo "✅ 使用现有构建文件"
    else
        echo "❌ 无法构建智能合约"
        kill $VALIDATOR_PID 2>/dev/null || true
        exit 1
    fi
fi

# 7. 部署智能合约
echo "🚀 部署智能合约..."
PROGRAM_ID=""
if [ -f "target/deploy/solana_oracle-keypair.json" ]; then
    PROGRAM_ID=$(grep -o '"pubkey":"[^"]*"' target/deploy/solana_oracle-keypair.json | cut -d'"' -f4)
    echo "📋 程序公钥: $PROGRAM_ID"
fi

if anchor deploy 2>/dev/null; then
    echo "✅ Anchor部署成功"
elif solana program deploy target/deploy/solana_oracle.so 2>/dev/null; then
    echo "✅ 手动部署成功"
else
    echo "❌ 部署失败"
    echo "💡 尝试使用Windows直接部署："
    echo "   1. 在PowerShell中运行: cd '$PROJECT_PATH'"
    echo "   2. 运行: solana config set --url http://localhost:8899"
    echo "   3. 运行: solana program deploy target/deploy/solana_oracle.so"
    kill $VALIDATOR_PID 2>/dev/null || true
    exit 1
fi

# 8. 更新配置
echo "📝 更新配置..."
if [ -f "Anchor.toml" ] && [ -n "$PROGRAM_ID" ]; then
    cp Anchor.toml Anchor.toml.backup
    sed -i "s|solana_oracle = \".*\"|solana_oracle = \"$PROGRAM_ID\"|g" Anchor.toml
    echo "✅ Anchor.toml已更新"
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
echo "1. 更新Rust项目的程序ID: $PROGRAM_ID"
echo "2. 运行测试: anchor test --skip-local-validator"
echo "3. 停止网络: kill $VALIDATOR_PID"
echo ""
echo "📋 保持此终端打开以运行测试网络"
echo "按 Ctrl+C 停止"

# 保存部署信息
cat > deploy-info.txt << EOF
部署时间: $(date)
程序ID: $PROGRAM_ID
网络: http://localhost:8899
钱包: test-wallet.json
验证器PID: $VALIDATOR_PID
Windows工具路径:
  solana: $WINDOWS_SOLANA
  anchor: $WINDOWS_ANCHOR
  cargo: $WINDOWS_CARGO
EOF

echo "✅ 部署信息已保存到: deploy-info.txt"

# 等待用户中断
trap "echo '停止验证器...'; kill $VALIDATOR_PID 2>/dev/null || true; echo '完成！'; exit 0" INT

while true; do
    sleep 10
done
