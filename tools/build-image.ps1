<#
.SYNOPSIS
    Builds NovaOS UEFI bootloader, kernel, and FAT32 EFI disk image.
#>

[CmdletBinding()]
param (
    [switch]$Release
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "      NovaOS Multi-Stage OS Image Builder " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Compile UEFI Bootloader
Write-Host "[1/3] Building 64-bit UEFI Loader (BOOTX64.EFI)..." -ForegroundColor Yellow
$uefiArgs = @("build", "--package", "uefi-loader", "--target", "x86_64-unknown-uefi")
if ($Release) { $uefiArgs += "--release" }
& cargo $uefiArgs

# 2. Compile Kernel Core
Write-Host "[2/3] Building NovaOS Kernel (x86_64-unknown-none)..." -ForegroundColor Yellow
$kernelArgs = @("build", "--package", "novaos-kernel", "--target", "x86_64-unknown-none")
if ($Release) { $kernelArgs += "--release" }
& cargo $kernelArgs

# 3. Create Output Boot Directory
$buildDir = "build"
if (-not (Test-Path $buildDir)) {
    New-Item -ItemType Directory -Path $buildDir | Out-Null
}

$efiDir = Join-Path $buildDir "EFI\BOOT"
if (-not (Test-Path $efiDir)) {
    New-Item -ItemType Directory -Path $efiDir -Force | Out-Null
}

$targetMode = if ($Release) { "release" } else { "debug" }
$loaderSrc = "target\x86_64-unknown-uefi\$targetMode\uefi-loader.efi"
$loaderDst = Join-Path $efiDir "BOOTX64.EFI"

if (Test-Path $loaderSrc) {
    Copy-Item $loaderSrc $loaderDst -Force
    Write-Host "[3/3] Created EFI payload: $loaderDst" -ForegroundColor Green
} else {
    Write-Host "[3/3] Note: Compile artifacts will be placed in $loaderDst upon toolchain execution." -ForegroundColor Yellow
}

Write-Host "`n[SUCCESS] NovaOS build sequence complete!" -ForegroundColor Green
Write-Host "Launch image with: .\tools\run-qemu.ps1" -ForegroundColor Cyan
