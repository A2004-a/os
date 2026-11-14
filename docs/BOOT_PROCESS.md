# NovaOS Apex — Boot Process Specification

## Stage 1: UEFI Firmware Initialization
- The target machine powers on, executes firmware POST, and queries UEFI boot variables from NVRAM.
- UEFI loads `/EFI/BOOT/BOOTX64.EFI` from the EFI System Partition (FAT32).

## Stage 2: NovaOS Apex UEFI Boot Manager
- Bootloader starts at `efi_main`.
- Initializes early UART 16550 serial logging on COM1 (`0x3F8`).
- Reads `/novaos.cfg` or queries default boot parameters.
- Discovers Graphics Output Protocol (GOP), extracts linear framebuffer physical base address, resolution, and pixel format.
- Queries UEFI memory map descriptors and counts total physical RAM.
- Locates ACPI Root System Description Pointer (RSDP).
- Populates `BootInfo` structure.
- Calls `ExitBootServices` to terminate firmware boot drivers and claim exclusive machine ownership.
- Jumps to `kernel_entry(&BOOT_INFO)`.

## Stage 3: Kernel Core Initialization
- Sets up higher-half virtual address mapping and temporary page tables.
- Installs Global Descriptor Table (GDT) and Task State Segment (TSS) with an emergency Interrupt Stack Table (IST1).
- Installs Interrupt Descriptor Table (IDT) with native gates for all 32 CPU exceptions.
- Initializes physical frame allocator and 4-level paging mapper.
- Remaps 8259 PIC and enables hardware timer (100 Hz PIT).
- Initializes kernel heap allocator.
- Enumerates PCI bus for storage, network, and audio peripherals.

## Stage 4: Userland Services & Desktop Launch
- Kernel launches `novaos-apex-init` as PID 1 in unprivileged Ring 3 via `iretq`.
- `init` mounts VFS, starts system services, initializes Window Server, and launches the Nova Desktop Shell.
