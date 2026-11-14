//! NovaOS Apex — Network Subsystem
//! Combines Ethernet II, ARP, IPv4, ICMP, UDP, TCP, BSD socket table, and verification suite.

pub mod arp;
pub mod ethernet;
pub mod icmp;
pub mod ipv4;
pub mod mac;
pub mod socket;
pub mod stack;
pub mod tcp;
pub mod tests;
pub mod udp;

use crate::serial;

pub fn init() {
    serial::write_str("[KERNEL] Initializing Network Subsystem...\n");
    stack::init_stack();
}
