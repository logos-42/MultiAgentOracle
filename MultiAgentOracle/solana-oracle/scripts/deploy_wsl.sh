#!/bin/bash

# WSL智能合约部署脚本
echo "🚀 开始在WSL中部署智能合约到本地测试网"

# 检查是否在WSL中
if ! grep -q Microsoft /proc/version; then
    echo "❌ 此脚本需要在WSL环境中运行"
    exit 1
fi

echo "✅ 检测到WSL环境: $(uname -a)"

# 设置颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 函数：打印带颜色的消息
print_info() {
    echo -e "${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 步骤1：检查工具
print_info "步骤1: 检查必要的工具"

check_tool() {
    if command -v $1 &> /dev/null; then
        print_success "$1 已安装: $($1 --version 2>/dev/null | head -n1)"
        return 0
    else
        print_error "$1 未安装"
        return 1
    fi
}

# 检查工具
check_tool "anchor" || {
    print_warning "Anchor未安装，尝试安装..."
    # 这里可以添加Anchor安装命令
    exit 1
}

check_tool "solana" || {
    print_warning "Solana CLI未安装，尝试安装..."
    # 这里可以添加Solana安装命令
    exit 1
}

check_tool "cargo" || {
    print_warning "Rust未安装，尝试安装..."
    # 这里可以添加Rust安装命令
    exit 1
}

# 步骤2：设置项目目录
print_info "步骤2: 设置项目目录"

# 假设项目在Windows文件系统中，需要映射到WSL
WINDOWS_PROJECT_PATH="/mnt/d/AI/预言机多智能体/MultiAgentOracle/solana-oracle"
LOCAL_PROJECT_PATH="$HOME/multi-agent-oracle"

if [ -d "$WINDOWS_PROJECT_PATH" ]; then
    print_info "找到Windows项目路径: $WINDOWS_PROJECT_PATH"
    
    # 复制项目到WSL本地（避免权限问题）
    if [ ! -d "$LOCAL_PROJECT_PATH" ]; then
        print_info "复制项目到WSL本地: $LOCAL_PROJECT_PATH"
        cp -r "$WINDOWS_PROJECT_PATH" "$LOCAL_PROJECT_PATH"
    fi
    
    cd "$LOCAL_PROJECT_PATH"
else
    print_warning "未找到Windows项目路径，使用当前目录"
    LOCAL_PROJECT_PATH="."
    cd "$LOCAL_PROJECT_PATH"
fi

print_success "项目目录: $(pwd)"

# 步骤3：启动本地测试网络
print_info "步骤3: 启动本地测试网络"

# 停止可能正在运行的测试验证器
print_info "停止现有测试验证器..."
pkill -f solana-test-validator 2>/dev/null || true
sleep 2

# 启动新的测试验证器
print_info "启动本地测试验证器..."
solana-test-validator \
    --reset \
    --quiet \
    --bpf-program DPZTkPxJcXZ3tHxqYrTkw6shLoR73pywLDJX82wXAZ7b target/deploy/solana_oracle.so \
    > /dev/null 2>&1 &
VALIDATOR_PID=$!

# 等待验证器启动
print_info "等待验证器启动..."
sleep 5

# 检查验证器是否运行
if ps -p $VALIDATOR_PID > /dev/null; then
    print_success "本地测试验证器已启动 (PID: $VALIDATOR_PID)"
else
    print_error "无法启动本地测试验证器"
    exit 1
fi

# 设置本地网络配置
print_info "配置本地网络..."
solana config set --url http://localhost:8899

# 创建测试钱包
print_info "创建测试钱包..."
if [ ! -f "test-wallet.json" ]; then
    solana-keygen new --outfile test-wallet.json --no-passphrase --silent
    print_success "创建新的测试钱包"
else
    print_success "使用现有测试钱包"
fi

solana config set --keypair test-wallet.json

# 获取测试SOL
print_info "获取测试SOL..."
solana airdrop 10

# 检查余额
BALANCE=$(solana balance)
print_success "当前余额: $BALANCE"

# 步骤4：构建智能合约
print_info "步骤4: 构建智能合约"

# 清理之前的构建
print_info "清理构建缓存..."
rm -rf target/deploy/* 2>/dev/null || true

# 构建智能合约
print_info "构建智能合约..."
if anchor build; then
    print_success "智能合约构建成功"
else
    print_error "智能合约构建失败"
    # 尝试使用cargo直接构建
    print_info "尝试使用cargo构建..."
    cd programs/solana-oracle
    if cargo build-sbf --sbf-out-dir ../../target/deploy; then
        print_success "使用cargo构建成功"
        cd ../..
    else
        print_error "所有构建方法都失败"
        kill $VALIDATOR_PID 2>/dev/null || true
        exit 1
    fi
fi

# 检查构建结果
if [ -f "target/deploy/solana_oracle.so" ]; then
    FILESIZE=$(stat -c%s "target/deploy/solana_oracle.so")
    print_success "程序文件: target/deploy/solana_oracle.so ($((FILESIZE/1024)) KB)"
else
    print_error "未找到构建的程序文件"
    kill $VALIDATOR_PID 2>/dev/null || true
    exit 1
fi

# 步骤5：部署智能合约
print_info "步骤5: 部署智能合约到本地网络"

# 获取程序ID
if [ -f "target/deploy/solana_oracle-keypair.json" ]; then
    PROGRAM_ID=$(solana-keygen pubkey target/deploy/solana_oracle-keypair.json)
    print_info "程序公钥: $PROGRAM_ID"
else
    print_error "未找到程序密钥对"
    kill $VALIDATOR_PID 2>/dev/null || true
    exit 1
fi

# 部署程序
print_info "部署智能合约..."
if anchor deploy; then
    print_success "智能合约部署成功"
else
    print_error "智能合约部署失败"
    print_info "尝试手动部署..."
    if solana program deploy target/deploy/solana_oracle.so; then
        print_success "手动部署成功"
    else
        print_error "所有部署方法都失败"
        kill $VALIDATOR_PID 2>/dev/null || true
        exit 1
    fi
fi

# 更新配置文件
print_info "更新配置文件..."
if [ -f "Anchor.toml" ]; then
    # 备份原文件
    cp Anchor.toml Anchor.toml.backup
    
    # 更新程序ID
    sed -i "s|solana_oracle = \".*\"|solana_oracle = \"$PROGRAM_ID\"|g" Anchor.toml
    print_success "Anchor.toml已更新"
fi

# 步骤6：验证部署
print_info "步骤6: 验证部署"

print_info "检查程序账户..."
if solana program show $PROGRAM_ID; then
    print_success "程序账户验证成功"
else
    print_error "程序账户验证失败"
fi

# 步骤7：运行测试
print_info "步骤7: 运行智能合约测试"

print_info "运行Anchor测试..."
if anchor test --skip-local-validator; then
    print_success "智能合约测试通过"
else
    print_warning "智能合约测试失败，但部署已完成"
fi

# 步骤8：清理和总结
print_info "步骤8: 部署完成"

# 显示部署信息
echo ""
echo -e "${GREEN}🎉 智能合约部署完成！${NC}"
echo "=========================================="
echo -e "${BLUE}部署信息:${NC}"
echo "  网络: http://localhost:8899"
echo "  程序ID: $PROGRAM_ID"
echo "  钱包: test-wallet.json"
echo "  余额: $BALANCE"
echo "  验证器PID: $VALIDATOR_PID"
echo ""
echo -e "${YELLOW}🚀 下一步:${NC}"
echo "  1. 更新Rust项目中的程序ID: $PROGRAM_ID"
echo "  2. 运行集成测试"
echo "  3. 使用测试客户端验证功能"
echo ""
echo -e "${YELLOW}⚠️  注意事项:${NC}"
echo "  • 本地验证器正在后台运行 (PID: $VALIDATOR_PID)"
echo "  • 停止验证器: kill $VALIDATOR_PID"
echo "  • 重新启动: 运行此脚本即可"

# 保存部署信息
cat > deploy-info.txt << EOF
部署时间: $(date)
程序ID: $PROGRAM_ID
网络: http://localhost:8899
钱包: test-wallet.json
验证器PID: $VALIDATOR_PID
EOF

print_success "部署信息已保存到: deploy-info.txt"

# 保持脚本运行，不退出验证器
print_info "本地测试网络正在运行..."
print_info "按 Ctrl+C 停止测试网络并退出"

# 等待用户中断
trap "print_info '停止测试验证器...'; kill $VALIDATOR_PID 2>/dev/null; print_success '部署完成！'; exit 0" INT

while true; do
    sleep 10
done
