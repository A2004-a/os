//! NovaOS Apex — Storage & Partitioning Subsystem
//! Implements GUID Partition Table (GPT), CRC32 validation, Protective MBR, and partition isolation.

pub mod crc32;
pub mod gpt;
pub mod partition_device;
