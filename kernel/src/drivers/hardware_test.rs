//! NovaOS Zero — Milestone M11 Automated Verification Suite
//! Validates AHCI SATA 1.3, NVMe 1.4, USB 3.0 xHCI & HID, and Intel HD Audio controller logic.

use crate::drivers::ahci::{
    AhciController, FisRegH2D, PrdtEntry, ATA_CMD_READ_DMA_EX, ATA_CMD_WRITE_DMA_EX,
    FIS_TYPE_REG_H2D, PCI_CLASS_STORAGE as AHCI_CLASS_STORAGE, PCI_SUBCLASS_SATA,
};
use crate::drivers::audio::{
    IntelHdaController, PCI_CLASS_MULTIMEDIA, PCI_SUBCLASS_HDA,
};
use crate::drivers::nvme::{
    NvmeCommand, NvmeCompletion, NvmeController, NVME_OP_READ, NVME_OP_WRITE,
    PCI_CLASS_STORAGE as NVME_CLASS_STORAGE, PCI_SUBCLASS_NVME,
};
use crate::drivers::xhci::{
    TransferRequestBlock, UsbHidKeyboardReport, UsbHidMouseReport, XhciController,
    PCI_CLASS_SERIAL, PCI_PROGIF_XHCI, PCI_SUBCLASS_USB, TRB_TYPE_NORMAL,
};
use crate::serial_println;

pub fn run_hardware_diagnostics() {
    serial_println!("\n=======================================================");
    serial_println!("  NovaOS Zero — Milestone M11 Automated Verification");
    serial_println!("=======================================================");

    test_ahci_sata_controller();
    test_nvme_storage_controller();
    test_usb_xhci_and_hid();
    test_intel_hd_audio();
    test_pci_class_code_definitions();

    serial_println!("[M11 SUCCESS] All Milestone M11 Verification Tests Passed Cleanly!\n");
}

fn test_ahci_sata_controller() {
    serial_println!("[TEST 1/5] AHCI 1.3 SATA Controller & FIS/PRDT Construction...");

    let ahci = AhciController::new(0xFEA0_0000, 0x05); // Ports 0 and 2 active
    assert_eq!(ahci.port_active_count(), 2);
    assert!(ahci.ports[0].is_connected);
    assert!(!ahci.ports[1].is_connected);
    assert!(ahci.ports[2].is_connected);

    // Test Read DMA FIS
    let read_fis = FisRegH2D::new_read_dma(1024, 8);
    assert_eq!(read_fis.fis_type, FIS_TYPE_REG_H2D);
    assert_eq!(read_fis.command, ATA_CMD_READ_DMA_EX);
    assert_eq!(read_fis.lba0, 0);
    assert_eq!(read_fis.lba1, 4); // 1024 = 0x0400 -> lba1 = 0x04
    assert_eq!(read_fis.count_low, 8);
    assert_eq!(read_fis.device & (1 << 6), 1 << 6); // LBA mode bit

    // Test Write DMA FIS
    let write_fis = FisRegH2D::new_write_dma(2048, 16);
    assert_eq!(write_fis.command, ATA_CMD_WRITE_DMA_EX);
    assert_eq!(write_fis.count_low, 16);

    // Test PRDT Entry
    let prdt = PrdtEntry::new(0x1000_0000, 4096, true);
    assert_eq!(prdt.get_data_base_address(), 0x1000_0000);
    assert_eq!(prdt.get_byte_count_ioc() & (1 << 31), 1 << 31); // IOC set
    assert_eq!(prdt.get_byte_count_ioc() & 0x003F_FFFF, 4095);

    serial_println!("  -> AHCI port masks, DMA FIS packets, and PRDT entries verified.");
}

fn test_nvme_storage_controller() {
    serial_println!("[TEST 2/5] NVMe 1.4 Controller, Doorbell Registers & Submission Ring...");

    let mut nvme = NvmeController::new(0xFEC0_0000);

    // Test Doorbell Offset Calculation
    assert_eq!(NvmeController::calculate_doorbell_offset(0, false, 0), 0x1000); // Admin SQ
    assert_eq!(NvmeController::calculate_doorbell_offset(0, true, 0), 0x1004);  // Admin CQ
    assert_eq!(NvmeController::calculate_doorbell_offset(1, false, 0), 0x1008); // I/O SQ 1
    assert_eq!(NvmeController::calculate_doorbell_offset(1, true, 0), 0x100C);  // I/O CQ 1

    // Test Read Command Construction
    let read_cmd = NvmeCommand::new_read(1, 1, 4096, 8, 0x2000_0000);
    assert_eq!(read_cmd.opcode, NVME_OP_READ);
    assert_eq!(read_cmd.get_nsid(), 1);
    assert_eq!(read_cmd.get_cdw10(), 4096);
    assert_eq!(read_cmd.get_cdw12(), 7); // 8 sectors - 1
    assert_eq!(read_cmd.get_dptr_prp1(), 0x2000_0000);

    // Test Write Command Construction
    let write_cmd = NvmeCommand::new_write(2, 1, 8192, 16, 0x3000_0000);
    assert_eq!(write_cmd.opcode, NVME_OP_WRITE);
    assert_eq!(write_cmd.get_cdw10(), 8192);

    // Submit command to ring
    let cid = nvme.submit_command(read_cmd);
    assert_eq!(cid, 1);
    assert_eq!(nvme.sq_tail, 1);

    // Test Completion Queue Status
    let comp = NvmeCompletion {
        command_specific: 0,
        reserved: 0,
        sq_head: 1,
        sq_id: 0,
        command_id: 1,
        status: 0,
    };
    assert!(comp.is_success());

    serial_println!("  -> NVMe queue submission, doorbell stride, and completion parsing verified.");
}

fn test_usb_xhci_and_hid() {
    serial_println!("[TEST 3/5] USB 3.0 xHCI Host Controller & USB HID Scancodes...");

    let mut xhci = XhciController::new(0xFE80_0000, 16, 4);

    // Test TRB Construction
    let trb = TransferRequestBlock::new(0x1234_5678_9ABC_DEF0, 4096, TRB_TYPE_NORMAL, true);
    assert_eq!(trb.get_param_low(), 0x9ABC_DEF0);
    assert_eq!(trb.get_param_high(), 0x1234_5678);
    assert_eq!(trb.get_trb_type(), TRB_TYPE_NORMAL);
    assert!(trb.get_cycle_bit());

    // Enqueue TRB into ring
    let slot = xhci.enqueue_trb(trb);
    assert_eq!(slot, 0);
    assert_eq!(xhci.ring_index, 1);

    // Test USB HID Keyboard Scancode Decoding
    assert_eq!(XhciController::decode_hid_scancode(0x04, false), Some('a'));
    assert_eq!(XhciController::decode_hid_scancode(0x04, true), Some('A'));
    assert_eq!(XhciController::decode_hid_scancode(0x1E, false), Some('1'));
    assert_eq!(XhciController::decode_hid_scancode(0x1E, true), Some('!'));
    assert_eq!(XhciController::decode_hid_scancode(0x28, false), Some('\n'));
    assert_eq!(XhciController::decode_hid_scancode(0x2C, false), Some(' '));

    // Test USB HID Mouse Report
    let mouse = UsbHidMouseReport::new(1, 15, -10, 1);
    assert_eq!(mouse.buttons, 1);
    assert_eq!(mouse.delta_x, 15);
    assert_eq!(mouse.delta_y, -10);

    let kbd = UsbHidKeyboardReport::new(0x02, [0x04, 0, 0, 0, 0, 0]); // Left Shift + 'a'
    assert!(kbd.is_shift_pressed());
    assert!(!kbd.is_ctrl_pressed());

    serial_println!("  -> xHCI ring cycle bits, TRB dispatch, and USB HID reports verified.");
}

fn test_intel_hd_audio() {
    serial_println!("[TEST 4/5] Intel High Definition Audio (HDA) PCM Stream Pipeline...");

    let mut hda = IntelHdaController::new(0xFEB0_0000);
    assert!(hda.is_ready);

    // Format 1: 48 kHz, 16-bit, 2 Channels (Stereo)
    let fmt_48k = IntelHdaController::encode_format(48000, 2, 16).expect("Encode 48k format failed");
    assert_eq!(fmt_48k, 0x0011); // Base 48k (0), 16-bit (0x10), 2 chan (0x01)

    // Format 2: 44.1 kHz, 24-bit, 2 Channels (Stereo)
    let fmt_44k = IntelHdaController::encode_format(44100, 2, 24).expect("Encode 44k format failed");
    assert_eq!(fmt_44k, 0x4031); // Base 44k (0x4000), 24-bit (0x30), 2 chan (0x01)

    // Configure Stream 0
    let stream_fmt = hda.configure_stream(0, 48000, 2, 16).expect("Configure stream failed");
    assert_eq!(stream_fmt, 0x0011);
    assert_eq!(hda.streams[0].format_tag, 0x0011);

    hda.set_stream_running(0, true);
    assert!(hda.streams[0].is_running);

    hda.set_stream_running(0, false);
    assert!(!hda.streams[0].is_running);

    serial_println!("  -> HDA PCM format encoding, multi-channel tags, and stream DMA states verified.");
}

fn test_pci_class_code_definitions() {
    serial_println!("[TEST 5/5] PCI Class Code Mapping & Controller Discovery...");

    assert_eq!(AHCI_CLASS_STORAGE, 0x01);
    assert_eq!(PCI_SUBCLASS_SATA, 0x06);

    assert_eq!(NVME_CLASS_STORAGE, 0x01);
    assert_eq!(PCI_SUBCLASS_NVME, 0x08);

    assert_eq!(PCI_CLASS_SERIAL, 0x0C);
    assert_eq!(PCI_SUBCLASS_USB, 0x03);
    assert_eq!(PCI_PROGIF_XHCI, 0x30);

    assert_eq!(PCI_CLASS_MULTIMEDIA, 0x04);
    assert_eq!(PCI_SUBCLASS_HDA, 0x03);

    serial_println!("  -> PCI Class/Subclass/ProgIF constants match PCI-SIG specification.");
}
