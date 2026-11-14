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

& python tools/test_runner.py
exit $LASTEXITCODE
