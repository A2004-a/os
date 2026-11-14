# NovaOS Apex — Hardware Compatibility List

## Target Architecture

- **Processor:** 64-bit x86_64 (Intel Core 2 Duo or newer, AMD Athlon 64 X2 or newer).
- **Firmware:** UEFI 2.0+ (64-bit). Legacy BIOS is unsupported.
- **RAM:** 256 MiB minimum (512 MiB+ recommended).

## Verified Emulation Hardware

| Component | Target Device | Status |
| :--- | :--- | :--- |
| **CPU** | QEMU `qemu64` / `host` | Supported |
| **Firmware** | EDK2 / OVMF UEFI | Supported |
| **Serial** | UART 16550 (COM1 0x3F8) | Supported |
| **Display** | UEFI GOP / QEMU `stdvga`, `bochs-display` | Supported |
| **Input** | PS/2 Keyboard & Mouse, USB HID Boot Protocol | Supported |
| **Storage** | VirtIO Block (`0x1AF4:0x1001`), AHCI SATA, NVMe 1.4 | Supported |
| **Network** | VirtIO Net (`0x1AF4:0x1000`), Intel e1000 | Supported |
| **Audio** | Intel High Definition Audio (HDA / Azalia) | Supported |
