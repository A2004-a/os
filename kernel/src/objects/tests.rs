//! NovaOS Apex — Kernel Object & Handle Model Verification Suite
//! Tests typed object insertion, capability rights enforcement, handle duplication, and lifecycle reclamation.

use super::handle::{HandleError, HandleTable};
use super::object::*;
use super::types::{CapabilityRights, ObjectType};
use crate::serial;

pub fn run_object_tests() -> bool {
    serial::write_str("\n[OBJECT TESTS] Starting Kernel Object & Typed-Handle Test Suite...\n");

    let mut table = HandleTable::new();

    // 1. Create File Object with READ + DUPLICATE rights
    let file_payload = ObjectPayload::File(FileObject {
        inode: 101,
        path: [0u8; 64],
        size: 4096,
        cursor: 0,
    });
    let file_rights = CapabilityRights::READ.union(CapabilityRights::DUPLICATE).union(CapabilityRights::CLOSE);
    let h_file = match table.insert_object(file_payload, file_rights) {
        Ok(h) => h,
        Err(_) => {
            serial::write_str("  [FAIL] Failed to create File object\n");
            return false;
        }
    };
    serial::write_str("  [PASS 1/5] Created typed File object with Handle(");
    print_u64(h_file.0 as u64);
    serial::write_str(")\n");

    // 2. Validate Type Safety (Type Confusion Prevention)
    let type_confusion_result = table.get(h_file, ObjectType::Socket, CapabilityRights::READ);
    match type_confusion_result {
        Err(HandleError::TypeMismatch) => {
            serial::write_str("  [PASS 2/5] Type safety verified (TypeMismatch correctly rejected)\n");
        }
        _ => {
            serial::write_str("  [FAIL] Type confusion check failed (expected TypeMismatch)\n");
            return false;
        }
    }

    // 3. Validate Capability Rights Enforcement
    let access_denied_result = table.get(h_file, ObjectType::File, CapabilityRights::WRITE);
    match access_denied_result {
        Err(HandleError::AccessDenied) => {
            serial::write_str("  [PASS 3/5] Capability rights enforced (unauthorized WRITE denied)\n");
        }
        _ => {
            serial::write_str("  [FAIL] Capability access check failed (expected AccessDenied)\n");
            return false;
        }
    }

    // 4. Validate Handle Duplication & Refcounting
    let h_file_dup = match table.duplicate(h_file, CapabilityRights::READ) {
        Ok(h) => h,
        Err(_) => {
            serial::write_str("  [FAIL] Handle duplication failed\n");
            return false;
        }
    };
    let obj = table.get(h_file_dup, ObjectType::File, CapabilityRights::READ).unwrap();
    if obj.ref_count != 2 {
        serial::write_str("  [FAIL] Object ref_count incorrect after duplication\n");
        return false;
    }
    serial::write_str("  [PASS 4/5] Handle duplicated successfully (ref_count = 2)\n");

    // 5. Validate Lifecycle Reclamation on Close
    assert!(table.close(h_file).is_ok());
    // Object should still be accessible via duplicated handle
    assert!(table.get(h_file_dup, ObjectType::File, CapabilityRights::READ).is_ok());
    // Close duplicate handle -> ref_count drops to 0, object reclaimed
    assert!(table.close(h_file_dup).is_ok());
    // Both handles must now be invalid
    match table.get(h_file, ObjectType::File, CapabilityRights::READ) {
        Err(HandleError::InvalidHandle) => {}
        _ => {
            serial::write_str("  [FAIL] Closed handle 1 should be invalid\n");
            return false;
        }
    }
    match table.get(h_file_dup, ObjectType::File, CapabilityRights::READ) {
        Err(HandleError::InvalidHandle) => {}
        _ => {
            serial::write_str("  [FAIL] Closed handle 2 should be invalid\n");
            return false;
        }
    }
    serial::write_str("  [PASS 5/5] Object lifecycle and automatic reclamation verified\n");

    serial::write_str("[OBJECT SUCCESS] Kernel Object & Typed-Handle Model fully verified!\n");
    true
}

fn print_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
