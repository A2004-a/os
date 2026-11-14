//! NovaOS Zero — NVM Express (NVMe 1.4) Storage Controller Driver
//! Class 01h (Mass Storage), Subclass 08h (Non-Volatile Memory), ProgIF 02h (NVM Express)

pub const PCI_CLASS_STORAGE: u8   = 0x01;
pub const PCI_SUBCLASS_NVME: u8   = 0x08;
pub const PCI_PROGIF_NVME: u8     = 0x02;

pub const NVME_OP_FLUSH: u8       = 0x00;
pub const NVME_OP_WRITE: u8       = 0x01;
pub const NVME_OP_READ: u8        = 0x02;
pub const NVME_ADMIN_IDENTIFY: u8 = 0x06;

pub const NVME_QUEUE_CAPACITY: usize = 32;

/// Standard NVMe 64-byte Submission Queue Entry (Command)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvmeCommand {
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub nsid: u32,
    pub reserved0: u64,
    pub mptr: u64,
    pub dptr_prp1: u64,
    pub dptr_prp2: u64,
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

impl NvmeCommand {
    pub fn empty() -> Self {
        NvmeCommand {
            opcode: 0,
            flags: 0,
            command_id: 0,
            nsid: 0,
            reserved0: 0,
            mptr: 0,
            dptr_prp1: 0,
            dptr_prp2: 0,
            cdw10: 0,
            cdw11: 0,
            cdw12: 0,
            cdw13: 0,
            cdw14: 0,
            cdw15: 0,
        }
    }

    pub fn new_read(cid: u16, nsid: u32, start_lba: u64, block_count: u16, prp1: u64) -> Self {
        NvmeCommand {
            opcode: NVME_OP_READ,
            flags: 0,
            command_id: cid,
            nsid,
            reserved0: 0,
            mptr: 0,
            dptr_prp1: prp1,
            dptr_prp2: 0,
            cdw10: (start_lba & 0xFFFF_FFFF) as u32,
            cdw11: ((start_lba >> 32) & 0xFFFF_FFFF) as u32,
            cdw12: (block_count.saturating_sub(1) & 0xFFFF) as u32,
            cdw13: 0,
            cdw14: 0,
            cdw15: 0,
        }
    }

    pub fn new_write(cid: u16, nsid: u32, start_lba: u64, block_count: u16, prp1: u64) -> Self {
        let mut cmd = Self::new_read(cid, nsid, start_lba, block_count, prp1);
        cmd.opcode = NVME_OP_WRITE;
        cmd
    }

    pub fn get_nsid(&self) -> u32 { self.nsid }
    pub fn get_cdw10(&self) -> u32 { self.cdw10 }
    pub fn get_cdw12(&self) -> u32 { self.cdw12 }
    pub fn get_dptr_prp1(&self) -> u64 { self.dptr_prp1 }
}

/// Standard NVMe 16-byte Completion Queue Entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvmeCompletion {
    pub command_specific: u32,
    pub reserved: u32,
    pub sq_head: u16,
    pub sq_id: u16,
    pub command_id: u16,
    pub status: u16, // Bit 0: Phase bit, Bits 15..1: Status code
}

impl NvmeCompletion {
    pub fn empty() -> Self {
        NvmeCompletion {
            command_specific: 0,
            reserved: 0,
            sq_head: 0,
            sq_id: 0,
            command_id: 0,
            status: 0,
        }
    }

    pub fn is_success(&self) -> bool {
        // Status code 0 indicates command executed successfully
        ((self.status >> 1) & 0x7FFF) == 0
    }
}

pub struct NvmeController {
    pub mmio_base: u64,
    pub next_cid: u16,
    pub sq_tail: u16,
    pub cq_head: u16,
    pub sq_entries: [NvmeCommand; NVME_QUEUE_CAPACITY],
    pub cq_entries: [NvmeCompletion; NVME_QUEUE_CAPACITY],
}

impl NvmeController {
    pub fn new(mmio_base: u64) -> Self {
        NvmeController {
            mmio_base,
            next_cid: 1,
            sq_tail: 0,
            cq_head: 0,
            sq_entries: [NvmeCommand::empty(); NVME_QUEUE_CAPACITY],
            cq_entries: [NvmeCompletion::empty(); NVME_QUEUE_CAPACITY],
        }
    }

    /// Calculate MMIO offset for Submission/Completion Queue doorbell
    pub fn calculate_doorbell_offset(queue_id: u16, is_cq: bool, dstrd: u8) -> u64 {
        let doorbell_stride = 4 << dstrd;
        let base_doorbell_offset = 0x1000u64;
        let index = (queue_id as u64 * 2) + if is_cq { 1 } else { 0 };
        base_doorbell_offset + (index * doorbell_stride)
    }

    /// Submit a command to the Submission Queue and advance tail
    pub fn submit_command(&mut self, mut cmd: NvmeCommand) -> u16 {
        let cid = self.next_cid;
        cmd.command_id = cid;
        self.next_cid = self.next_cid.wrapping_add(1);

        let tail = self.sq_tail as usize;
        self.sq_entries[tail] = cmd;
        self.sq_tail = (self.sq_tail + 1) % (NVME_QUEUE_CAPACITY as u16);
        cid
    }
}
