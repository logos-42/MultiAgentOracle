#!/bin/bash

# 分步部署脚本
echo "🔧 分步部署智能合约"

# 设置环境变量
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 步骤1: 检查环境
echo "=== 步骤1: 检查环境 ==="
echo "Rust: $(rustc --version 2>/dev/null || echo '未安装')"
echo "Cargo: $(cargo --version 2>/dev/null || echo '未安装')"
echo "Solana: $(solana --version 2>/dev/null || echo '未安装')"
echo "Anchor: $(anchor --version 2>/dev/null || echo '未安装')"

# 步骤2: 进入项目目录
echo ""
echo "=== 步骤2: 设置项目 ==="
cd "/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"
echo "项目目录: $(pwd)"
ls -la

# 步骤3: 停止现有进程
echo ""
echo "=== 步骤3: 清理环境 ==="
echo "停止现有验证器..."
pkill -f solana-test-validator 2>/dev/null || echo "无验证器运行"
sleep 2

# 步骤4: 启动验证器（单独终端）
echo ""
echo "=== 步骤4: 启动本地测试网络 ==="
echo "请在新终端中运行以下命令:"
echo "----------------------------------------"
echo "cd $(pwd)"
echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
echo "export PATH=\"\$HOME/.local/share/solana/install/active_release/bin:\$PATH\""
echo "solana-test-validator --reset --log"
echo "----------------------------------------"
echo ""
read -p "验证器已启动？(按回车继续) "

# 步骤5: 配置网络
echo ""
echo "=== 步骤5: 配置网络 ==="
solana config set --url http://localhost:8899
solana config get

# 步骤6: 设置钱包
echo ""
echo "=== 步骤6: 设置钱包 ==="
if [ -f "test-wallet.json" ]; then
    echo "使用现有钱包"
else
    echo "创建新钱包"
    solana-keygen new --outfile test-wallet.json --no-passphrase --silent
fi
solana config set --keypair test-wallet.json

# 步骤7: 获取测试SOL
echo ""
echo "=== 步骤7: 获取测试SOL ==="
solana airdrop 5
echo "余额: $(solana balance)"

# 步骤8: 构建智能合约
echo ""
echo "=== 步骤8: 构建智能合约 ==="
echo "清理构建缓存..."
cargo clean 2>/dev/null || true
rm -rf target/ 2>/dev/null || true

echo "开始构建..."
if anchor build; then
    echo "✅ 构建成功"
else
    echo "❌ 构建失败"
    echo "尝试使用cargo构建..."
    cd programs/solana-oracle
    if cargo build-sbf --sbf-out-dir ../../target/deploy; then
        echo "✅ Cargo构建成功"
        cd ../..
    else
        echo "❌ 所有构建方法都失败"
        exit 1
    fi
fi

# 检查构建结果
if [ -f "target/deploy/solana_oracle.so" ]; then
    echo "程序文件: target/deploy/solana_oracle.so"
    echo "文件大小: $(stat -c%s target/deploy/solana_oracle.so) 字节"
else
    echo "❌ 未找到构建的程序文件"
    exit 1
fi

# 步骤9: 部署智能合约
echo ""
echo "=== 步骤9: 部署智能合约 ==="
PROGRAM_ID=$(solana-keygen pubkey target/deploy/solana_oracle-keypair.json 2>/dev/null || echo "未知")
echo "程序公钥: $PROGRAM_ID"

echo "开始部署..."
if anchor deploy; then
    echo "✅ Anchor部署成功"
else
    echo "尝试手动部署..."
    if solana program deploy target/deploy/solana_oracle.so; then
        echo "✅ 手动部署成功"
    else
        echo "❌ 部署失败"
        echo "请检查:"
        echo "1. 验证器是否运行: solana cluster-version"
        echo "2. 余额是否充足: solana balance"
        echo "3. 网络配置: solana config get"
        exit 1
    fi
fi

# 步骤10: 验证部署
echo ""
echo "=== 步骤10: 验证部署 ==="
if solana program show $PROGRAM_ID 2>/dev/null; then
    echo "✅ 部署验证成功"
else
    echo "⚠️  部署验证警告"
fi

# 步骤11: 更新配置
echo ""
echo "=== 步骤11: 更新配置 ==="
if [ -f "Anchor.toml" ]; then
    cp Anchor.toml Anchor.toml.backup
    sed -i "s|solana_oracle = \".*\"|solana_oracle = \"$PROGRAM_ID\"|g" Anchor.toml
    echo "✅ Anchor.toml已更新"
    echo "新程序ID: $PROGRAM_ID"
fi

# 完成
echo ""
echo "🎉 部署完成！"
echo "========================"
echo "程序ID: $PROGRAM_ID"
echo "网络: http://localhost:8899"
echo "钱包: test-wallet.json"
echo "余额: $(solana balance)"
echo ""
echo "🚀 下一步:"
echo "1. 运行测试: anchor test --skip-local-validator"
echo "2. 更新Rust项目配置"
echo "3. 运行集成测试"

# 保存部署信息
cat > deployment-info.txt << EOF
部署完成时间: $(date)
程序ID: $PROGRAM_ID
网络URL: http://localhost:8899
钱包文件: test-wallet.json
钱包地址: $(solana-keygen pubkey test-wallet.json)
构建文件: target/deploy/solana_oracle.so
配置文件: Anchor.toml (已更新)
EOF

echo "✅ 部署信息保存到: deployment-info.txt"
