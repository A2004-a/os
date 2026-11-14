<#
.SYNOPSIS
    Builds NovaOS Zero UEFI bootloader, kernel, and FAT32 EFI disk image.
#>

[CmdletBinding()]
param (
    [switch]$Release
)

$ErrorActionPreference = "Stop"

# Auto-locate cargo in default Rustup user directory if not in PATH
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoPath) {
        $env:PATH = "$cargoPath;$env:PATH"
    }
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   NovaOS Zero Multi-Stage OS Builder     " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Compile UEFI Bootloader
Write-Host "[1/3] Building 64-bit UEFI Loader (BOOTX64.EFI)..." -ForegroundColor Yellow
$uefiArgs = @("build", "--package", "novaos-uefi-boot", "--target", "x86_64-unknown-uefi")
if ($Release) { $uefiArgs += "--release" }
& cargo $uefiArgs

# 2. Compile Kernel Core
Write-Host "[2/4] Building NovaOS Kernel (x86_64-unknown-none)..." -ForegroundColor Yellow
$kernelArgs = @("build", "--package", "novaos-kernel", "--target", "x86_64-unknown-none")
if ($Release) { $kernelArgs += "--release" }
& cargo $kernelArgs

# 3. Compile Userland Init
Write-Host "[3/5] Building NovaOS Userland Init (novaos-init)..." -ForegroundColor Yellow
$initArgs = @("build", "--package", "novaos-init", "--target", "x86_64-unknown-none")
if ($Release) { $initArgs += "--release" }
& cargo $initArgs

# 4. Compile Userland Shell
Write-Host "[4/6] Building NovaOS Userland Shell (novaos-shell)..." -ForegroundColor Yellow
$shellArgs = @("build", "--package", "novaos-shell", "--target", "x86_64-unknown-none")
if ($Release) { $shellArgs += "--release" }
& cargo $shellArgs

# 5. Compile Userland Native Apps
Write-Host "[5/6] Building NovaOS Userland Native Apps (novaos-apps)..." -ForegroundColor Yellow
$appsArgs = @("build", "--package", "novaos-apps", "--target", "x86_64-unknown-none")
if ($Release) { $appsArgs += "--release" }
& cargo $appsArgs

# 6. Create Output Boot Directory
$buildDir = "build"
if (-not (Test-Path $buildDir)) {
    New-Item -ItemType Directory -Path $buildDir | Out-Null
}

$efiDir = Join-Path $buildDir "EFI\BOOT"
if (-not (Test-Path $efiDir)) {
    New-Item -ItemType Directory -Path $efiDir -Force | Out-Null
}

$targetMode = if ($Release) { "release" } else { "debug" }
$loaderCandidates = @(
    "target\x86_64-unknown-uefi\$targetMode\novaos-uefi-boot.efi",
    "target\x86_64-unknown-uefi\$targetMode\uefi-loader.efi"
)
$loaderDst = Join-Path $efiDir "BOOTX64.EFI"

foreach ($src in $loaderCandidates) {
    if (Test-Path $src) {
        Copy-Item $src $loaderDst -Force
        Write-Host "[6/6] Created EFI payload: $loaderDst" -ForegroundColor Green
        break
    }
}

# Copy kernel ELF to root of boot partition
$kernelSrc = "target\x86_64-unknown-none\$targetMode\novaos-kernel"
$kernelDst = Join-Path $buildDir "novaos.elf"
if (Test-Path $kernelSrc) {
    Copy-Item $kernelSrc $kernelDst -Force
    Write-Host "[6/6] Created kernel payload: $kernelDst" -ForegroundColor Green
}

# Copy userland init ELF to root of boot partition
$initSrc = "target\x86_64-unknown-none\$targetMode\novaos-init"
$initDst = Join-Path $buildDir "init.elf"
if (Test-Path $initSrc) {
    Copy-Item $initSrc $initDst -Force
    Write-Host "[6/6] Created userland init payload: $initDst" -ForegroundColor Green
}

# Copy userland shell ELF to root of boot partition
$shellSrc = "target\x86_64-unknown-none\$targetMode\novaos-shell"
$shellDst = Join-Path $buildDir "shell.elf"
if (Test-Path $shellSrc) {
    Copy-Item $shellSrc $shellDst -Force
    Write-Host "[6/6] Created userland shell payload: $shellDst" -ForegroundColor Green
}

# Copy userland apps ELF to root of boot partition
$appsSrc = "target\x86_64-unknown-none\$targetMode\novaos-apps"
$appsDst = Join-Path $buildDir "apps.elf"
if (Test-Path $appsSrc) {
    Copy-Item $appsSrc $appsDst -Force
    Write-Host "[6/6] Created userland apps payload: $appsDst" -ForegroundColor Green
}

# Copy boot configuration
if (Test-Path "novaos.cfg") {
    Copy-Item "novaos.cfg" (Join-Path $buildDir "novaos.cfg") -Force
    Write-Host "[6/6] Copied boot configuration: $buildDir\novaos.cfg" -ForegroundColor Green
}

Write-Host "`n[SUCCESS] NovaOS Zero build sequence complete!" -ForegroundColor Green
Write-Host "Launch image with: .\tools\run-qemu.ps1" -ForegroundColor Cyan
