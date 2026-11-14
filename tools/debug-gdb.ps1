<#
.SYNOPSIS
    Starts QEMU in debug mode and connects GDB.
#>

[CmdletBinding()]
param()

$qemuJob = Start-Process powershell -ArgumentList "-File .\tools\run-qemu.ps1 -Debug" -PassThru
Write-Host "[GDB] Waiting for QEMU GDB server on localhost:1234..." -ForegroundColor Yellow
Start-Sleep -Seconds 1

$gdb = Get-Command "gdb" -ErrorAction SilentlyContinue
if (-not $gdb) {
    $gdb = Get-Command "rust-gdb" -ErrorAction SilentlyContinue
}

if ($gdb) {
    & $gdb -ex "target remote localhost:1234" -ex "symbol-file build\novaos-apex.elf"
} else {
    Write-Host "[INFO] Connect GDB with: gdb -ex 'target remote localhost:1234' -ex 'symbol-file build\novaos-apex.elf'" -ForegroundColor Cyan
}

if ($qemuJob -and -not $qemuJob.HasExited) {
    Stop-Process -Id $qemuJob.Id -Force
}
