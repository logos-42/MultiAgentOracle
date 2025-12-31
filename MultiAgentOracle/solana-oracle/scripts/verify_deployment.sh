#!/bin/bash

# 部署验证脚本
echo "🔍 验证智能合约部署"

# 设置环境
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

cd "/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"

echo "📁 项目目录: $(pwd)"

# 程序ID
PROGRAM_ID="DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b"
echo "📋 程序ID: $PROGRAM_ID"

echo ""
echo "=== 验证步骤 ==="

# 1. 检查验证器是否运行
echo "1. 检查本地验证器..."
if solana cluster-version --url http://localhost:8899 2>&1 | grep -q "1."; then
    echo "✅ 本地验证器运行中"
else
    echo "❌ 本地验证器未运行"
    echo "   启动命令: solana-test-validator --reset --quiet &"
fi

# 2. 检查网络配置
echo ""
echo "2. 检查网络配置..."
solana config get

# 3. 检查钱包余额
echo ""
echo "3. 检查钱包余额..."
BALANCE=$(solana balance 2>/dev/null || echo "无法获取余额")
echo "   余额: $BALANCE"

# 4. 检查程序是否部署
echo ""
echo "4. 检查程序部署..."
if solana program show $PROGRAM_ID --url http://localhost:8899 2>&1 | grep -q "Program Id:"; then
    echo "✅ 程序已部署到本地网络"
    solana program show $PROGRAM_ID --url http://localhost:8899 | head -5
else
    echo "❌ 程序未部署到本地网络"
    echo "   部署命令: anchor deploy 或 solana program deploy target/deploy/solana_oracle.so"
fi

# 5. 检查devnet部署
echo ""
echo "5. 检查devnet部署..."
if solana program show $PROGRAM_ID --url https://api.devnet.solana.com 2>&1 | grep -q "Program Id:"; then
    echo "✅ 程序已部署到devnet"
else
    echo "⚠️  程序未部署到devnet"
    echo "   部署命令: anchor deploy --provider.cluster devnet"
fi

# 6. 检查构建文件
echo ""
echo "6. 检查构建文件..."
if [ -f "target/deploy/solana_oracle.so" ]; then
    SIZE=$(stat -c%s "target/deploy/solana_oracle.so" 2>/dev/null || echo "未知")
    echo "✅ 找到程序文件: target/deploy/solana_oracle.so ($((SIZE/1024)) KB)"
else
    echo "❌ 未找到程序文件"
    echo "   构建命令: anchor build"
fi

# 7. 检查配置文件
echo ""
echo "7. 检查配置文件..."
if grep -q "solana_oracle = \"$PROGRAM_ID\"" Anchor.toml; then
    echo "✅ Anchor.toml配置正确"
else
    echo "❌ Anchor.toml配置不匹配"
    echo "   当前配置: $(grep "solana_oracle = " Anchor.toml)"
fi

# 8. 检查Rust项目配置
echo ""
echo "8. 检查Rust项目配置..."
RUST_CONFIG="/mnt/d/AI/预言机多智能体/MultiAgentOracle/src/solana/types.rs"
if [ -f "$RUST_CONFIG" ] && grep -q "\"$PROGRAM_ID\"" "$RUST_CONFIG"; then
    echo "✅ Rust项目配置正确"
else
    echo "⚠️  Rust项目配置需要检查"
fi

# 总结
echo ""
echo "=== 验证总结 ==="
echo "程序ID: $PROGRAM_ID"
echo "本地网络: http://localhost:8899"
echo "Devnet: https://api.devnet.solana.com"
echo ""
echo "🚀 部署状态:"

if solana program show $PROGRAM_ID --url http://localhost:8899 2>&1 | grep -q "Program Id:"; then
    echo "✅ 本地部署: 成功"
else
    echo "❌ 本地部署: 未完成"
fi

if solana program show $PROGRAM_ID --url https://api.devnet.solana.com 2>&1 | grep -q "Program Id:"; then
    echo "✅ Devnet部署: 成功"
else
    echo "⚠️  Devnet部署: 未完成 (可选)"
fi

echo ""
echo "💡 建议操作:"
echo "1. 启动本地测试网: solana-test-validator --reset --quiet &"
echo "2. 部署到本地: anchor deploy"
echo "3. 运行测试: anchor test --skip-local-validator"
echo "4. 部署到devnet: anchor deploy --provider.cluster devnet"
echo "5. 获取devnet测试SOL: solana airdrop 1 --url https://api.devnet.solana.com"

# 保存验证报告
cat > VERIFICATION_REPORT.md << EOF
# 智能合约部署验证报告

## 验证时间
$(date)

## 程序信息
- **程序ID**: $PROGRAM_ID
- **本地网络**: http://localhost:8899
- **Devnet**: https://api.devnet.solana.com

## 验证结果
$(if solana program show $PROGRAM_ID --url http://localhost:8899 2>&1 | grep -q "Program Id:"; then
    echo "- ✅ 本地部署: 成功"
else
    echo "- ❌ 本地部署: 失败"
fi)

$(if solana program show $PROGRAM_ID --url https://api.devnet.solana.com 2>&1 | grep -q "Program Id:"; then
    echo "- ✅ Devnet部署: 成功"
else
    echo "- ⚠️  Devnet部署: 未完成"
fi)

## 系统状态
- **钱包余额**: $BALANCE
- **程序文件**: $(if [ -f "target/deploy/solana_oracle.so" ]; then echo "存在"; else echo "不存在"; fi)
- **配置文件**: $(if grep -q "solana_oracle = \"$PROGRAM_ID\"" Anchor.toml; then echo "正确"; else echo "需要更新"; fi)

## 下一步
1. 完成本地部署测试
2. 运行智能合约测试
3. 部署到devnet进行公开测试
4. 集成到多智能体预言机系统

EOF

echo "✅ 验证报告保存到: VERIFICATION_REPORT.md"
