# NovaOS Boot Process Specification

```mermaid
sequenceDiagram
    participant Firmware as UEFI Firmware (OVMF)
    participant Loader as uefi-loader (BOOTX64.EFI)
    participant Kernel as novaos-kernel (kernel_entry)

    Firmware->>Loader: Start UEFI application from EFI/BOOT/BOOTX64.EFI
    Loader->>Loader: Initialize UEFI services & print banner
    Loader->>Loader: Open GraphicsOutput protocol (GOP)
    Loader->>Loader: Query UEFI memory map & locate ACPI RSDP
    Loader->>Loader: Build structured BootInfo block
    Loader->>Firmware: ExitBootServices()
    Loader->>Kernel: Branch to kernel_entry(&BootInfo)
    Kernel->>Kernel: Verify BootInfo magic (0x4E4F56414F532131)
    Kernel->>Kernel: Initialize GDT, IDT, PIC, Paging, and Heap
    Kernel->>Kernel: Enter interactive CPU halt loop (hlt)
```

1. **UEFI Entry:** The firmware locates `\EFI\BOOT\BOOTX64.EFI` on the FAT32 boot volume.
2. **Protocol Discovery:** `uefi-loader` requests the Graphics Output Protocol (GOP) to obtain the physical address, dimensions, and pixel format of the display framebuffer.
3. **Memory Map Construction:** `boot_services.memory_map()` records physical RAM regions.
4. **BootInfo Handover:** `BootInfo` structure is populated with framebuffer details, memory regions, and ACPI pointers.
5. **Exit Boot Services:** Firmware boot services are terminated. Control transfers to `kernel_entry`.
