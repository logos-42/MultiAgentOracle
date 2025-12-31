#!/bin/bash

# 最终部署脚本
echo "🎯 最终部署智能合约"

# 设置环境
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

cd "/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"

echo "📁 项目目录: $(pwd)"

# 1. 停止现有验证器
echo "🛑 停止现有验证器..."
pkill -f solana-test-validator 2>/dev/null || echo "无验证器运行"
sleep 3

# 2. 启动验证器
echo "🌐 启动验证器..."
solana-test-validator --reset --quiet &
VALIDATOR_PID=$!
echo "验证器PID: $VALIDATOR_PID"
sleep 5

# 3. 检查验证器
echo "🔍 检查验证器..."
if solana cluster-version --url http://localhost:8899 2>&1 | grep -q "1."; then
    echo "✅ 验证器运行正常"
else
    echo "❌ 验证器未运行"
    exit 1
fi

# 4. 配置网络
echo "🔧 配置网络..."
solana config set --url http://localhost:8899
solana config set --keypair test-wallet.json

# 5. 获取测试SOL
echo "💰 获取测试SOL..."
solana airdrop 10 2>/dev/null || echo "可能已有足够余额"
BALANCE=$(solana balance)
echo "余额: $BALANCE"

# 6. 部署合约
echo "🚀 部署智能合约..."
echo "程序公钥: $(solana-keygen pubkey target/deploy/solana_oracle-keypair.json)"

if anchor deploy; then
    echo "✅ Anchor部署成功"
else
    echo "尝试手动部署..."
    if solana program deploy target/deploy/solana_oracle.so; then
        echo "✅ 手动部署成功"
    else
        echo "❌ 部署失败"
        exit 1
    fi
fi

# 7. 获取程序ID
PROGRAM_ID=$(solana-keygen pubkey target/deploy/solana_oracle-keypair.json)
echo "📋 程序ID: $PROGRAM_ID"

# 8. 验证部署
echo "🔍 验证部署..."
if solana program show $PROGRAM_ID 2>/dev/null; then
    echo "✅ 部署验证成功"
else
    echo "⚠️  部署验证警告"
fi

# 9. 更新配置
echo "📝 更新配置..."
if [ -f "Anchor.toml" ]; then
    cp Anchor.toml Anchor.toml.backup
    sed -i "s|solana_oracle = \".*\"|solana_oracle = \"$PROGRAM_ID\"|g" Anchor.toml
    echo "✅ Anchor.toml已更新"
fi

# 10. 显示结果
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
echo "1. 更新Rust项目配置"
echo "2. 运行测试: anchor test --skip-local-validator"
echo "3. 停止验证器: kill $VALIDATOR_PID"

# 保存信息
cat > DEPLOYMENT_SUCCESS.md << EOF
# 智能合约部署成功

## 部署信息
- **时间**: $(date)
- **程序ID**: $PROGRAM_ID
- **网络**: http://localhost:8899
- **钱包**: test-wallet.json
- **余额**: $BALANCE
- **验证器PID**: $VALIDATOR_PID

## 验证命令
\`\`\`bash
# 检查程序
solana program show $PROGRAM_ID

# 检查余额
solana balance

# 停止验证器
kill $VALIDATOR_PID
\`\`\`

## 下一步
1. 更新Rust项目中的程序ID配置
2. 运行集成测试
3. 部署到devnet/testnet
EOF

echo "✅ 部署信息保存到: DEPLOYMENT_SUCCESS.md"

# 保持运行
echo ""
echo "📋 验证器正在运行，按 Ctrl+C 停止"
trap "echo '停止验证器...'; kill $VALIDATOR_PID 2>/dev/null; echo '完成！'; exit 0" INT
while true; do sleep 10; done
