# 验证修复脚本
# 这个脚本检查 zk_fingerprint_experiment 示例是否可以成功编译

echo "=== 验证 zk_fingerprint_experiment 编译 ==="
cargo check --example zk_fingerprint_experiment

$exitCode = $LASTEXITCODE

if ($exitCode -eq 0) {
    echo ""
    echo "✅ 编译检查通过！"
    echo ""
    echo "=== 构建示例 ==="
    cargo build --example zk_fingerprint_experiment
    
    $buildExitCode = $LASTEXITCODE
    if ($buildExitCode -eq 0) {
        echo ""
        echo "✅ 构建成功！"
    } else {
        echo ""
        echo "❌ 构建失败，退出码: $buildExitCode"
    }
} else {
    echo ""
    echo "❌ 编译检查失败，退出码: $exitCode"
}

echo ""
echo "=== 验证完成 ==="
