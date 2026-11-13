<#
.SYNOPSIS
    Runs NovaOS with QEMU using OVMF UEFI firmware.
#>

[CmdletBinding()]
param (
    [switch]$NoGui,
    [switch]$Gdb,
    [string]$Memory = "512M",
    [string]$OvmfPath = ""
)

$ErrorActionPreference = "Stop"

# Detect QEMU binary
$qemuExe = "qemu-system-x86_64"
if (-not (Get-Command $qemuExe -ErrorAction SilentlyContinue)) {
    if (Test-Path "C:\Program Files\qemu\qemu-system-x86_64.exe") {
        $qemuExe = "C:\Program Files\qemu\qemu-system-x86_64.exe"
    } else {
        Write-Error "QEMU not found. Install QEMU via 'winget install SoftwareFreedomConservancy.QEMU'"
    }
}

# Resolve OVMF firmware
if (-not $OvmfPath -or -not (Test-Path $OvmfPath)) {
    $commonOvmf = @(
        "C:\Program Files\qemu\share\edk2-x86_64-code.fd",
        "C:\Program Files\qemu\share\bios-256k.bin",
        "OVMF.fd"
    )
    foreach ($cand in $commonOvmf) {
        if (Test-Path $cand) {
            $OvmfPath = $cand
            break
        }
    }
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "         Launching NovaOS in QEMU         " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Firmware:  $OvmfPath" -ForegroundColor DarkGray
Write-Host "Memory:    $Memory" -ForegroundColor DarkGray
Write-Host "Serial:    stdio (mirroring COM1 kernel logs)" -ForegroundColor DarkGray

$qemuArgs = @(
    "-m", $Memory,
    "-serial", "stdio",
    "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"
)

if ($OvmfPath -and (Test-Path $OvmfPath)) {
    $qemuArgs += @("-bios", $OvmfPath)
}

if (Test-Path "build") {
    $qemuArgs += @("-drive", "format=raw,file=fat:rw:build")
}

if ($NoGui) {
    $qemuArgs += @("-display", "none")
}

if ($Gdb) {
    Write-Host "[GDB] Waiting for GDB connection on localhost:1234..." -ForegroundColor Yellow
    $qemuArgs += @("-s", "-S")
}

& $qemuExe $qemuArgs
