#!/usr/bin/env python3
"""
NovaOS Apex — Desktop Screen Capture Engine
Boots NovaOS Apex in QEMU, waits for steady-state desktop composition,
takes a screendump via QEMU monitor, and converts to PNG.
"""

import sys
import os
import time
import subprocess
from PIL import Image

def capture(output_png="novaos_desktop.png", timeout_sec=60):
    qemu_path = r"C:\Program Files\qemu\qemu-system-x86_64.exe"
    ovmf_path = os.path.join(os.getcwd(), "tools", "OVMF.fd")
    
    if not os.path.exists(qemu_path):
        qemu_path = "qemu-system-x86_64"

    serial_log = "serial_capture.log"
    ppm_file = "screenshot.ppm"

    if os.path.exists(serial_log):
        try: os.remove(serial_log)
        except: pass
    if os.path.exists(ppm_file):
        try: os.remove(ppm_file)
        except: pass

    cmd = [
        qemu_path,
        "-m", "256M",
        "-drive", "format=raw,file=fat:rw:build",
        "-net", "none",
        "-drive", f"if=pflash,format=raw,readonly=on,file={ovmf_path}",
        "-serial", f"file:{serial_log}",
        "-monitor", "stdio",
        "-display", "none"
    ]

    print(f"[CAPTURE] Launching QEMU to capture desktop: {' '.join(cmd)}")
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )

    start_time = time.time()
    booted = False

    while time.time() - start_time < timeout_sec:
        if os.path.exists(serial_log):
            try:
                with open(serial_log, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
                    if "PID 1 entering steady-state supervisory loop" in content or "NovaOS Apex" in content and "steady-state" in content:
                        print("[CAPTURE] Detected steady-state supervisory loop & desktop composition!")
                        booted = True
                        break
            except Exception:
                pass
        time.sleep(0.5)

    if not booted:
        print("[CAPTURE] Timeout waiting for desktop boot marker.")

    # Give the compositor time to finalize all window blits
    time.sleep(1.5)

    print("[CAPTURE] Issuing screendump to QEMU monitor...")
    try:
        proc.stdin.write(f"screendump {ppm_file}\n")
        proc.stdin.flush()
        time.sleep(1.0)
        proc.stdin.write("quit\n")
        proc.stdin.flush()
    except Exception as e:
        print(f"[CAPTURE] Error sending monitor command: {e}")

    try:
        proc.wait(timeout=3)
    except:
        proc.kill()

    if os.path.exists(ppm_file):
        print(f"[CAPTURE] Converting {ppm_file} to {output_png}...")
        try:
            with Image.open(ppm_file) as img:
                img.save(output_png, "PNG")
            print(f"[SUCCESS] Saved desktop screenshot: {output_png} ({img.size[0]}x{img.size[1]})")
            try: os.remove(ppm_file)
            except: pass
            return True
        except Exception as e:
            print(f"[CAPTURE] Failed to convert PPM: {e}")
            return False
    else:
        print(f"[CAPTURE] PPM screendump file was not generated.")
        return False

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "novaos_desktop.png"
    success = capture(out)
    sys.exit(0 if success else 1)
