//! NovaOS Apex — Memory Statistics and Telemetry Reporting

use super::frame::get_frame_stats;
use super::heap::get_heap_stats;
use crate::serial;

#[derive(Clone, Copy, Debug)]
pub struct SystemMemoryStats {
    pub phys_total_bytes: u64,
    pub phys_usable_bytes: u64,
    pub phys_allocated_bytes: u64,
    pub phys_free_bytes: u64,
    pub heap_total_bytes: usize,
    pub heap_allocated_bytes: usize,
    pub heap_free_bytes: usize,
    pub heap_peak_bytes: usize,
    pub heap_alloc_count: usize,
    pub heap_dealloc_count: usize,
}

pub fn get_system_stats() -> SystemMemoryStats {
    let frame_stats = get_frame_stats();
    let heap_stats = get_heap_stats();

    SystemMemoryStats {
        phys_total_bytes: frame_stats.total_bytes,
        phys_usable_bytes: frame_stats.usable_bytes,
        phys_allocated_bytes: frame_stats.allocated_bytes,
        phys_free_bytes: frame_stats.free_bytes,
        heap_total_bytes: heap_stats.total_bytes,
        heap_allocated_bytes: heap_stats.allocated_bytes,
        heap_free_bytes: heap_stats.free_bytes,
        heap_peak_bytes: heap_stats.peak_allocated_bytes,
        heap_alloc_count: heap_stats.allocation_count,
        heap_dealloc_count: heap_stats.deallocation_count,
    }
}

pub fn print_summary() {
    let stats = get_system_stats();
    serial::write_str("\n=======================================================\n");
    serial::write_str("   NovaOS Apex — Kernel Memory Subsystem Telemetry     \n");
    serial::write_str("=======================================================\n");
    serial::write_str("  Physical Memory:\n");
    serial::write_str("    Total Tracked:    "); print_u64(stats.phys_total_bytes / (1024 * 1024)); serial::write_str(" MiB\n");
    serial::write_str("    Usable RAM:       "); print_u64(stats.phys_usable_bytes / (1024 * 1024)); serial::write_str(" MiB\n");
    serial::write_str("    Allocated Frames: "); print_u64(stats.phys_allocated_bytes / 1024); serial::write_str(" KiB\n");
    serial::write_str("    Free Frames:      "); print_u64(stats.phys_free_bytes / (1024 * 1024)); serial::write_str(" MiB\n");
    serial::write_str("  Kernel Heap:\n");
    serial::write_str("    Total Buffer:     "); print_u64((stats.heap_total_bytes / 1024) as u64); serial::write_str(" KiB\n");
    serial::write_str("    Allocated Bytes:  "); print_u64(stats.heap_allocated_bytes as u64); serial::write_str(" bytes\n");
    serial::write_str("    Free Bytes:       "); print_u64(stats.heap_free_bytes as u64); serial::write_str(" bytes\n");
    serial::write_str("    Peak Allocation:  "); print_u64(stats.heap_peak_bytes as u64); serial::write_str(" bytes\n");
    serial::write_str("    Alloc/Dealloc:    "); print_u64(stats.heap_alloc_count as u64); serial::write_str(" / "); print_u64(stats.heap_dealloc_count as u64); serial::write_str("\n");
    serial::write_str("=======================================================\n");
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
