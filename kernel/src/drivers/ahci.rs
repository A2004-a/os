//! NovaOS Zero — Serial ATA Advanced Host Controller Interface (AHCI 1.3) Driver
//! Class 01h (Mass Storage), Subclass 06h (SATA Controller), ProgIF 01h (AHCI)

pub const PCI_CLASS_STORAGE: u8   = 0x01;
pub const PCI_SUBCLASS_SATA: u8   = 0x06;
pub const PCI_PROGIF_AHCI: u8     = 0x01;

pub const FIS_TYPE_REG_H2D: u8    = 0x27;
pub const ATA_CMD_READ_DMA_EX: u8 = 0x25;
pub const ATA_CMD_WRITE_DMA_EX: u8 = 0x35;
pub const ATA_CMD_IDENTIFY: u8    = 0xEC;

pub const AHCI_PORT_CMD_ST: u32   = 1 << 0;  // Start processing command list
pub const AHCI_PORT_CMD_FRE: u32  = 1 << 4;  // FIS Receive Enable
pub const AHCI_PORT_CMD_FR: u32   = 1 << 14; // FIS Receive Running
pub const AHCI_PORT_CMD_CR: u32   = 1 << 15; // Command List Running

/// Host to Device Register FIS (20 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FisRegH2D {
    pub fis_type: u8,        // 0x27
    pub pm_port_c: u8,       // Bit 7: 1=Command, 0=Control; Bits 3..0: PM Port
    pub command: u8,         // Command register
    pub features_low: u8,    // Feature register (7..0)
    pub lba0: u8,            // LBA low (7..0)
    pub lba1: u8,            // LBA mid (15..8)
    pub lba2: u8,            // LBA high (23..16)
    pub device: u8,          // Device register (1 << 6 for LBA mode)
    pub lba3: u8,            // LBA (31..24)
    pub lba4: u8,            // LBA (39..32)
    pub lba5: u8,            // LBA (47..40)
    pub features_high: u8,   // Feature register (15..8)
    pub count_low: u8,       // Sector count (7..0)
    pub count_high: u8,      // Sector count (15..8)
    pub icc: u8,             // Isochronous command completion
    pub control: u8,         // Control register
    pub reserved: [u8; 4],
}

impl FisRegH2D {
    pub fn new_read_dma(lba: u64, count: u16) -> Self {
        FisRegH2D {
            fis_type: FIS_TYPE_REG_H2D,
            pm_port_c: 0x80, // Command bit set
            command: ATA_CMD_READ_DMA_EX,
            features_low: 0,
            lba0: (lba & 0xFF) as u8,
            lba1: ((lba >> 8) & 0xFF) as u8,
            lba2: ((lba >> 16) & 0xFF) as u8,
            device: 1 << 6, // LBA mode
            lba3: ((lba >> 24) & 0xFF) as u8,
            lba4: ((lba >> 32) & 0xFF) as u8,
            lba5: ((lba >> 40) & 0xFF) as u8,
            features_high: 0,
            count_low: (count & 0xFF) as u8,
            count_high: ((count >> 8) & 0xFF) as u8,
            icc: 0,
            control: 0,
            reserved: [0; 4],
        }
    }

    pub fn new_write_dma(lba: u64, count: u16) -> Self {
        let mut fis = Self::new_read_dma(lba, count);
        fis.command = ATA_CMD_WRITE_DMA_EX;
        fis
    }
}

/// Physical Region Descriptor Table Entry (PRDT, 16 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrdtEntry {
    pub data_base_address: u64,
    pub reserved: u32,
    pub byte_count_ioc: u32, // Bit 31: Interrupt On Completion; Bits 21..0: Byte count - 1
}

impl PrdtEntry {
    pub fn new(phys_addr: u64, byte_count: u32, ioc: bool) -> Self {
        let count_field = (byte_count.saturating_sub(1) & 0x003F_FFFF) | if ioc { 1 << 31 } else { 0 };
        PrdtEntry {
            data_base_address: phys_addr,
            reserved: 0,
            byte_count_ioc: count_field,
        }
    }

    pub fn get_data_base_address(&self) -> u64 { self.data_base_address }
    pub fn get_byte_count_ioc(&self) -> u32 { self.byte_count_ioc }
}

/// AHCI Command Header (32 bytes)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandHeader {
    pub flags: u16,          // CFL (Command FIS Length), Write, Prefetchable
    pub prdt_len: u16,       // Number of PRDT entries
    pub prdb_count: u32,     // Transferred byte count
    pub command_table_base: u64, // 128-byte aligned command table base
    pub reserved: [u32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AhciPort {
    pub port_no: usize,
    pub is_connected: bool,
    pub sector_count: u64,
}

pub struct AhciController {
    pub mmio_base: u64,
    pub ports_implemented: u32,
    pub ports: [AhciPort; 8],
}

impl AhciController {
    pub fn new(mmio_base: u64, ports_implemented: u32) -> Self {
        let mut ports = [AhciPort { port_no: 0, is_connected: false, sector_count: 0 }; 8];
        for i in 0..8 {
            ports[i].port_no = i;
            if (ports_implemented & (1 << i)) != 0 {
                ports[i].is_connected = true;
                ports[i].sector_count = 1048576; // Default virtual 512 MiB
            }
        }

        AhciController {
            mmio_base,
            ports_implemented,
            ports,
        }
    }

    pub fn port_active_count(&self) -> usize {
        self.ports.iter().filter(|p| p.is_connected).count()
    }
}
