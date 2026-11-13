<#
.SYNOPSIS
    Builds the NovaOS x86_64 bare-metal kernel and bootable disk image.
#>

[CmdletBinding()]
param (
    [switch]$Release
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "       Building NovaOS Bare-Metal Kernel  " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Check for cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Cargo is not found. Please run .\scripts\install-toolchain.ps1 first."
}

# Check for bootimage
if (-not (Get-Command bootimage -ErrorAction SilentlyContinue)) {
    Write-Host "Bootimage tool not detected; installing..." -ForegroundColor Yellow
    cargo install bootimage
}

$buildArgs = @("bootimage")
if ($Release) {
    $buildArgs += "--release"
}

Write-Host "Compiling kernel binary and linking UEFI bootloader..." -ForegroundColor Yellow
& cargo $buildArgs

$targetDir = if ($Release) { "release" } else { "debug" }
$binPath = "target\x86_64-novaos\$targetDir\bootimage-novaos-kernel.bin"

if (Test-Path $binPath) {
    $fileInfo = Get-Item $binPath
    Write-Host "`n[SUCCESS] Bootable disk image generated successfully!" -ForegroundColor Green
    Write-Host "Artifact: $binPath ($([math]::Round($fileInfo.Length / 1MB, 2)) MB)" -ForegroundColor Cyan
} else {
    Write-Error "Build finished but bootable artifact was not found at $binPath"
}
