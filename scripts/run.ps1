<#
.SYNOPSIS
    Runs NovaOS bare-metal kernel inside QEMU with hardware emulation.
#>

[CmdletBinding()]
param (
    [switch]$Release,
    [switch]$NoGui,
    [string]$Memory = "512M"
)

$ErrorActionPreference = "Stop"

$targetDir = if ($Release) { "release" } else { "debug" }
$binPath = "target\x86_64-novaos\$targetDir\bootimage-novaos-kernel.bin"

if (-not (Test-Path $binPath)) {
    Write-Host "Kernel binary not found. Building first..." -ForegroundColor Yellow
    & .\scripts\build.ps1
}

# Resolve QEMU binary
$qemuExe = "qemu-system-x86_64"
if (-not (Get-Command $qemuExe -ErrorAction SilentlyContinue)) {
    if (Test-Path "C:\Program Files\qemu\qemu-system-x86_64.exe") {
        $qemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
    } else {
        Write-Error "QEMU not found. Please install QEMU or run .\scripts\install-toolchain.ps1"
    }
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "       Launching NovaOS in QEMU            " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Boot Disk: $binPath" -ForegroundColor DarkGray
Write-Host "Memory:    $Memory" -ForegroundColor DarkGray
Write-Host "Serial:    stdio (mirroring COM1 kernel logs)" -ForegroundColor DarkGray
Write-Host "Exit:      Close QEMU window or press Ctrl+C" -ForegroundColor Yellow

$qemuArgs = @(
    "-drive", "format=raw,file=$binPath",
    "-serial", "stdio",
    "-m", $Memory,
    "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"
)

if ($NoGui) {
    $qemuArgs += @("-display", "none")
}

& $qemuExe $qemuArgs
