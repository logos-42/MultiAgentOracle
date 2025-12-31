# 永久设置Solana环境变量
# 将此脚本添加到PowerShell Profile或每次运行前执行

# 设置HOME环境变量
$env:HOME = $env:USERPROFILE

# 添加Solana到PATH
$solanaPath = "C:\Users\$env:USERNAME\.local\share\solana\install\active_release\bin"
if (Test-Path $solanaPath) {
    if ($env:PATH -notlike "*$solanaPath*") {
        $env:PATH = "$solanaPath;$env:PATH"
    }
}

Write-Host "✅ Solana环境变量已设置" -ForegroundColor Green
