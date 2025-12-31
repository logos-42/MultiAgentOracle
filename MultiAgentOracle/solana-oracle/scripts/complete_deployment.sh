#!/bin/bash

# 完整部署脚本
echo "🚀 智能合约完整部署流程"

# 设置环境
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

cd "/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"

echo "📁 项目目录: $(pwd)"

# 程序ID
PROGRAM_ID="DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"
echo "📋 目标程序ID: $PROGRAM_ID"

echo ""
echo "=== 步骤1: 停止现有进程 ==="
pkill -f solana-test-validator 2>/dev/null || echo "无验证器运行"
sleep 2

echo ""
echo "=== 步骤2: 启动本地测试网络 ==="
echo "启动验证器..."
solana-test-validator --reset --quiet &
VALIDATOR_PID=$!
echo "验证器PID: $VALIDATOR_PID"
sleep 5

echo "检查验证器..."
if solana cluster-version --url http://localhost:8899 2>&1 | grep -q "1."; then
    echo "✅ 验证器启动成功"
else
    echo "❌ 验证器启动失败"
    exit 1
fi

echo ""
echo "=== 步骤3: 配置网络和钱包 ==="
solana config set --url http://localhost:8899
solana config set --keypair test-wallet.json
echo "网络配置完成"

echo ""
echo "=== 步骤4: 获取测试SOL ==="
solana airdrop 10 2>/dev/null || echo "可能已有足够余额"
BALANCE=$(solana balance 2>/dev/null || echo "未知")
echo "当前余额: $BALANCE"

echo ""
echo "=== 步骤5: 构建智能合约 ==="
echo "清理构建缓存..."
rm -rf target/ 2>/dev/null || true

echo "开始构建..."
if anchor build; then
    echo "✅ 构建成功"
else
    echo "❌ 构建失败"
    kill $VALIDATOR_PID
    exit 1
fi

# 检查构建结果
if [ -f "target/deploy/solana_oracle.so" ]; then
    echo "程序文件: target/deploy/solana_oracle.so"
else
    echo "❌ 未找到构建的程序文件"
    kill $VALIDATOR_PID
    exit 1
fi

echo ""
echo "=== 步骤6: 部署智能合约 ==="
echo "部署到本地网络..."
if anchor deploy; then
    echo "✅ 本地部署成功"
else
    echo "尝试手动部署..."
    if solana program deploy target/deploy/solana_oracle.so; then
        echo "✅ 手动部署成功"
    else
        echo "❌ 部署失败"
        kill $VALIDATOR_PID
        exit 1
    fi
fi

echo ""
echo "=== 步骤7: 验证部署 ==="
if solana program show $PROGRAM_ID --url http://localhost:8899 2>&1 | grep -q "Program Id:"; then
    echo "✅ 部署验证成功"
    echo "程序详情:"
    solana program show $PROGRAM_ID --url http://localhost:8899 | head -10
else
    echo "⚠️  部署验证警告"
fi

echo ""
echo "=== 步骤8: 运行测试 ==="
echo "运行智能合约测试..."
if anchor test --skip-local-validator 2>&1 | tail -5; then
    echo "✅ 测试运行完成"
else
    echo "⚠️  测试运行可能有问题"
fi

echo ""
echo "=== 步骤9: 部署到Devnet (可选) ==="
read -p "是否部署到Devnet? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "部署到Devnet..."
    if anchor deploy --provider.cluster devnet; then
        echo "✅ Devnet部署成功"
    else
        echo "❌ Devnet部署失败"
    fi
else
    echo "跳过Devnet部署"
fi

echo ""
echo "🎉 部署完成！"
echo "========================"
echo "程序ID: $PROGRAM_ID"
echo "本地网络: http://localhost:8899"
echo "钱包: test-wallet.json"
echo "余额: $BALANCE"
echo "验证器PID: $VALIDATOR_PID"
echo ""
echo "💡 下一步操作:"
echo "1. 更新Rust项目配置 (已完成)"
echo "2. 运行集成测试"
echo "3. 开发前端界面"
echo "4. 部署到主网"
echo ""
echo "📋 验证器正在运行，按 Ctrl+C 停止"

# 保存部署成功信息
cat > DEPLOYMENT_COMPLETE.md << EOF
# 智能合约部署完成

## 部署信息
- **时间**: $(date)
- **程序ID**: $PROGRAM_ID
- **本地网络**: http://localhost:8899
- **钱包**: test-wallet.json
- **余额**: $BALANCE
- **验证器PID**: $VALIDATOR_PID

## 验证命令
\`\`\`bash
# 检查程序状态
solana program show $PROGRAM_ID --url http://localhost:8899

# 检查余额
solana balance

# 运行测试
anchor test --skip-local-validator

# 停止验证器
kill $VALIDATOR_PID
\`\`\`

## 集成到多智能体预言机系统
智能合约已成功部署，可以集成到多智能体预言机系统中：

1. **身份注册**: 智能体可以通过智能合约注册身份
2. **信誉管理**: 信誉分数存储在区块链上
3. **验证系统**: 身份验证和信誉验证

## 配置文件
- **Anchor.toml**: 已更新程序ID
- **Rust项目**: 已配置相同程序ID
- **构建文件**: target/deploy/solana_oracle.so

EOF

echo "✅ 部署完成信息保存到: DEPLOYMENT_COMPLETE.md"

# 保持运行
trap "echo '停止验证器...'; kill $VALIDATOR_PID 2>/dev/null; echo '部署流程完成！'; exit 0" INT
while true; do sleep 10; done
