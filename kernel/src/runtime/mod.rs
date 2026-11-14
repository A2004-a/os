//! NovaOS Apex — Embedded Runtimes & Multi-Call Coreutils Subsystem (M14)
//! Houses Busybox-style core utilities, MicroPython scripting, and sandboxed WebAssembly execution.

pub mod coreutils;
pub mod micropython;
pub mod wasm;
pub mod m14_tests;
