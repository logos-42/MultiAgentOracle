cd "d:\AI\预言机多智能体\MultiAgentOracle"
$env:RUST_BACKTRACE = "1"
$env:RUST_LOG = "debug"
Write-Host "开始编译 zk_fingerprint_experiment 示例..."
cargo build --example zk_fingerprint_experiment 2>&1 | Tee-Object -FilePath .\compile_error.log
Write-Host "编译完成，退出码: $LASTEXITCODE"
