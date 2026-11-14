<#
.SYNOPSIS
    Runs NovaOS Apex in QEMU with UEFI (OVMF) firmware.
#>

param (
    [switch]$Debug,
    [switch]$Gdb,
    [switch]$NoGui,
    [string]$SerialLog
)

$qemu = Get-Command "qemu-system-x86_64" -ErrorAction SilentlyContinue
if (-not $qemu) {
    $qemuPaths = @(
        "C:\Program Files\qemu\qemu-system-x86_64.exe",
        "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe"
    )
    foreach ($p in $qemuPaths) {
        if (Test-Path $p) {
            $qemu = $p
            break
        }
    }
}

if (-not $qemu) {
    Write-Error "QEMU not found. Install QEMU via 'winget install SoftwareFreedomConservancy.QEMU' or add it to PATH."
    exit 1
}

$ovmfPaths = @(
    "tools\OVMF.fd",
    "C:\Program Files\qemu\share\edk2-x86_64-code.fd",
    "C:\Program Files\qemu\share\OVMF.fd",
    "/usr/share/OVMF/OVMF_CODE.fd"
)

$ovmf = $null
foreach ($p in $ovmfPaths) {
    if (Test-Path $p) {
        $ovmf = $p
        break
    }
}

$qemuArgs = @(
    "-m", "256M",
    "-drive", "format=raw,file=fat:rw:build",
    "-net", "none"
)

if ($ovmf) {
    $qemuArgs += @("-drive", "if=pflash,format=raw,readonly=on,file=$ovmf")
}

if ($Debug -or $Gdb) {
    $qemuArgs += @("-s", "-S")
    Write-Host "[DEBUG] QEMU GDB server listening on localhost:1234 (halted at entry)" -ForegroundColor Yellow
}

if ($NoGui) {
    $qemuArgs += @("-display", "none")
}

if ($SerialLog) {
    $qemuArgs += @("-serial", "file:$SerialLog")
} else {
    $qemuArgs += @("-serial", "stdio")
}

Write-Host "[QEMU] Starting NovaOS Apex: $qemu $($qemuArgs -join ' ')" -ForegroundColor Cyan
& $qemu $qemuArgs
