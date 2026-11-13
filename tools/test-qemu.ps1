<#
.SYNOPSIS
    Runs NovaOS Zero headless verification test in QEMU.
#>

[CmdletBinding()]
param ()

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "      NovaOS Zero QEMU Headless Test      " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Build Image
& .\tools\build-image.ps1

# 2. Run Headless
& .\tools\run-qemu.ps1 -NoGui
