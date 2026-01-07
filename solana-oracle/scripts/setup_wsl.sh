#!/bin/bash

# WSL环境设置脚本
echo "🔧 设置WSL开发环境"

# 检查WSL
if ! grep -q Microsoft /proc/version; then
    echo "❌ 此脚本需要在WSL环境中运行"
    exit 1
fi

echo "✅ 检测到WSL环境: $(uname -a)"

# 更新系统
echo "🔄 更新系统包..."
sudo apt update && sudo apt upgrade -y

# 安装必要工具
echo "📦 安装基础工具..."
sudo apt install -y curl git build-essential pkg-config libssl-dev

# 安装Rust
echo "🦀 安装Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo "✅ Rust安装完成: $(rustc --version)"
else
    echo "✅ Rust已安装: $(rustc --version)"
fi

# 安装Solana
echo "🔗 安装Solana..."
if ! command -v solana &> /dev/null; then
    sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
    echo "✅ Solana安装完成: $(solana --version)"
else
    echo "✅ Solana已安装: $(solana --version)"
fi

# 安装Anchor
echo "⚓ 安装Anchor..."
if ! command -v anchor &> /dev/null; then
    cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
    echo "✅ Anchor安装完成: $(anchor --version)"
else
    echo "✅ Anchor已安装: $(anchor --version)"
fi

# 安装Node.js（如果未安装）
if ! command -v node &> /dev/null; then
    echo "📦 安装Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt install -y nodejs
    echo "✅ Node.js安装完成: $(node --version)"
else
    echo "✅ Node.js已安装: $(node --version)"
fi

# 安装Yarn
if ! command -v yarn &> /dev/null; then
    echo "🧶 安装Yarn..."
    npm install -g yarn
    echo "✅ Yarn安装完成: $(yarn --version)"
else
    echo "✅ Yarn已安装: $(yarn --version)"
fi

# 设置项目目录
echo "📁 设置项目目录..."
PROJECT_PATH="/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"
LOCAL_PROJECT_PATH="$HOME/solana-oracle"

if [ -d "$PROJECT_PATH" ]; then
    echo "✅ 找到Windows项目路径"
    
    # 复制项目到WSL本地（避免权限问题）
    if [ ! -d "$LOCAL_PROJECT_PATH" ]; then
        echo "复制项目到WSL本地..."
        cp -r "$PROJECT_PATH" "$LOCAL_PROJECT_PATH"
    fi
    
    cd "$LOCAL_PROJECT_PATH"
    echo "✅ 项目目录: $(pwd)"
else
    echo "⚠️  未找到Windows项目路径"
    echo "💡 请确保项目在: D:\\AI\\预言机多智能体\\MultiAgentOracle\\solana-oracle"
    exit 1
fi

# 检查项目文件
echo "🔍 检查项目文件..."
if [ -f "Anchor.toml" ] && [ -f "programs/solana-oracle/src/lib.rs" ]; then
    echo "✅ 项目文件完整"
else
    echo "❌ 项目文件不完整"
    exit 1
fi

# 设置环境变量
echo "⚙️  设置环境变量..."
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export PATH="$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 验证环境
echo "🔬 环境验证:"
echo "  Rust: $(rustc --version 2>/dev/null || echo '未安装')"
echo "  Cargo: $(cargo --version 2>/dev/null || echo '未安装')"
echo "  Solana: $(solana --version 2>/dev/null || echo '未安装')"
echo "  Anchor: $(anchor --version 2>/dev/null || echo '未安装')"
echo "  Node.js: $(node --version 2>/dev/null || echo '未安装')"
echo "  Yarn: $(yarn --version 2>/dev/null || echo '未安装')"

echo ""
echo "🎉 WSL环境设置完成！"
echo ""
echo "🚀 下一步:"
echo "  1. 运行部署脚本: ./scripts/deploy_wsl_simple.sh"
echo "  2. 或者手动部署:"
echo "     cd $LOCAL_PROJECT_PATH"
echo "     ./scripts/deploy_wsl_simple.sh"
echo ""
echo "💡 提示:"
echo "  • 如果遇到权限问题，运行: chmod +x scripts/*.sh"
echo "  • 确保WSL有足够的内存和磁盘空间"
echo "  • 部署前关闭其他占用端口的程序"
