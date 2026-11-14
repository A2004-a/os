//! NovaOS Zero — Milestone M5 Automated Test Suite
//! Validates NovaFS Formatting, Inodes, Directory Entries, VFS Path Resolution, and Multi-Block I/O.

use crate::storage::RamBlockDevice;
use crate::fs::novafs::{NovaFs, NOVA_MAGIC, NOVA_VERSION, BLOCK_SIZE};
use crate::fs::vfs::{FileOperations, FsError, NodeType, VFS};
use crate::serial_println;

pub fn run_fs_diagnostics() {
    serial_println!("\n=======================================================");
    serial_println!("  NovaOS Zero — Milestone M5 Automated Verification");
    serial_println!("=======================================================");

    test_novafs_format_and_superblock();
    test_file_creation_and_rw();
    test_multiblock_file_write_and_read();
    test_directory_hierarchy_and_lookup();
    test_vfs_open_read_write_close();

    serial_println!("[M5 SUCCESS] All Milestone M5 Verification Tests Passed Cleanly!\n");
}

fn test_novafs_format_and_superblock() {
    serial_println!("[TEST 1/5] NovaFS Formatting & Superblock Signature...");

    let ramdisk = RamBlockDevice::new();
    let fs = NovaFs::format(ramdisk).expect("Formatting NovaFS ramdisk failed");

    let magic = fs.superblock.magic;
    assert_eq!(magic, NOVA_MAGIC);
    let version = fs.superblock.version;
    assert_eq!(version, NOVA_VERSION);
    let block_size = fs.superblock.block_size as usize;
    assert_eq!(block_size, BLOCK_SIZE);
    let total_blocks = fs.superblock.total_blocks;
    assert_eq!(total_blocks, 512);
    let free_blocks = fs.superblock.free_blocks;
    assert!(free_blocks > 400);

    serial_println!("  -> NovaFS superblock signature 'NOVA' (0x4E4F5641) verified.");
}

fn test_file_creation_and_rw() {
    serial_println!("[TEST 2/5] Inode Allocation, File Creation & Byte Parity...");

    let ramdisk = RamBlockDevice::new();
    let mut fs = NovaFs::format(ramdisk).expect("Formatting failed");

    // Inode 1 is root directory. Create file "readme.txt" in root
    let file_inode = fs.create(1, "readme.txt", NodeType::File)
        .expect("Failed to create file");
    assert!(file_inode >= 2);

    let test_data = b"NovaOS Bare-Metal Microkernel v0.5.0\nRunning on x86_64 native paging.\n";
    let written = fs.write(file_inode, 0, test_data).expect("Failed to write data");
    assert_eq!(written, test_data.len());

    let mut read_buf = [0u8; 128];
    let read_bytes = fs.read(file_inode, 0, &mut read_buf).expect("Failed to read file");
    assert_eq!(read_bytes, test_data.len());
    assert_eq!(&read_buf[..read_bytes], test_data);

    let meta = fs.get_metadata(file_inode).expect("Failed to get metadata");
    assert_eq!(meta.size as usize, test_data.len());

    serial_println!("  -> File creation, write, and exact byte readback verified.");
}

fn test_multiblock_file_write_and_read() {
    serial_println!("[TEST 3/5] Multi-Block Spanning I/O (1200 Bytes)...");

    let ramdisk = RamBlockDevice::new();
    let mut fs = NovaFs::format(ramdisk).expect("Formatting failed");

    let file_inode = fs.create(1, "payload.bin", NodeType::File).expect("Failed to create file");

    // 1200 bytes spans 3 distinct 512-byte blocks (512 + 512 + 176)
    let mut payload = [0u8; 1200];
    for i in 0..1200 {
        payload[i] = (i % 251) as u8;
    }

    let written = fs.write(file_inode, 0, &payload).expect("Failed to write large payload");
    assert_eq!(written, 1200);

    let mut verify_buf = [0u8; 1200];
    let read_bytes = fs.read(file_inode, 0, &mut verify_buf).expect("Failed to read large payload");
    assert_eq!(read_bytes, 1200);
    assert_eq!(&verify_buf[..], &payload[..]);

    let meta = fs.get_metadata(file_inode).expect("Failed to get metadata");
    assert_eq!(meta.blocks, 3); // 3 blocks allocated

    serial_println!("  -> Multi-block allocation and seamless boundary read verified.");
}

fn test_directory_hierarchy_and_lookup() {
    serial_println!("[TEST 4/5] Directory Tree & Hierarchical Path Resolution...");

    let ramdisk = RamBlockDevice::new();
    let mut fs = NovaFs::format(ramdisk).expect("Formatting failed");

    // Create /etc directory
    let etc_inode = fs.create(1, "etc", NodeType::Directory).expect("Failed to create /etc");
    assert!(etc_inode >= 2);

    // Create /etc/hostname file
    let hostname_inode = fs.create(etc_inode, "hostname", NodeType::File).expect("Failed to create /etc/hostname");
    let name_bytes = b"novaos-box\n";
    fs.write(hostname_inode, 0, name_bytes).expect("Failed to write hostname");

    // Test directory lookups
    let looked_up_etc = fs.lookup(1, "etc").expect("Lookup of 'etc' failed");
    assert_eq!(looked_up_etc, etc_inode);

    let looked_up_host = fs.lookup(etc_inode, "hostname").expect("Lookup of 'hostname' failed");
    assert_eq!(looked_up_host, hostname_inode);

    // Test non-existent entry
    assert_eq!(fs.lookup(etc_inode, "shadow"), Err(FsError::NotFound));

    serial_println!("  -> Directory creation and nested child lookup verified.");
}

fn test_vfs_open_read_write_close() {
    serial_println!("[TEST 5/5] VFS Global Mount & File Handle Lifecycle...");

    static mut STATIC_FS: Option<NovaFs<RamBlockDevice>> = None;

    unsafe {
        let disk = RamBlockDevice::new();
        let fs = NovaFs::format(disk).expect("Format failed");
        *core::ptr::addr_of_mut!(STATIC_FS) = Some(fs);

        let fs_ref = (*core::ptr::addr_of_mut!(STATIC_FS)).as_mut().unwrap();
        // Populate a file in root
        let doc_inode = fs_ref.create(1, "welcome.txt", NodeType::File).unwrap();
        fs_ref.write(doc_inode, 0, b"Welcome to NovaOS VFS!").unwrap();

        let mut vfs = VFS.lock();
        vfs.mount_root(fs_ref);

        // 1. Resolve path
        let inode = vfs.resolve_path("/welcome.txt").expect("VFS resolve failed");
        assert_eq!(inode, doc_inode);

        // 2. Open file read-only
        let fd = vfs.open("/welcome.txt", false).expect("VFS open failed");

        // 3. Read through VFS descriptor
        let mut buf = [0u8; 64];
        let bytes = vfs.read(fd, &mut buf).expect("VFS read failed");
        assert_eq!(&buf[..bytes], b"Welcome to NovaOS VFS!");

        // 4. Writing to read-only descriptor must fail
        assert_eq!(vfs.write(fd, b"Deny"), Err(FsError::PermissionDenied));

        // 5. Close descriptor
        vfs.close(fd).expect("VFS close failed");

        // 6. Accessing closed descriptor must fail
        assert_eq!(vfs.read(fd, &mut buf), Err(FsError::InvalidHandle));
    }

    serial_println!("  -> VFS open/read/write/close handle lifecycle verified.");
}
