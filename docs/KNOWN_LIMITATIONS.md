# NovaOS Known Limitations

In adherence to Section 1 of the Master Directive, NovaOS explicitly reports all current systems boundaries:

1. **Bare-Metal Operating System Boundary:**
   * NovaOS is an independent, freestanding operating system kernel. It does **not** provide compatibility layers for executing Windows (`.exe`/Win32), Linux (`ELF` glibc/musl), or macOS (`Mach-O`) application binaries.
2. **Current Storage Persistence:**
   * The kernel currently operates from an in-memory RamFS and the initial UEFI boot volume. Disk persistence to NVMe/SATA/VirtIO block devices is scheduled for Milestone M7.
3. **Multitasking:**
   * The current scheduler provides cooperative task yields and PIT timer tick time-slicing. Preemptive multi-core APIC context-switching (SMP) will be enabled in milestone M5.2.
4. **Hardware Networking:**
   * Physical Ethernet, Wi-Fi, and TCP/IP networking are scheduled for Milestone M9 and are not currently active.
5. **Graphical Interface:**
   * Visual output currently utilizes linear UEFI GOP framebuffer rendering (drawing rectangles, text strings via bitmap font, and VGA fallback). A full multi-window desktop compositor is scheduled for Milestone M10.
