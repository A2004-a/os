<#
.SYNOPSIS
    Installs Rust nightly toolchain, bare-metal targets, bootimage utility, and QEMU.
.DESCRIPTION
    Automates toolchain setup on Windows 10/11 for NovaOS bare-metal development.
#>

[CmdletBinding()]
param ()

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   NovaOS Bare-Metal Toolchain Installer  " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Install Rustup if not found
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Write-Host "[1/4] Installing Rustup via winget..." -ForegroundColor Yellow
    winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")
} else {
    Write-Host "[1/4] Rustup is already installed." -ForegroundColor Green
}

# 2. Configure Nightly Channel and Bare-Metal Components
Write-Host "[2/4] Configuring Rust nightly channel & bare-metal components..." -ForegroundColor Yellow
rustup default nightly
rustup component add rust-src llvm-tools-preview

# 3. Install QEMU if not found
if (-not (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue)) {
    Write-Host "[3/4] Installing QEMU via winget..." -ForegroundColor Yellow
    winget install --id SoftwareFreedomConservancy.QEMU -e --accept-source-agreements --accept-package-agreements
    $qemuPath = "C:\Program Files\qemu"
    if (Test-Path $qemuPath) {
        $env:Path += ";$qemuPath"
        [System.Environment]::SetEnvironmentVariable("Path", $env:Path, "User")
    }
} else {
    Write-Host "[3/4] QEMU is already installed." -ForegroundColor Green
}

# 4. Install bootimage utility
Write-Host "[4/4] Installing cargo bootimage utility..." -ForegroundColor Yellow
cargo install bootimage

Write-Host "`n[SUCCESS] NovaOS toolchain configuration complete!" -ForegroundColor Green
Write-Host "You can now run: .\scripts\build.ps1 or .\scripts\run.ps1" -ForegroundColor Cyan
