<#
.SYNOPSIS
    Attaches GDB to QEMU on port 1234 for kernel debugging.
#>

[CmdletBinding()]
param (
    [string]$TargetElf = "target\x86_64-unknown-none\debug\novaos-kernel"
)

Write-Host "Connecting GDB to QEMU at localhost:1234 with symbols from $TargetElf..." -ForegroundColor Cyan

$gdbCommands = @"
target remote localhost:1234
symbol-file $TargetElf
break kernel_entry
continue
"@

$gdbCommands | Out-File -FilePath ".gdbinit" -Encoding ascii
Write-Host "Run: gdb -x .gdbinit" -ForegroundColor Green
