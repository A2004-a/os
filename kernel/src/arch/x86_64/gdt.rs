//! NovaOS Zero — Freestanding 64-bit Global Descriptor Table (GDT) & Task State Segment (TSS)

use core::mem::size_of;

pub const DOUBLE_FAULT_IST_INDEX: usize = 0; // IST 1

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
pub struct TaskStateSegment {
    pub reserved_1: u32,
    pub rsp: [u64; 3],
    pub reserved_2: u64,
    pub ist: [u64; 7],
    pub reserved_3: u64,
    pub reserved_4: u16,
    pub iomap_base: u16,
}

const EMERGENCY_STACK_SIZE: usize = 4096 * 5; // 20 KiB isolated emergency stack
static mut EMERGENCY_STACK: [u8; EMERGENCY_STACK_SIZE] = [0; EMERGENCY_STACK_SIZE];

static mut TSS: TaskStateSegment = TaskStateSegment {
    reserved_1: 0,
    rsp: [0; 3],
    reserved_2: 0,
    ist: [0; 7],
    reserved_3: 0,
    reserved_4: 0,
    iomap_base: size_of::<TaskStateSegment>() as u16,
};

#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

static mut GDT_ENTRIES: [u64; 7] = [
    0x0000000000000000, // 0x00: Null
    0x00AF9A000000FFFF, // 0x08: Kernel Code 64-bit (Ring 0)
    0x00CF92000000FFFF, // 0x10: Kernel Data 64-bit (Ring 0)
    0x00CFF2000000FFFF, // 0x18: User Data 64-bit (Ring 3)
    0x00AFFB000000FFFF, // 0x20: User Code 64-bit (Ring 3)
    0x0000000000000000, // 0x28: TSS Low (populated at runtime)
    0x0000000000000000, // 0x30: TSS High (populated at runtime)
];

pub fn init() {
    unsafe {
        // Set up IST 1 stack pointer for emergency double faults
        let stack_end = EMERGENCY_STACK.as_ptr() as u64 + EMERGENCY_STACK_SIZE as u64;
        TSS.ist[DOUBLE_FAULT_IST_INDEX] = stack_end;

        // Configure 16-byte TSS descriptor in GDT
        let tss_addr = &raw const TSS as u64;
        let tss_size = (size_of::<TaskStateSegment>() - 1) as u64;

        let limit_low = tss_size & 0xFFFF;
        let base_low = tss_addr & 0xFFFF;
        let base_mid = (tss_addr >> 16) & 0xFF;
        let flags = 0x89; // Present (0x80) | 64-bit TSS available (0x09)
        let limit_high_flags = (tss_size >> 16) & 0x0F;
        let base_high = (tss_addr >> 24) & 0xFF;

        GDT_ENTRIES[5] = limit_low
            | (base_low << 16)
            | (base_mid << 32)
            | ((flags as u64) << 40)
            | (limit_high_flags << 48)
            | (base_high << 56);
        GDT_ENTRIES[6] = tss_addr >> 32;

        let gdt_desc = GdtDescriptor {
            size: (size_of::<[u64; 7]>() - 1) as u16,
            offset: GDT_ENTRIES.as_ptr() as u64,
        };

        // Load GDT
        core::arch::asm!("lgdt [{}]", in(reg) &gdt_desc, options(nostack, readonly));

        // Reload segment registers with kernel selectors
        core::arch::asm!(
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            // Far jump to reload CS selector with 0x08
            "lea rax, [rip + 2f]",
            "push 0x08",
            "push rax",
            "retfq",
            "2:",
            out("rax") _,
            options(nomem)
        );

        // Load TSS selector (0x28)
        core::arch::asm!("ltr ax", in("ax") 0x28u16, options(nostack));
    }
}
