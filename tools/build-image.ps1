<#
.SYNOPSIS
    Builds NovaOS Apex UEFI bootloader, kernel, and FAT32 EFI disk image.
#>

[CmdletBinding()]
param (
    [switch]$Release
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoPath) {
        $env:PATH = "$cargoPath;$env:PATH"
    }
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   NovaOS Apex Multi-Stage OS Builder     " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Compile Userland Init Daemon (Ring 3)
Write-Host "[1/4] Building NovaOS Apex Init Daemon (novaos-apex-init)..." -ForegroundColor Yellow
$initArgs = @("rustc", "--package", "novaos-apex-init", "--target", "x86_64-unknown-none")
if ($Release) { $initArgs += "--release" }
$initArgs += @("--", "-C", "link-arg=--image-base=0x40000000")
& cargo $initArgs

# 2. Compile UEFI Bootloader
Write-Host "[2/4] Building 64-bit UEFI Loader (BOOTX64.EFI)..." -ForegroundColor Yellow
$uefiArgs = @("build", "--package", "novaos-apex-boot", "--target", "x86_64-unknown-uefi")
if ($Release) { $uefiArgs += "--release" }
& cargo $uefiArgs

# 3. Compile Kernel Core
Write-Host "[3/4] Building NovaOS Apex Kernel (x86_64-unknown-none)..." -ForegroundColor Yellow
$kernelArgs = @("build", "--package", "novaos-apex-kernel", "--target", "x86_64-unknown-none")
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
$loaderSrc = "target\x86_64-unknown-uefi\$targetMode\novaos-apex-boot.efi"
$loaderDst = Join-Path $efiDir "BOOTX64.EFI"

if (Test-Path $loaderSrc) {
    Copy-Item $loaderSrc $loaderDst -Force
    Write-Host "[3/3] Created EFI payload: $loaderDst" -ForegroundColor Green
}

$kernelSrc = "target\x86_64-unknown-none\$targetMode\novaos-apex-kernel"
$kernelDst = Join-Path $buildDir "novaos-apex.elf"
$kernelDstFallback = Join-Path $efiDir "novaos-apex.elf"
if (Test-Path $kernelSrc) {
    Copy-Item $kernelSrc $kernelDst -Force
    Copy-Item $kernelSrc $kernelDstFallback -Force
    Write-Host "[4/4] Created kernel payload: $kernelDst" -ForegroundColor Green
}

$initSrc = "target\x86_64-unknown-none\$targetMode\novaos-apex-init"
$initDst = Join-Path $buildDir "init"
$initDstFallback = Join-Path $efiDir "init"
if (Test-Path $initSrc) {
    Copy-Item $initSrc $initDst -Force
    Copy-Item $initSrc $initDstFallback -Force
    Write-Host "[4/4] Created userland init payload: $initDst" -ForegroundColor Green
}

# 4. Copy boot configuration
$cfgSrc = Join-Path $buildDir "novaos.cfg"
$cfgFallback = Join-Path $efiDir "novaos.cfg"
if (Test-Path $cfgSrc) {
    Copy-Item $cfgSrc $cfgFallback -Force
} else {
    $defaultCfg = @"
timeout=3
default=novaos-apex
kernel=\novaos-apex.elf
cmdline=console=com1 init=/init loglevel=debug
video_mode=1024x768x32
"@
    Set-Content -Path $cfgSrc -Value $defaultCfg
    Set-Content -Path $cfgFallback -Value $defaultCfg
}

Write-Host "`n[SUCCESS] NovaOS Apex image build sequence complete!" -ForegroundColor Green
Write-Host "Launch image with: .\tools\run-qemu.ps1" -ForegroundColor Cyan
