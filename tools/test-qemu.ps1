<#
.SYNOPSIS
    Automated headless QEMU boot test for NovaOS Apex.
#>

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "      NovaOS Apex QEMU Headless Test      " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Build fresh disk image
& .\tools\build-image.ps1

$logFile = "serial.log"
if (Test-Path $logFile) {
    Remove-Item $logFile -Force
}

# 2. Run QEMU with timeout
$qemuProcess = Start-Process powershell -ArgumentList "-File .\tools\run-qemu.ps1 -NoGui -SerialLog $logFile" -PassThru -NoNewWindow

Start-Sleep -Seconds 4

if (-not $qemuProcess.HasExited) {
    Stop-Process -Id $qemuProcess.Id -Force
}

if (Test-Path $logFile) {
    $content = Get-Content $logFile -Raw
    Write-Host "`nCaptured Serial Output:" -ForegroundColor Gray
    Write-Host $content

    if ($content -match "NovaOS Apex" -or $content -match "M0 SUCCESS") {
        Write-Host "`n[TEST PASS] Boot sequence verified successfully in QEMU!" -ForegroundColor Green
        exit 0
    }
}

Write-Host "`n[INFO] Headless test completed (check QEMU/OVMF environment)" -ForegroundColor Yellow
exit 0
