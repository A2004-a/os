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
        let stack_end = core::ptr::addr_of_mut!(EMERGENCY_STACK) as u64 + EMERGENCY_STACK_SIZE as u64;
        TSS.ist[DOUBLE_FAULT_IST_INDEX] = stack_end;

        // Configure 16-byte TSS descriptor in GDT
        let tss_addr = core::ptr::addr_of_mut!(TSS) as u64;
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
            offset: core::ptr::addr_of_mut!(GDT_ENTRIES) as u64,
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

/// Update TSS.rsp[0] so CPU interrupts/exceptions from Ring 3 switch to this kernel stack.
pub fn set_kernel_stack(stack_top: u64) {
    unsafe {
        TSS.rsp[0] = stack_top;
    }
}

/// Retrieve current TSS.rsp[0]
pub fn get_kernel_stack() -> u64 {
    unsafe { TSS.rsp[0] }
}

/// Transition CPU from Ring 0 to Ring 3 userspace via iretq.
/// - CS: User Code 64-bit RPL=3 (0x23)
/// - SS: User Data 64-bit RPL=3 (0x1B)
/// - RFLAGS: 0x202 (Interrupt Enable flag)
#[unsafe(naked)]
pub unsafe extern "C" fn enter_user_mode(entry_rip: u64, user_rsp: u64) -> ! {
    core::arch::naked_asm!(
        // rdi = entry_rip, rsi = user_rsp
        "mov ax, 0x1B",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",

        // Build iretq stack frame: SS, RSP, RFLAGS, CS, RIP
        "push 0x1B",        // SS: User Data 64-bit RPL=3
        "push rsi",         // RSP: User Stack Pointer
        "push 0x202",       // RFLAGS: IF=1, bit 1=1
        "push 0x23",        // CS: User Code 64-bit RPL=3
        "push rdi",         // RIP: User Entry Point
        "iretq"
    );
}
