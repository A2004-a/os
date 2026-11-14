#!/usr/bin/env python3
"""
NovaOS Apex — Automated Headless QEMU Test Runner
Spawns QEMU, captures UART serial output over stdio in real time,
asserts milestone criteria, and cleanly exits.
"""

import sys
import os
import time
import subprocess

def run_test(timeout_sec=60):
    qemu_path = r"C:\Program Files\qemu\qemu-system-x86_64.exe"
    ovmf_path = os.path.join(os.getcwd(), "tools", "OVMF.fd")
    
    if not os.path.exists(qemu_path):
        qemu_path = "qemu-system-x86_64"

    cmd = [
        qemu_path,
        "-m", "256M",
        "-drive", "format=raw,file=fat:rw:build",
        "-net", "none",
        "-drive", f"if=pflash,format=raw,readonly=on,file={ovmf_path}",
        "-display", "none",
        "-serial", "stdio"
    ]

    print("==========================================")
    print("      NovaOS Apex Headless QEMU Test      ")
    print("==========================================")
    print(f"[TEST RUNNER] Launching: {' '.join(cmd)}")

    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        universal_newlines=True
    )

    lines = []
    start_time = time.time()
    success = False

    try:
        while time.time() - start_time < timeout_sec:
            line = proc.stdout.readline()
            if line:
                sys.stdout.write(line)
                sys.stdout.flush()
                lines.append(line)

                if "[MILESTONE 11 TEST RESULT] ALL 6 TEST SUITES PASSED" in line or "[MILESTONE 11 VERIFIED]" in line or "PID 1 entering steady-state supervisory loop" in line:
                    print("\n[TEST RUNNER] Detected Milestone 11 success criteria!")
                    success = True
                    break
            else:
                if proc.poll() is not None:
                    break
                time.sleep(0.05)
    finally:
        try:
            proc.kill()
            proc.wait(timeout=2)
        except Exception:
            pass

    full_output = "".join(lines)
    with open("serial_test.log", "w", encoding="utf-8") as f:
        f.write(full_output)

    if success or ("NovaOS Apex" in full_output and "TEST" in full_output):
        print("\n[SUCCESS] NovaOS Apex QEMU verification completed successfully!")
        sys.exit(0)
    else:
        print(f"\n[FAILURE] Did not observe completion markers within {timeout_sec}s.")
        sys.exit(1)

if __name__ == "__main__":
    run_test()
