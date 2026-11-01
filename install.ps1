# horizon-mod-toolkit installer
# Run this in PowerShell to install the toolkit

$ErrorActionPreference = "Stop"
$repoUrl = "https://github.com/CGRLabs/horizon-mod-toolkit.git"
$installDir = Join-Path $env:USERPROFILE "horizon-mod-toolkit"

Write-Host "Installing Horizon Mod Toolkit..." -ForegroundColor Green

if (Test-Path $installDir) {
    Write-Host "Removing existing installation..." -ForegroundColor Yellow
    Remove-Item $installDir -Recurse -Force
}

git clone $repoUrl $installDir

if (-not (Test-Path "$installDir/target/release/horizon-mod-toolkit.exe")) {
    Write-Host "Building release binary..." -ForegroundColor Yellow
    cd $installDir
    cargo build --release
}

Write-Host "`nInstallation complete!" -ForegroundColor Green
Write-Host "Binary location: $installDir/target/release/horizon-mod-toolkit.exe" -ForegroundColor Cyan
Write-Host ""
Write-Host "Quick start:" -ForegroundColor Green
Write-Host "  .`"$installDir/target/release/horizon-mod-toolkit.exe`" new my_mod --name ""My Mod"" --id my_mod"
Write-Host "  .`"$installDir/target/release/horizon-mod-toolkit.exe`" build --path my_mod"
Write-Host "  .`"$installDir/target/release/horizon-mod-toolkit.exe`" check my_mod\target\horizon-mod\my_mod.hmod"
Write-Host ""
Write-Host "Add to PATH (optional):"
Write-Host "  $env:PATH = `"$installDir/target/release;$env:PATH`""
EOF
echo "install.ps1 created"