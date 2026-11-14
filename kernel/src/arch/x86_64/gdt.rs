//! NovaOS Apex — 64-bit Global Descriptor Table (GDT) & Task State Segment (TSS)
//! Configures kernel/user segments and dedicated IST1 double-fault stack.

use super::instructions::{lgdt, ltr, reload_segments, DescriptorTablePointer};
use core::ptr::addr_of_mut;

pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub const USER_DATA_SELECTOR: u16 = 0x18 | 3;
pub const USER_CODE_SELECTOR: u16 = 0x20 | 3;
pub const TSS_SELECTOR: u16 = 0x28;

pub const DOUBLE_FAULT_IST_INDEX: u8 = 1;
pub const DOUBLE_FAULT_STACK_SIZE: usize = 32 * 1024; // 32 KiB

#[repr(C, packed)]
pub struct TaskStateSegment {
    pub reserved1: u32,
    pub rsp: [u64; 3],
    pub reserved2: u64,
    pub ist: [u64; 7],
    pub reserved3: u64,
    pub reserved4: u16,
    pub iomap_base: u16,
}

static mut TSS: TaskStateSegment = TaskStateSegment {
    reserved1: 0,
    rsp: [0; 3],
    reserved2: 0,
    ist: [0; 7],
    reserved3: 0,
    reserved4: 0,
    iomap_base: core::mem::size_of::<TaskStateSegment>() as u16,
};

#[allow(dead_code)]
#[repr(align(16))]
struct StackStorage([u8; DOUBLE_FAULT_STACK_SIZE]);

static mut DOUBLE_FAULT_STACK: StackStorage = StackStorage([0u8; DOUBLE_FAULT_STACK_SIZE]);

#[repr(C, align(16))]
pub struct GdtTable {
    pub entries: [u64; 7],
}

static mut GDT: GdtTable = GdtTable {
    entries: [
        0x0000000000000000, // 0x00: Null Descriptor
        0x00AF9A000000FFFF, // 0x08: Kernel Code 64 (Present, Ring 0, Read/Execute, 64-bit Long Mode)
        0x00CF92000000FFFF, // 0x10: Kernel Data 64 (Present, Ring 0, Read/Write)
        0x00CFF2000000FFFF, // 0x18: User Data 64   (Present, Ring 3, Read/Write)
        0x00AFF8000000FFFF, // 0x20: User Code 64   (Present, Ring 3, Read/Execute, 64-bit Long Mode)
        0x0000000000000000, // 0x28: TSS Low (populated dynamically)
        0x0000000000000000, // 0x30: TSS High (populated dynamically)
    ],
};

pub fn init() {
    unsafe {
        // 1. Configure TSS IST1 for dedicated Double Fault stack
        let tss_ptr = addr_of_mut!(TSS);
        let stack_ptr = addr_of_mut!(DOUBLE_FAULT_STACK);
        let stack_top = (stack_ptr as u64) + (DOUBLE_FAULT_STACK_SIZE as u64);

        (*tss_ptr).ist[0] = stack_top; // IST1 = index 0

        // 2. Configure 16-byte TSS descriptor in GDT entries 5 & 6
        let tss_base = tss_ptr as u64;
        let tss_limit = (core::mem::size_of::<TaskStateSegment>() - 1) as u64;

        // TSS Low (64-bit TSS Available = 0x89)
        let mut tss_low: u64 = 0;
        tss_low |= tss_limit & 0xFFFF;
        tss_low |= (tss_base & 0xFFFFFF) << 16;
        tss_low |= 0x89u64 << 40; // Present=1, DPL=0, Type=0x9 (64-bit TSS available)
        tss_low |= ((tss_limit >> 16) & 0x0F) << 48;
        tss_low |= ((tss_base >> 24) & 0xFF) << 56;

        // TSS High
        let tss_high: u64 = (tss_base >> 32) & 0xFFFFFFFF;

        let gdt_ptr = addr_of_mut!(GDT);
        (*gdt_ptr).entries[5] = tss_low;
        (*gdt_ptr).entries[6] = tss_high;

        // 3. Load GDT via lgdt
        let gdt_descriptor = DescriptorTablePointer {
            limit: (core::mem::size_of::<GdtTable>() - 1) as u16,
            base: gdt_ptr as u64,
        };
        lgdt(&gdt_descriptor);

        // 4. Reload segment registers
        reload_segments(KERNEL_CODE_SELECTOR, KERNEL_DATA_SELECTOR);

        // 5. Load Task Register
        ltr(TSS_SELECTOR);
    }
}

pub fn get_double_fault_stack_top() -> u64 {
    let stack_ptr = addr_of_mut!(DOUBLE_FAULT_STACK);
    (stack_ptr as u64) + (DOUBLE_FAULT_STACK_SIZE as u64)
}
