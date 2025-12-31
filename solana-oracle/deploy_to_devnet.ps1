# PowerShell script to deploy Solana identity registry to devnet

Write-Host "🚀 Starting Solana Identity Registry Deployment to Devnet..." -ForegroundColor Green
Write-Host "=".repeat(60)

# Set environment variables
$env:HOME = $env:USERPROFILE
Write-Host "✅ Set HOME environment variable: $env:HOME" -ForegroundColor Yellow

# Step 1: Check connection
Write-Host "`n📡 Step 1: Checking network connection..." -ForegroundColor Cyan
$balance = solana balance
Write-Host "   Current balance: $balance" -ForegroundColor Yellow

if ($balance -eq "0 SOL") {
    Write-Host "⚠️  Balance is 0, requesting airdrop..." -ForegroundColor Red
    solana airdrop 0.5
    Start-Sleep -Seconds 5
    $balance = solana balance
    Write-Host "   New balance: $balance" -ForegroundColor Yellow
}

# Step 2: Build the program
Write-Host "`n🔨 Step 2: Building Solana program..." -ForegroundColor Cyan
try {
    anchor build
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Build successful!" -ForegroundColor Green
    } else {
        Write-Host "❌ Build failed" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Build error: $_" -ForegroundColor Red
    exit 1
}

# Step 3: Get program ID
Write-Host "`n📝 Step 3: Getting program ID..." -ForegroundColor Cyan
$programId = solana address -k target/deploy/solana_oracle-keypair.json
Write-Host "   Program ID: $programId" -ForegroundColor Yellow

# Step 4: Update program ID in source
Write-Host "`n🔄 Step 4: Updating program ID in source..." -ForegroundColor Cyan
$libRsPath = "programs\solana-oracle\src\lib.rs"
$content = Get-Content $libRsPath -Raw
$updatedContent = $content -replace 'declare_id\(".*"\)', "declare_id(`"$programId`")"
Set-Content $libRsPath -Value $updatedContent
Write-Host "✅ Updated program ID in $libRsPath" -ForegroundColor Green

# Step 5: Rebuild with updated program ID
Write-Host "`n🔨 Step 5: Rebuilding with updated program ID..." -ForegroundColor Cyan
anchor build
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Rebuild successful!" -ForegroundColor Green
} else {
    Write-Host "❌ Rebuild failed" -ForegroundColor Red
    exit 1
}

# Step 6: Deploy to devnet
Write-Host "`n🌐 Step 6: Deploying to devnet..." -ForegroundColor Cyan
Write-Host "   This may take a few minutes..." -ForegroundColor Yellow

try {
    $deployOutput = anchor deploy --provider.cluster devnet 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Deployment successful!" -ForegroundColor Green
        Write-Host "   Program deployed at: $programId" -ForegroundColor Cyan
        Write-Host "   Explorer: https://explorer.solana.com/address/$programId?cluster=devnet" -ForegroundColor Blue
    } else {
        Write-Host "❌ Deployment failed" -ForegroundColor Red
        Write-Host "   Error output:" -ForegroundColor Red
        Write-Host $deployOutput -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Deployment error: $_" -ForegroundColor Red
}

# Step 7: Update Anchor.toml
Write-Host "`n📋 Step 7: Updating Anchor.toml..." -ForegroundColor Cyan
$anchorTomlPath = "Anchor.toml"
$anchorContent = Get-Content $anchorTomlPath -Raw
$updatedAnchorContent = $anchorContent -replace 'solana_oracle = ".*"', "solana_oracle = `"$programId`""
Set-Content $anchorTomlPath -Value $updatedAnchorContent
Write-Host "✅ Updated $anchorTomlPath with new program ID" -ForegroundColor Green

# Step 8: Verify deployment
Write-Host "`n🔍 Step 8: Verifying deployment..." -ForegroundColor Cyan
try {
    $programInfo = solana program show $programId
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Program verified on devnet!" -ForegroundColor Green
        Write-Host $programInfo -ForegroundColor Yellow
    } else {
        Write-Host "⚠️  Program not found on devnet" -ForegroundColor Red
    }
} catch {
    Write-Host "⚠️  Verification failed: $_" -ForegroundColor Red
}

Write-Host "`n" + "=".repeat(60)
Write-Host "🎉 Deployment process completed!" -ForegroundColor Green
Write-Host "`n📋 Next steps:" -ForegroundColor Cyan
Write-Host "   1. Update Rust client configuration with new program ID" -ForegroundColor Yellow
Write-Host "   2. Test the deployed program with the demo" -ForegroundColor Yellow
Write-Host "   3. Integrate with multi-agent oracle system" -ForegroundColor Yellow
