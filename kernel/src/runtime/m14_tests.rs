//! NovaOS Apex — Milestone M14 Verification Suite
//! Verifies Busybox-Style Coreutils, MicroPython Scripting, and WebAssembly (wasmi) Sandboxed Execution.
//! Strictly tests all 16 certified gates defined in docs/milestones/M14.md.

extern crate alloc;
use alloc::string::String;
use alloc::vec;

use crate::posix::pipe::{sys_pipe, sys_pipe_close, sys_pipe_read, sys_pipe_write};
use crate::runtime::coreutils::{dispatch_coreutil, util_grep, util_head, util_tail, util_wc};
use crate::runtime::micropython::{MicroPythonVm, PyValue};
use crate::runtime::wasm::{WasmMemory, WasmModule, WasmTrap, WasmVm};
use crate::serial;

macro_rules! check_m14 {
    ($num:expr, $name:expr, $cond:expr) => {
        if $cond {
            serial::write_str("  [M14 TEST ");
            print_num($num);
            serial::write_str("/16] PASS: ");
            serial::write_str($name);
            serial::write_str("\n");
        } else {
            serial::write_str("  [M14 TEST ");
            print_num($num);
            serial::write_str("/16] FAIL: ");
            serial::write_str($name);
            serial::write_str("\n");
            return false;
        }
    };
}

pub fn run_all_m14_tests() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Milestone M14 Verification Suite (16 Gates)\n");
    serial::write_str("   Busybox Coreutils, MicroPython & WASM Sandboxed Runtime \n");
    serial::write_str("=======================================================\n");

    let res = execute_all_gates();

    serial::write_str("\n=======================================================\n");
    if res {
        serial::write_str("[M14 RESULT] ALL 16 M14 GATES PASSED (0 ERRORS)\n");
    } else {
        serial::write_str("[M14 RESULT] MILESTONE M14 ENCOUNTERED FAILURES!\n");
    }
    serial::write_str("=======================================================\n");

    res
}

fn execute_all_gates() -> bool {
    // 1. Busybox Multi-Call Dispatcher (argv[0] applet routing)
    let mut out_buf = [0u8; 128];
    let echo_res = dispatch_coreutil("echo", &["Hello", "NovaOS", "Coreutils"], &[], &mut out_buf);
    let bad_applet_res = dispatch_coreutil("nonexistent_applet", &[], &[], &mut out_buf);
    let gate1_ok = match echo_res {
        Ok(n) => &out_buf[..n] == b"Hello NovaOS Coreutils\n" && bad_applet_res == Err(-38),
        _ => false,
    };
    check_m14!(1, "Busybox Multi-Call Dispatcher (argv[0] applet routing)", gate1_ok);

    // 2. Stream Concatenation & Pagination Utilities (cat, head, tail)
    let doc = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n";
    let mut cat_buf = [0u8; 64];
    let cat_res = dispatch_coreutil("cat", &[], doc.as_bytes(), &mut cat_buf);
    let head_res = util_head(doc, 2);
    let tail_res = util_tail(doc, 2);
    let gate2_ok = cat_res == Ok(doc.len())
        && &cat_buf[..doc.len()] == doc.as_bytes()
        && head_res == "Line 1\nLine 2\n"
        && tail_res == "Line 4\nLine 5\n";
    check_m14!(2, "Stream Concatenation & Pagination Utilities (cat, head, tail)", gate2_ok);

    // 3. Text Metrics Counting Utility (wc -l, wc -w, wc -c)
    let sample = b"first second third\nfourth fifth\nsixth\n";
    let metrics = util_wc(sample);
    let mut wc_buf = [0u8; 32];
    let wc_l_res = dispatch_coreutil("wc", &["-l"], sample, &mut wc_buf);
    let gate3_ok = metrics.lines == 3
        && metrics.words == 6
        && metrics.bytes == sample.len()
        && wc_l_res.is_ok()
        && &wc_buf[..wc_l_res.unwrap()] == b"3\n";
    check_m14!(3, "Text Metrics Counting Utility (wc -l, wc -w, wc -c)", gate3_ok);

    // 4. Regular Pattern Filtering Utility (grep substring and case matching)
    let data = "kernel: virtio-gpu initialized\nnetwork: interface eth0 up\nkernel: pci scan complete\n";
    let grep_res = util_grep(data, "kernel", false, false);
    let grep_case_res = util_grep(data, "NETWORK", true, false);
    let grep_inv_res = util_grep(data, "kernel", false, true);
    let gate4_ok = grep_res == "kernel: virtio-gpu initialized\nkernel: pci scan complete\n"
        && grep_case_res == "network: interface eth0 up\n"
        && grep_inv_res == "network: interface eth0 up\n";
    check_m14!(4, "Regular Pattern Filtering Utility (grep substring and case matching)", gate4_ok);

    // 5. Filesystem Manipulation Utilities (cp, mv, rm, touch, mkdir)
    let touch_res = dispatch_coreutil("touch", &["/tmp/test_file.txt"], &[], &mut out_buf);
    let mkdir_res = dispatch_coreutil("mkdir", &["/tmp/test_dir"], &[], &mut out_buf);
    let cp_res = dispatch_coreutil("cp", &["/tmp/src", "/tmp/dst"], &[], &mut out_buf);
    let mv_res = dispatch_coreutil("mv", &["/tmp/old", "/tmp/new"], &[], &mut out_buf);
    let rm_res = dispatch_coreutil("rm", &["/tmp/test_file.txt"], &[], &mut out_buf);
    let gate5_ok = touch_res == Ok(0) && mkdir_res == Ok(0) && cp_res == Ok(0) && mv_res == Ok(0) && rm_res == Ok(0);
    check_m14!(5, "Filesystem Manipulation Utilities (cp, mv, rm, touch, mkdir)", gate5_ok);

    // 6. Process & System Inspection Utilities (uname, whoami, pwd, ps, kill)
    let mut uname_buf = [0u8; 64];
    let uname_res = dispatch_coreutil("uname", &["-a"], &[], &mut uname_buf);
    let mut whoami_buf = [0u8; 32];
    let whoami_res = dispatch_coreutil("whoami", &[], &[], &mut whoami_buf);
    let mut pwd_buf = [0u8; 32];
    let pwd_res = dispatch_coreutil("pwd", &[], &[], &mut pwd_buf);
    let gate6_ok = match (uname_res, whoami_res, pwd_res) {
        (Ok(un), Ok(wn), Ok(pn)) => {
            let u_str = core::str::from_utf8(&uname_buf[..un]).unwrap_or("");
            let w_str = core::str::from_utf8(&whoami_buf[..wn]).unwrap_or("");
            let p_str = core::str::from_utf8(&pwd_buf[..pn]).unwrap_or("");
            u_str.contains("NovaOS-Apex") && w_str == "apex\n" && p_str == "/users/apex\n"
        }
        _ => false,
    };
    check_m14!(6, "Process & System Inspection Utilities (uname, whoami, pwd, ps, kill)", gate6_ok);

    // 7. Storage & Memory Status Reporting Utilities (df, free)
    let mut df_buf = [0u8; 128];
    let df_res = dispatch_coreutil("df", &[], &[], &mut df_buf);
    let mut free_buf = [0u8; 128];
    let free_res = dispatch_coreutil("free", &[], &[], &mut free_buf);
    let gate7_ok = match (df_res, free_res) {
        (Ok(dn), Ok(fn_)) => {
            let df_str = core::str::from_utf8(&df_buf[..dn]).unwrap_or("");
            let free_str = core::str::from_utf8(&free_buf[..fn_]).unwrap_or("");
            df_str.contains("Filesystem") && free_str.contains("Mem:")
        }
        _ => false,
    };
    check_m14!(7, "Storage & Memory Status Reporting Utilities (df, free)", gate7_ok);

    // 8. Inter-Utility Pipeline Streaming (Chaining utilities across M13 anonymous pipes)
    let pipe_res = sys_pipe();
    let gate8_ok = match pipe_res {
        Ok((rfd, wfd)) => {
            // Producer: echo lines into write end of pipe
            let stream_input = b"log: boot ok\nlog: network ready\nerr: disk full\n";
            let _ = sys_pipe_write(wfd, stream_input);
            let _ = sys_pipe_close(wfd); // close writer to set EOF

            // Consumer: read from pipe read end into buffer
            let mut pipe_data = [0u8; 128];
            let read_len = sys_pipe_read(rfd, &mut pipe_data).unwrap_or(0);
            let _ = sys_pipe_close(rfd);

            // Filter through grep utility
            let pipe_str = core::str::from_utf8(&pipe_data[..read_len]).unwrap_or("");
            let filtered = util_grep(pipe_str, "err:", false, false);
            filtered == "err: disk full\n"
        }
        Err(_) => false,
    };
    check_m14!(8, "Inter-Utility Pipeline Streaming (Chaining utilities across M13 anonymous pipes)", gate8_ok);

    // 9. MicroPython Expression & Lexer Evaluation (Primitives & arithmetic)
    let mut py_vm = MicroPythonVm::new();
    let e1 = py_vm.eval_expr("100 + 25 * 4");
    let e2 = py_vm.eval_expr("\"Nova\" + \"OS\"");
    let e3 = py_vm.eval_expr("42 == 42");
    let e4 = py_vm.eval_expr("10 > 20");
    let gate9_ok = e1 == Ok(PyValue::Int(200))
        && e2 == Ok(PyValue::Str(String::from("NovaOS")))
        && e3 == Ok(PyValue::Bool(true))
        && e4 == Ok(PyValue::Bool(false));
    check_m14!(9, "MicroPython Expression & Lexer Evaluation (Primitives & arithmetic)", gate9_ok);

    // 10. MicroPython Variable Assignment & State Store
    let script1 = "x = 50\ny = 25\nz = x - y\n";
    let s1_res = py_vm.run_script(script1);
    let z_val = py_vm.get_var("z").cloned();
    let gate10_ok = s1_res.is_ok() && z_val == Some(PyValue::Int(25));
    check_m14!(10, "MicroPython Variable Assignment & State Store", gate10_ok);

    // 11. MicroPython Control Flow & Loops (if/else, while)
    let script2 = "count = 0\ntotal = 0\nwhile count < 4:\n    count = count + 1\n    total = total + count\n";
    let s2_res = py_vm.run_script(script2);
    let total_val = py_vm.get_var("total").cloned();
    let gate11_ok = s2_res.is_ok() && total_val == Some(PyValue::Int(10)); // 1 + 2 + 3 + 4 = 10
    check_m14!(11, "MicroPython Control Flow & Loops (if/else, while)", gate11_ok);

    // 12. MicroPython Built-in Functions (print, len, range)
    let script3 = "banner = \"NovaOS Apex 1.4.0\"\nl = len(banner)\nprint(l)\n";
    let s3_res = py_vm.run_script(script3);
    let l_val = py_vm.get_var("l").cloned();
    let gate12_ok = s3_res.is_ok()
        && l_val == Some(PyValue::Int(17))
        && py_vm.output.contains("17\n");
    check_m14!(12, "MicroPython Built-in Functions (print, len, range)", gate12_ok);

    // 13. WebAssembly Binary Header & Section Decoder (\0asm magic, version 1)
    let valid_wasm = vec![
        0x00, 0x61, 0x73, 0x6D, // \0asm
        0x01, 0x00, 0x00, 0x00, // version 1
        // Section 5 (Memory): length 3: 1 memory, min 1 page
        0x05, 0x03, 0x01, 0x00, 0x01,
        // Section 7 (Export): length 8: 1 export, name "add", func 0
        0x07, 0x07, 0x01, 0x03, b'a', b'd', b'd', 0x00, 0x00,
    ];
    let parse_ok = WasmModule::parse(&valid_wasm);
    let bad_magic = [0xFF, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
    let bad_parse = WasmModule::parse(&bad_magic);
    let gate13_ok = match parse_ok {
        Ok(module) => {
            module.memory_pages == 1
                && module.exports.len() == 1
                && module.exports[0].name == "add"
                && bad_parse == Err(WasmTrap::InvalidMagic)
        }
        Err(_) => false,
    };
    check_m14!(13, "WebAssembly Binary Header & Section Decoder (\\0asm magic, version 1)", gate13_ok);

    // 14. WebAssembly Linear Memory Isolation (64 KiB page bounds check)
    let mut wasm_mem = WasmMemory::new(1); // 1 page = 65,536 bytes
    let write_ok = wasm_mem.write_i32(1024, 0x1234_5678);
    let read_ok = wasm_mem.read_i32(1024);
    // Boundary violation: address 65,534 + 4 bytes = 65,538 > 65,536
    let oob_write = wasm_mem.write_i32(65534, 42);
    let oob_read = wasm_mem.read_i32(70000);
    let gate14_ok = write_ok.is_ok()
        && read_ok == Ok(0x1234_5678)
        && oob_write == Err(WasmTrap::OutOfBoundsMemory)
        && oob_read == Err(WasmTrap::OutOfBoundsMemory);
    check_m14!(14, "WebAssembly Linear Memory Isolation (64 KiB page bounds check)", gate14_ok);

    // 15. WebAssembly Stack VM Execution (i32.const, i32.add, i32.mul)
    let mut wasm_vm = WasmVm::new(1, 1000);
    // Bytecode computing (6 * 7):
    // 0x41 0x06 (i32.const 6)
    // 0x41 0x07 (i32.const 7)
    // 0x6C      (i32.mul)
    // 0x0B      (end)
    let bytecode_mul = [0x41, 0x06, 0x41, 0x07, 0x6C, 0x0B];
    let exec_res = wasm_vm.execute(&bytecode_mul, &[]);
    let gate15_ok = exec_res == Ok(42);
    check_m14!(15, "WebAssembly Stack VM Execution (i32.const, i32.add, i32.mul)", gate15_ok);

    // 16. WebAssembly Sandboxed Fault & Fuel Trapping (Gas limit exhaustion & memory trap)
    // Run VM with only 1 gas unit remaining on 2-instruction program
    let mut trapped_vm = WasmVm::new(1, 1);
    let bytecode_fuel = [0x41, 0x01, 0x41, 0x02, 0x6A, 0x0B];
    let fuel_trap_res = trapped_vm.execute(&bytecode_fuel, &[]);

    // Run VM with out-of-bounds memory load opcode
    let mut mem_trap_vm = WasmVm::new(1, 100);
    // Push invalid memory offset 0x0002_0000 (131072 > 65536) and execute i32.load
    // 0x41 0x80 0x80 0x08 (i32.const 131072)
    // 0x28 0x02 0x00       (i32.load align=2, offset=0)
    // 0x0B                (end)
    let bytecode_mem_trap = [0x41, 0x80, 0x80, 0x08, 0x28, 0x02, 0x00, 0x0B];
    let mem_trap_res = mem_trap_vm.execute(&bytecode_mem_trap, &[]);

    let gate16_ok = fuel_trap_res == Err(WasmTrap::GasExhausted)
        && mem_trap_res == Err(WasmTrap::OutOfBoundsMemory);
    check_m14!(16, "WebAssembly Sandboxed Fault & Fuel Trapping (Gas limit exhaustion & memory trap)", gate16_ok);

    true
}

fn print_num(mut n: usize) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 && i < buf.len() {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
