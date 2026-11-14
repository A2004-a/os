//! NovaOS Apex — Milestone 1 Deliberate Exception & Fault Verification Suite
//! Proves that all 4 controlled exceptions (#DE, #UD, #PF, #DF) emit diagnostics and recover cleanly.

use super::exceptions::{TestMode, ACTIVE_TEST, TEST_RESULT_PASSED};
use crate::serial;
use core::arch::asm;

pub fn run_all_tests() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex (M1) — Deliberate Exception Test Suite  \n");
    serial::write_str("=======================================================\n");

    let t1 = test_divide_by_zero();
    let t2 = test_invalid_opcode();
    let t3 = test_page_fault();
    let t4 = test_double_fault();

    serial::write_str("\n=======================================================\n");
    serial::write_str("   Milestone 1 Acceptance Results Summary              \n");
    serial::write_str("=======================================================\n");
    serial::write_str("  1. Divide-by-Zero (#DE):       ");
    if t1 { serial::write_str("[PASS]\n"); } else { serial::write_str("[FAIL]\n"); }

    serial::write_str("  2. Invalid Opcode (#UD):       ");
    if t2 { serial::write_str("[PASS]\n"); } else { serial::write_str("[FAIL]\n"); }

    serial::write_str("  3. Page Fault (#PF):           ");
    if t3 { serial::write_str("[PASS]\n"); } else { serial::write_str("[FAIL]\n"); }

    serial::write_str("  4. Double Fault (IST1 #DF):    ");
    if t4 { serial::write_str("[PASS]\n"); } else { serial::write_str("[FAIL]\n"); }

    let all_passed = t1 && t2 && t3 && t4;
    if all_passed {
        serial::write_str("\n[M1 SUCCESS] All 4 exception gates verified. Zero silent resets.\n");
    } else {
        serial::write_str("\n[M1 FAILURE] One or more exception diagnostic tests failed.\n");
    }
    serial::write_str("=======================================================\n");

    all_passed
}

fn test_divide_by_zero() -> bool {
    serial::write_str("[TEST 1/4] Triggering deliberate Divide-by-Zero (#DE)...\n");
    unsafe {
        ACTIVE_TEST = TestMode::DivideByZero;
        TEST_RESULT_PASSED = false;

        asm!(
            "mov eax, 100",
            "xor ecx, ecx",
            "idiv ecx", // 2 bytes: F7 F9
            out("eax") _,
            out("ecx") _,
            options(nomem)
        );

        ACTIVE_TEST = TestMode::None;
        TEST_RESULT_PASSED
    }
}

fn test_invalid_opcode() -> bool {
    serial::write_str("[TEST 2/4] Triggering deliberate Invalid Opcode (#UD)...\n");
    unsafe {
        ACTIVE_TEST = TestMode::InvalidOpcode;
        TEST_RESULT_PASSED = false;

        asm!(
            "ud2", // 2 bytes: 0F 0B
            options(nomem)
        );

        ACTIVE_TEST = TestMode::None;
        TEST_RESULT_PASSED
    }
}

fn test_page_fault() -> bool {
    serial::write_str("[TEST 3/4] Triggering deliberate Page Fault (#PF at 0x7EADBEEF0000)...\n");
    unsafe {
        ACTIVE_TEST = TestMode::PageFault;
        TEST_RESULT_PASSED = false;

        asm!(
            "mov rdx, 0x00007EADBEEF0000",
            "mov rax, [rdx]", // 3 bytes: 48 8B 02
            out("rax") _,
            out("rdx") _,
            options(nostack)
        );

        ACTIVE_TEST = TestMode::None;
        TEST_RESULT_PASSED
    }
}

fn test_double_fault() -> bool {
    serial::write_str("[TEST 4/4] Triggering deliberate Double Fault (#DF) with IST1 stack...\n");
    unsafe {
        ACTIVE_TEST = TestMode::DoubleFault;
        TEST_RESULT_PASSED = false;

        // Software invoke vector 8 which is mapped to IST1 in IDT
        asm!(
            "int 8",
            options(nomem)
        );

        ACTIVE_TEST = TestMode::None;
        TEST_RESULT_PASSED
    }
}
