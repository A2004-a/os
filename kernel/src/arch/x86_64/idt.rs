//! NovaOS Apex — 256-Entry Interrupt Descriptor Table (IDT) & Assembly Trampolines
//! Maps hardware exceptions, sets IST1 for Double Fault (#DF), and manages IDTR.

use super::gdt::{DOUBLE_FAULT_IST_INDEX, KERNEL_CODE_SELECTOR};
use super::instructions::{lidt, DescriptorTablePointer};
use core::arch::global_asm;
use core::ptr::addr_of_mut;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct IdtEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub ist: u8,
    pub type_attr: u8,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub zero: u32,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    pub fn set_handler(&mut self, handler: usize, ist: u8, dpl: u8) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.selector = KERNEL_CODE_SELECTOR;
        self.ist = ist & 0x07;
        self.type_attr = 0x80 | ((dpl & 3) << 5) | 0x0E; // Present=1, DPL, 64-bit Interrupt Gate (0xE)
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = ((handler >> 32) & 0xFFFFFFFF) as u32;
        self.zero = 0;
    }
}

#[repr(C, align(16))]
pub struct IdtTable {
    pub entries: [IdtEntry; 256],
}

static mut IDT: IdtTable = IdtTable {
    entries: [IdtEntry::empty(); 256],
};

extern "C" {
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();
    fn isr_default();
}

pub fn init() {
    unsafe {
        let idt_ptr = addr_of_mut!(IDT);
        let entries = &mut (*idt_ptr).entries;

        // Set default handler for all 256 interrupt gates
        for entry in entries.iter_mut() {
            entry.set_handler(isr_default as *const () as usize, 0, 0);
        }

        // Register handlers for standard x86_64 architecture exceptions (0..31)
        entries[0].set_handler(isr0 as *const () as usize, 0, 0);   // #DE: Divide Error
        entries[1].set_handler(isr1 as *const () as usize, 0, 0);   // #DB: Debug
        entries[2].set_handler(isr2 as *const () as usize, 0, 0);   // #NMI: Non-Maskable Interrupt
        entries[3].set_handler(isr3 as *const () as usize, 0, 3);   // #BP: Breakpoint (User callable)
        entries[4].set_handler(isr4 as *const () as usize, 0, 0);   // #OF: Overflow
        entries[5].set_handler(isr5 as *const () as usize, 0, 0);   // #BR: Bound Range
        entries[6].set_handler(isr6 as *const () as usize, 0, 0);   // #UD: Invalid Opcode
        entries[7].set_handler(isr7 as *const () as usize, 0, 0);   // #NM: Device Not Available
        entries[8].set_handler(isr8 as *const () as usize, DOUBLE_FAULT_IST_INDEX, 0); // #DF: Double Fault on IST1!
        entries[9].set_handler(isr9 as *const () as usize, 0, 0);   // Coprocessor Segment Overrun
        entries[10].set_handler(isr10 as *const () as usize, 0, 0); // #TS: Invalid TSS
        entries[11].set_handler(isr11 as *const () as usize, 0, 0); // #NP: Segment Not Present
        entries[12].set_handler(isr12 as *const () as usize, 0, 0); // #SS: Stack Fault
        entries[13].set_handler(isr13 as *const () as usize, 0, 0); // #GP: General Protection Fault
        entries[14].set_handler(isr14 as *const () as usize, 0, 0); // #PF: Page Fault
        entries[15].set_handler(isr15 as *const () as usize, 0, 0); // Reserved
        entries[16].set_handler(isr16 as *const () as usize, 0, 0); // #MF: x87 FPU Floating-Point Error
        entries[17].set_handler(isr17 as *const () as usize, 0, 0); // #AC: Alignment Check
        entries[18].set_handler(isr18 as *const () as usize, 0, 0); // #MC: Machine Check
        entries[19].set_handler(isr19 as *const () as usize, 0, 0); // #XM: SIMD Floating-Point Exception
        entries[20].set_handler(isr20 as *const () as usize, 0, 0); // #VE: Virtualization Exception
        entries[21].set_handler(isr21 as *const () as usize, 0, 0); // #CP: Control Protection Exception
        entries[22].set_handler(isr22 as *const () as usize, 0, 0);
        entries[23].set_handler(isr23 as *const () as usize, 0, 0);
        entries[24].set_handler(isr24 as *const () as usize, 0, 0);
        entries[25].set_handler(isr25 as *const () as usize, 0, 0);
        entries[26].set_handler(isr26 as *const () as usize, 0, 0);
        entries[27].set_handler(isr27 as *const () as usize, 0, 0);
        entries[28].set_handler(isr28 as *const () as usize, 0, 0);
        entries[29].set_handler(isr29 as *const () as usize, 0, 0); // #VC: VMM Communication Exception
        entries[30].set_handler(isr30 as *const () as usize, 0, 0); // #SX: Security Exception
        entries[31].set_handler(isr31 as *const () as usize, 0, 0);

        // Load IDTR via lidt instruction
        let idt_descriptor = DescriptorTablePointer {
            limit: (core::mem::size_of::<IdtTable>() - 1) as u16,
            base: idt_ptr as u64,
        };
        lidt(&idt_descriptor);
    }
}

global_asm!(
    r#"
.macro ISR_NOERRCODE num
.global isr\num
isr\num:
    push 0
    push \num
    jmp isr_common
.endm

.macro ISR_ERRCODE num
.global isr\num
isr\num:
    push \num
    jmp isr_common
.endm

ISR_NOERRCODE 0
ISR_NOERRCODE 1
ISR_NOERRCODE 2
ISR_NOERRCODE 3
ISR_NOERRCODE 4
ISR_NOERRCODE 5
ISR_NOERRCODE 6
ISR_NOERRCODE 7
ISR_ERRCODE   8
ISR_NOERRCODE 9
ISR_ERRCODE   10
ISR_ERRCODE   11
ISR_ERRCODE   12
ISR_ERRCODE   13
ISR_ERRCODE   14
ISR_NOERRCODE 15
ISR_NOERRCODE 16
ISR_ERRCODE   17
ISR_NOERRCODE 18
ISR_NOERRCODE 19
ISR_NOERRCODE 20
ISR_ERRCODE   21
ISR_NOERRCODE 22
ISR_NOERRCODE 23
ISR_NOERRCODE 24
ISR_NOERRCODE 25
ISR_NOERRCODE 26
ISR_NOERRCODE 27
ISR_NOERRCODE 28
ISR_ERRCODE   29
ISR_ERRCODE   30
ISR_NOERRCODE 31

.global isr_default
isr_default:
    push 0
    push 255
    jmp isr_common

.global isr_common
isr_common:
    // Save general-purpose registers (SysV ABI & ExceptionContext)
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    // First argument to handle_exception in rdi is pointer to ExceptionContext (rsp)
    mov rdi, rsp
    call handle_exception

    // Restore general-purpose registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    // Pop vector (8 bytes) and error code (8 bytes)
    add rsp, 16

    iretq
"#
);
