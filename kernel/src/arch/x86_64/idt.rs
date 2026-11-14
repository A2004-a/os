//! NovaOS Zero — Freestanding 256-Entry x86_64 Interrupt Descriptor Table (IDT)
//! Handles all 32 CPU Architecture Exceptions and Hardware IRQs

use core::mem::size_of;
use crate::arch::x86_64::registers::ControlRegisters;
use crate::serial_println;

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
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

    pub fn set_handler(&mut self, handler: usize, ist_index: u8) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.selector = 0x08; // Kernel code segment
        self.ist = ist_index & 0x7;
        self.type_attr = 0x8E; // Present, Ring 0, 64-bit Interrupt Gate
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.zero = 0;
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

static mut IDT_TABLE: [IdtEntry; 256] = [IdtEntry::empty(); 256];

#[repr(C)]
#[derive(Debug)]
pub struct InterruptContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub int_num: u64,
    pub error_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

// Global exception callback router
#[no_mangle]
pub extern "C" fn exception_dispatcher(ctx: &InterruptContext) {
    match ctx.int_num {
        0 => {
            serial_println!("\n[CPU EXCEPTION: #DE (Divide by Zero)]");
            serial_println!("  RIP: 0x{:016X}, CS: 0x{:04X}", ctx.rip, ctx.cs);
            serial_println!("  RSP: 0x{:016X}, RFLAGS: 0x{:08X}", ctx.rsp, ctx.rflags);
            panic!("Divide-by-zero fault encountered");
        }
        6 => {
            serial_println!("\n[CPU EXCEPTION: #UD (Invalid Opcode)]");
            serial_println!("  Faulting Instruction at RIP: 0x{:016X}", ctx.rip);
            serial_println!("  RAX: 0x{:016X}, RBX: 0x{:016X}", ctx.rax, ctx.rbx);
            panic!("Invalid CPU opcode executed");
        }
        8 => {
            serial_println!("\n[FATAL EXCEPTION: #DF (Double Fault)]");
            serial_println!("  Active on dedicated IST 1 emergency stack!");
            serial_println!("  RIP: 0x{:016X}, RSP: 0x{:016X}", ctx.rip, ctx.rsp);
            panic!("Fatal Double Fault trapped successfully without reset");
        }
        13 => {
            serial_println!("\n[CPU EXCEPTION: #GP (General Protection Fault)]");
            serial_println!("  Selector Error Code: 0x{:X}", ctx.error_code);
            serial_println!("  RIP: 0x{:016X}, RSP: 0x{:016X}", ctx.rip, ctx.rsp);
            panic!("General Protection Fault");
        }
        14 => {
            let cr2 = ControlRegisters::read_cr2();
            serial_println!("\n[CPU EXCEPTION: #PF (Page Fault)]");
            serial_println!("  Faulting Linear Address (CR2): 0x{:016X}", cr2);
            serial_println!("  Page Fault Error Code: 0x{:X}", ctx.error_code);
            serial_println!("  RIP: 0x{:016X}, RSP: 0x{:016X}", ctx.rip, ctx.rsp);
            panic!("Unrecoverable Page Fault");
        }
        32 => {
            crate::interrupts::timer::on_timer_tick();
            unsafe {
                crate::arch::x86_64::io::outb(0x20, 0x20); // EOI to Master PIC
            }
        }
        33 => {
            crate::drivers::ps2_keyboard::on_keyboard_interrupt();
            unsafe {
                crate::arch::x86_64::io::outb(0x20, 0x20); // EOI to Master PIC
            }
        }
        _ => {
            serial_println!("\n[CPU INTERRUPT / EXCEPTION: Vector {}]", ctx.int_num);
            serial_println!("  Error Code: 0x{:X}, RIP: 0x{:016X}", ctx.error_code, ctx.rip);
        }
    }
}

// Trampoline macros in assembly
macro_rules! isr_no_err {
    ($name:ident, $num:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "push 0",           // Dummy error code
                concat!("push ", $num), // Interrupt number
                "push rax", "push rbx", "push rcx", "push rdx",
                "push rsi", "push rdi", "push rbp", "push r8",
                "push r9", "push r10", "push r11", "push r12",
                "push r13", "push r14", "push r15",
                "mov rdi, rsp",
                "call exception_dispatcher",
                "pop r15", "pop r14", "pop r13", "pop r12",
                "pop r11", "pop r10", "pop r9", "pop r8",
                "pop rbp", "pop rdi", "pop rsi", "pop rdx",
                "pop rcx", "pop rbx", "pop rax",
                "add rsp, 16",      // Clean int_num and error_code
                "iretq",
            );
        }
    };
}

macro_rules! isr_err {
    ($name:ident, $num:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                concat!("push ", $num), // Interrupt number
                "push rax", "push rbx", "push rcx", "push rdx",
                "push rsi", "push rdi", "push rbp", "push r8",
                "push r9", "push r10", "push r11", "push r12",
                "push r13", "push r14", "push r15",
                "mov rdi, rsp",
                "call exception_dispatcher",
                "pop r15", "pop r14", "pop r13", "pop r12",
                "pop r11", "pop r10", "pop r9", "pop r8",
                "pop rbp", "pop rdi", "pop rsi", "pop rdx",
                "pop rcx", "pop rbx", "pop rax",
                "add rsp, 16",      // Clean int_num and error_code
                "iretq",
            );
        }
    };
}

isr_no_err!(isr0, 0);   // #DE
isr_no_err!(isr1, 1);   // #DB
isr_no_err!(isr2, 2);   // NMI
isr_no_err!(isr3, 3);   // #BP
isr_no_err!(isr4, 4);   // #OF
isr_no_err!(isr5, 5);   // #BR
isr_no_err!(isr6, 6);   // #UD
isr_no_err!(isr7, 7);   // #NM
isr_err!(isr8, 8);      // #DF (has error code)
isr_no_err!(isr9, 9);   // Coprocessor Segment Overrun
isr_err!(isr10, 10);    // #TS
isr_err!(isr11, 11);    // #NP
isr_err!(isr12, 12);    // #SS
isr_err!(isr13, 13);    // #GP
isr_err!(isr14, 14);    // #PF
isr_no_err!(isr15, 15); // Reserved
isr_no_err!(isr16, 16); // #MF
isr_err!(isr17, 17);    // #AC
isr_no_err!(isr18, 18); // #MC
isr_no_err!(isr19, 19); // #XM
isr_no_err!(isr20, 20); // #VE
isr_err!(isr21, 21);    // #CP

isr_no_err!(isr32, 32); // IRQ0 Timer
isr_no_err!(isr33, 33); // IRQ1 Keyboard

pub fn init() {
    unsafe {
        // Register CPU Architecture Exceptions
        IDT_TABLE[0].set_handler(isr0 as usize, 0);
        IDT_TABLE[1].set_handler(isr1 as usize, 0);
        IDT_TABLE[2].set_handler(isr2 as usize, 0);
        IDT_TABLE[3].set_handler(isr3 as usize, 0);
        IDT_TABLE[4].set_handler(isr4 as usize, 0);
        IDT_TABLE[5].set_handler(isr5 as usize, 0);
        IDT_TABLE[6].set_handler(isr6 as usize, 0);
        IDT_TABLE[7].set_handler(isr7 as usize, 0);
        // Vector 8 (#DF) uses IST 1 (index 1 in IDT entry field)
        IDT_TABLE[8].set_handler(isr8 as usize, 1);
        IDT_TABLE[9].set_handler(isr9 as usize, 0);
        IDT_TABLE[10].set_handler(isr10 as usize, 0);
        IDT_TABLE[11].set_handler(isr11 as usize, 0);
        IDT_TABLE[12].set_handler(isr12 as usize, 0);
        IDT_TABLE[13].set_handler(isr13 as usize, 0);
        IDT_TABLE[14].set_handler(isr14 as usize, 0);
        IDT_TABLE[15].set_handler(isr15 as usize, 0);
        IDT_TABLE[16].set_handler(isr16 as usize, 0);
        IDT_TABLE[17].set_handler(isr17 as usize, 0);
        IDT_TABLE[18].set_handler(isr18 as usize, 0);
        IDT_TABLE[19].set_handler(isr19 as usize, 0);
        IDT_TABLE[20].set_handler(isr20 as usize, 0);
        IDT_TABLE[21].set_handler(isr21 as usize, 0);

        // Hardware IRQ Interrupts
        IDT_TABLE[32].set_handler(isr32 as usize, 0);
        IDT_TABLE[33].set_handler(isr33 as usize, 0);

        let idt_desc = IdtDescriptor {
            size: (size_of::<[IdtEntry; 256]>() - 1) as u16,
            offset: IDT_TABLE.as_ptr() as u64,
        };

        core::arch::asm!("lidt [{}]", in(reg) &idt_desc, options(nostack, readonly));
    }
}
