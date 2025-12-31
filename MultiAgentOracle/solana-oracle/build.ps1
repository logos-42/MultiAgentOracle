# PowerShell build script for Solana Identity Registry Program

Write-Host "🔨 Building Solana Identity Registry Program..." -ForegroundColor Green

# Set required environment variables
$env:HOME = $env:USERPROFILE
$env:PATH = "$env:PATH;C:\Users\Mechrevo\.cargo\bin"

# Build using anchor
Write-Host "📦 Building with Anchor..." -ForegroundColor Yellow
anchor build

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful!" -ForegroundColor Green
    
    # Show program ID
    $programId = solana address -k target/deploy/solana_oracle-keypair.json
    Write-Host "📝 Program ID: $programId" -ForegroundColor Cyan
    
    # Update Anchor.toml with localnet program ID
    Write-Host "🔄 Updating Anchor.toml..." -ForegroundColor Yellow
    $anchorContent = Get-Content Anchor.toml -Raw
    $updatedContent = $anchorContent -replace 'solana_oracle = ".*"', "solana_oracle = `"$programId`""
    Set-Content Anchor.toml -Value $updatedContent
    
    Write-Host "✅ Configuration updated!" -ForegroundColor Green
} else {
    Write-Host "❌ Build failed" -ForegroundColor Red
    exit 1
}
