#!/bin/bash
# Solana开发环境快速设置脚本
# 根据 INSTALLATION_PATHS.md 中的路径配置

echo "🚀 Solana开发环境快速设置"
echo "================================"

# 检查是否在WSL中
echo "🔍 检查运行环境..."
if grep -q Microsoft /proc/version 2>/dev/null; then
    echo "   ✅ 检测到WSL环境"
else
    echo "   ⚠️  未检测到WSL环境，继续执行..."
fi

# 设置环境变量
echo ""
echo "📝 设置环境变量..."
export RUSTUP_HOME="$HOME/.rustup"
export CARGO_HOME="$HOME/.cargo"
export SOLANA_HOME="$HOME/.local/share/solana/install/active_release"

# 添加到PATH
export PATH="$CARGO_HOME/bin:$PATH"
export PATH="$SOLANA_HOME/bin:$PATH"

echo "   RUSTUP_HOME: $RUSTUP_HOME"
echo "   CARGO_HOME: $CARGO_HOME"
echo "   SOLANA_HOME: $SOLANA_HOME"
echo "   PATH已更新"

# 验证安装
echo ""
echo "🔍 验证工具安装..."
check_tool() {
    local tool=$1
    local name=$2
    if command -v $tool &> /dev/null; then
        local version=$($tool --version 2>/dev/null | head -1)
        echo "   ✅ $name: $version"
        return 0
    else
        echo "   ❌ $name: 未找到"
        return 1
    fi
}

check_tool rustc "Rust"
check_tool cargo "Cargo"
check_tool solana "Solana"
check_tool anchor "Anchor"

# 检查项目目录
echo ""
echo "📁 检查项目目录..."
PROJECT_DIR="/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"
if [ -d "$PROJECT_DIR" ]; then
    echo "   ✅ 项目目录可访问: $PROJECT_DIR"
    cd "$PROJECT_DIR"
    echo "   当前目录: $(pwd)"
else
    echo "   ⚠️  项目目录不可访问: $PROJECT_DIR"
    echo "   请确保Windows文件系统已正确挂载"
fi

# 创建别名
echo ""
echo "⚡ 创建实用别名..."
cat > /tmp/solana_aliases.sh << 'EOF'
# Solana开发环境别名
alias solana-env='echo "Rust: $(rustc --version 2>/dev/null) | Solana: $(solana --version 2>/dev/null) | Anchor: $(anchor --version 2>/dev/null)"'
alias solana-test='solana-test-validator --reset --quiet & echo "测试网已启动 (PID: $!)"'
alias solana-status='solana cluster-version && solana config get && solana balance'
alias solana-build='anchor build'
alias solana-deploy='anchor deploy'
alias solana-clean='cargo clean && rm -rf target/'
alias solana-logs='tail -f test-ledger/validator.log 2>/dev/null || echo "日志文件不存在"'
EOF

echo "   别名已创建到 /tmp/solana_aliases.sh"
echo "   使用: source /tmp/solana_aliases.sh"

# 生成快速参考
echo ""
echo "📋 快速参考:"
echo "   1. 启动测试网: solana-test-validator --reset --quiet &"
echo "   2. 构建项目: anchor build"
echo "   3. 部署合约: anchor deploy"
echo "   4. 检查状态: solana cluster-version"
echo "   5. 请求空投: solana airdrop 100"
echo "   6. 运行测试: node test_simple.js"

# 永久配置建议
echo ""
echo "💾 永久配置建议:"
echo "   将以下内容添加到 ~/.bashrc:"
echo ""
echo "   # Solana开发环境"
echo "   export PATH=\"\$HOME/.cargo/bin:\$PATH\""
echo "   export PATH=\"\$HOME/.local/share/solana/install/active_release/bin:\$PATH\""
echo "   alias solana-env='echo \"Rust: \$(rustc --version 2>/dev/null) | Solana: \$(solana --version 2>/dev/null) | Anchor: \$(anchor --version 2>/dev/null)\"'"

echo ""
echo "✅ 环境设置完成!"
echo "💡 详细路径信息请查看: INSTALLATION_PATHS.md"
echo "🚀 开始开发: cd $PROJECT_DIR && source /tmp/solana_aliases.sh"
