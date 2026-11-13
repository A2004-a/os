# NovaOS Hardware Support Matrix

| Category | Component / Feature | Status | Notes |
|---|---|---|---|
| **CPU Architecture** | `x86_64` (64-Bit Long Mode) | Supported | Core kernel architecture |
| **Firmware** | UEFI 2.0+ (OVMF) | Supported | Native bootloader entry (`BOOTX64.EFI`) |
| **Firmware (Legacy)**| BIOS / MBR | Unsupported | NovaOS strictly requires UEFI GOP & 64-bit tables |
| **Memory** | 4-Level Paging (PML4) | Supported | Standard x86_64 MMU paging |
| **Console Output** | UART 16550 Serial (COM1 `0x3F8`)| Supported | Tested at 115200 baud, 8N1 |
| **Text Video** | VGA 80x25 Buffer (`0xb8000`) | Supported | Fallback text mode console |
| **Graphics Video** | UEFI GOP Linear Framebuffer | Supported | Tested on QEMU standard VGA / VirtIO GPU |
| **Interrupts** | Dual Chained 8259 PIC | Supported | Remapped to IRQ vectors 32..47 |
| **Timer** | Programmable Interval Timer (PIT)| Supported | 100 Hz system tick counter |
| **Input** | PS/2 Keyboard | Supported | Scancode Set 1 / US 104-key layout |
| **Bus** | PCI Configuration Space | Supported | Enumerates buses 0..8 via ports `0xCF8`/`0xCFC` |
| **Storage (RAM)** | In-Memory RamFS | Supported | Built-in test filesystem |
| **Storage (Block)**| VirtIO Block | Planned (M7) | Driver specification drafted |
| **Networking** | VirtIO Network | Planned (M9) | Ethernet/IP stack planned for later milestone |
