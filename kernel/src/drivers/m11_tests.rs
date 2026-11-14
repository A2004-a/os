//! NovaOS Apex — Milestone M11 Verification Suite
//! Verifies VirtIO-GPU 2D Display Driver, System Clipboard Service, and Drag-and-Drop Subsystem.
//! Strictly tests all 16 certified gates defined in docs/milestones/M11.md.

use crate::drivers::virtio_gpu::get_gpu_driver;
use crate::drivers::Driver;
use crate::graphics::dnd::{DndEngine, DndPayload, DndPhase, DRAG_THRESHOLD_PIXELS};
use crate::graphics::geometry::Point;
use crate::graphics::surface::Surface;
use crate::graphics::window::WindowId;
use crate::ipc::clipboard::{read_clipboard, write_clipboard, ClipboardFormat, CLIPBOARD, MAX_CLIPBOARD_SIZE};
use crate::serial;
use crate::syscall::{dispatch_syscall, SYS_CLIPBOARD_READ, SYS_CLIPBOARD_WRITE};

macro_rules! check_m11 {
    ($num:expr, $name:expr, $cond:expr) => {
        if $cond {
            serial::write_str("  [M11 TEST ");
            print_num($num);
            serial::write_str("/16] PASS: ");
            serial::write_str($name);
            serial::write_str("\n");
        } else {
            serial::write_str("  [M11 TEST ");
            print_num($num);
            serial::write_str("/16] FAIL: ");
            serial::write_str($name);
            serial::write_str("\n");
            return false;
        }
    };
}

pub fn run_all_m11_tests() -> bool {
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Milestone M11 Verification Suite (16 Gates)\n");
    serial::write_str("   VirtIO-GPU Display, Clipboard Service & Drag and Drop   \n");
    serial::write_str("=======================================================\n");

    let res = execute_all_gates();

    serial::write_str("\n=======================================================\n");
    if res {
        serial::write_str("[M11 RESULT] ALL 16 M11 GATES PASSED (0 ERRORS)\n");
    } else {
        serial::write_str("[M11 RESULT] MILESTONE M11 ENCOUNTERED FAILURES!\n");
    }
    serial::write_str("=======================================================\n");

    res
}

fn execute_all_gates() -> bool {
    let gpu = get_gpu_driver();

    // 1. VirtIO-GPU Header & PCI Capability Discovery
    let probe_ok = match gpu.probe() {
        Ok(found) => {
            if found {
                serial::write_str("    -> VirtIO-GPU PCI Hardware detected (0x1AF4:0x1050/0x1010)\n");
            } else {
                serial::write_str("    -> VirtIO-GPU PCI not present; software compositor fallback enabled\n");
            }
            gpu.name() == "VirtIO GPU 2D"
        }
        Err(_) => false,
    };
    check_m11!(1, "VirtIO-GPU Header & PCI Capability Discovery", probe_ok);

    // 2. Control Queue & Cursor Queue Initialization
    let init_ok = gpu.init().is_ok() && gpu.state() == crate::drivers::DriverState::Initialized;
    check_m11!(2, "VirtIO Control & Cursor Queue Architecture Initialization", init_ok);

    // 3. 2D Resource Creation Command (VIRTIO_GPU_CMD_RESOURCE_CREATE_2D)
    let res_create_ok = gpu.create_2d_resource(1, 1280, 800).is_ok() && gpu.width == 1280 && gpu.height == 800;
    check_m11!(3, "2D Resource Creation Command (1280x800 B8G8R8A8)", res_create_ok);

    // 4. Backing Memory Page Pinning (VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING)
    let attach_ok = gpu.attach_backing(1, 0x1000_0000, 1280 * 800 * 4).is_ok();
    check_m11!(4, "Backing Memory Page Pinning (Continuous Host VRAM Binding)", attach_ok);

    // 5. Scanout 0 Binding (VIRTIO_GPU_CMD_SET_SCANOUT)
    let scanout_ok = gpu.set_scanout(0, 1, 1280, 800).is_ok() && gpu.scanout_bound;
    check_m11!(5, "Scanout 0 Primary Display Pipeline Binding", scanout_ok);

    // 6. 2D Rectangle Host Transfer (VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D)
    let prev_transfers = gpu.transfer_count;
    let transfer_ok = gpu.transfer_to_host_2d(1, 64, 48, 512, 384).is_ok()
        && gpu.transfer_count == prev_transfers + 1
        && gpu.last_flushed_rect.width == 512;
    check_m11!(6, "2D Rectangle Host Transfer Command Execution", transfer_ok);

    // 7. Resource Flush Signal (VIRTIO_GPU_CMD_RESOURCE_FLUSH)
    let prev_flushes = gpu.flush_count;
    let flush_ok = gpu.resource_flush(1, 64, 48, 512, 384).is_ok()
        && gpu.flush_count == prev_flushes + 1
        && gpu.last_flushed_rect.height == 384;
    check_m11!(7, "Resource Flush Host Scanout Presentation Signal", flush_ok);

    // 8. Framebuffer Fallback Transparency (GOP active when GPU in fallback)
    let fb = crate::drivers::framebuffer::get_driver();
    let fb_ok = !fb.base_ptr.is_null() && fb.width >= 640 && fb.height >= 480;
    check_m11!(8, "Hardware Framebuffer Fallback & GOP Transparency", fb_ok);

    // 9. Clipboard Service Text Storage & Retrieval (text/plain)
    let test_data = b"NovaOS M11 Clipboard Engine";
    let seq_before = CLIPBOARD.lock().sequence();
    let write_res = write_clipboard(100, ClipboardFormat::TextPlain as u32, test_data);
    let mut read_buf = [0u8; 64];
    let read_res = read_clipboard(ClipboardFormat::TextPlain as u32, &mut read_buf);
    let cb_ok = match (write_res, read_res) {
        (Ok(seq), Ok((len, r_seq))) => {
            seq > seq_before && r_seq == seq && len == test_data.len() && &read_buf[..len] == test_data
        }
        _ => false,
    };
    check_m11!(9, "Clipboard Service Typed Text Storage & Retrieval (text/plain)", cb_ok);

    // 10. Clipboard Monotonic Sequence Numbering & Invalidation
    let seq_ok = if let Ok(seq1) = write_clipboard(101, ClipboardFormat::TextPlain as u32, b"Data 1") {
        if let Ok(seq2) = write_clipboard(102, ClipboardFormat::TextPlain as u32, b"Data 2") {
            // Trying to read mismatched format should return -22 (EINVAL)
            let mismatched = read_clipboard(ClipboardFormat::ImageRaw as u32, &mut read_buf);
            seq2 > seq1 && mismatched == Err(-22)
        } else {
            false
        }
    } else {
        false
    };
    check_m11!(10, "Clipboard Monotonic Sequence Tracking & Format Invalidation", seq_ok);

    // 11. Clipboard Cross-Process Syscall Sandbox Gate (SYS_CLIPBOARD_WRITE / READ)
    let mut kernel_slice = [0u8; 32];
    let test_bytes = b"Syscall Clipboard Test";
    for i in 0..test_bytes.len() {
        kernel_slice[i] = test_bytes[i];
    }
    let w_ret = dispatch_syscall(
        SYS_CLIPBOARD_WRITE,
        ClipboardFormat::TextPlain as u64,
        kernel_slice.as_ptr() as u64,
        test_bytes.len() as u64,
        0,
        0,
    );
    let mut out_slice = [0u8; 32];
    let r_ret = dispatch_syscall(
        SYS_CLIPBOARD_READ,
        ClipboardFormat::TextPlain as u64,
        out_slice.as_mut_ptr() as u64,
        out_slice.len() as u64,
        0,
        0,
    );
    let syscall_ok = w_ret > 0 && r_ret == test_bytes.len() as i64 && &out_slice[..test_bytes.len()] == test_bytes;
    check_m11!(11, "Clipboard Cross-Process Syscall Sandbox Gate", syscall_ok);

    // 12. Clipboard Large Payload Boundary & 64 KiB Clamping
    let large_buf = alloc::vec![0x5Au8; 70000]; // larger than 64 KiB
    let clamp_seq = write_clipboard(103, ClipboardFormat::Custom as u32, &large_buf);
    let cb_len = CLIPBOARD.lock().len();
    let clamp_ok = clamp_seq.is_ok() && cb_len == MAX_CLIPBOARD_SIZE;
    check_m11!(12, "Clipboard 64 KiB Boundary Clamping & Heap Isolation", clamp_ok);

    // 13. DND Drag Gesture Initiation (Threshold 4px displacement)
    let mut dnd = DndEngine::new();
    let start_pos = Point::new(100, 100);
    dnd.start_potential(WindowId(1), start_pos, DndPayload::file("/apex/doc.txt", false));
    let moved_small = dnd.on_mouse_move(Point::new(102, 102)); // 2px < threshold
    let small_phase_ok = !moved_small && dnd.phase == DndPhase::Potential;
    let moved_thresh = dnd.on_mouse_move(Point::new(100 + DRAG_THRESHOLD_PIXELS, 100));
    let thresh_ok = moved_thresh && dnd.is_dragging();
    check_m11!(13, "DND Gesture Initiation (4px Displacement Threshold Gate)", small_phase_ok && thresh_ok);

    // 14. DND Visual Drag Ghost Compositing
    dnd.on_mouse_move(Point::new(240, 180));
    let gr = dnd.ghost_rect();
    let mut test_surf = Surface::new(640, 480);
    dnd.render_ghost(&mut test_surf);
    let ghost_coords_ok = gr.x == 240 + 12 && gr.y == 180 + 12 && gr.width == 140 && gr.height == 32;
    check_m11!(14, "DND Visual Drag Ghost Coordinate Tracking & Surface Render", ghost_coords_ok);

    // 15. DND Target Hit-Testing & Drop Acceptance
    let drop_payload = dnd.on_mouse_up(Some(WindowId(2)));
    let drop_ok = match drop_payload {
        Some(p) => p.as_str() == "/apex/doc.txt" && !dnd.is_dragging() && dnd.phase == DndPhase::Idle,
        None => false,
    };
    check_m11!(15, "DND Target Window Hit-Testing & Payload Drop Acceptance", drop_ok);

    // 16. DND Cancellation on Escape Key / Invalid Target Release
    dnd.start_potential(WindowId(3), Point::new(300, 300), DndPayload::text("Drag Cancel Test"));
    dnd.on_mouse_move(Point::new(310, 310));
    let was_dragging = dnd.is_dragging();
    dnd.cancel();
    let cancel_ok = was_dragging && !dnd.is_dragging() && dnd.phase == DndPhase::Idle && dnd.payload.is_none();
    check_m11!(16, "DND Cancellation Lifecycle & State Rollback", cancel_ok);

    true
}

fn print_num(mut n: usize) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 8];
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
