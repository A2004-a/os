//! NovaOS Apex — Kernel Synchronization Primitives & Deadlock Diagnostics
//! Provides interrupt-safe spinlocks (cli/sti), RAII guards, deadlock tracking, and kernel events.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use crate::arch::x86_64::instructions::{cli, sti};
use crate::serial;

pub const SPIN_DEADLOCK_THRESHOLD: u64 = 10_000_000;

/// Spinlock with interrupt-disabling semantics to prevent deadlocks with interrupt handlers.
pub struct SpinLock<T: ?Sized> {
    lock: AtomicBool,
    name: &'static str,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for SpinLock<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(name: &'static str, data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            name,
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> SpinLock<T> {
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        let rflags = read_and_disable_interrupts();
        let mut spins: u64 = 0;

        while self.lock.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_err() {
            core::hint::spin_loop();
            spins += 1;
            if spins == SPIN_DEADLOCK_THRESHOLD {
                serial::write_str("\n[DEADLOCK WARNING] Potential deadlock detected on spinlock '");
                serial::write_str(self.name);
                serial::write_str("' (spun > 10,000,000 iterations).\n");
            }
        }

        SpinLockGuard {
            lock: &self.lock,
            rflags,
            data: unsafe { &mut *self.data.get() },
        }
    }

    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        let rflags = read_and_disable_interrupts();
        if self.lock.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_ok() {
            Some(SpinLockGuard {
                lock: &self.lock,
                rflags,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            restore_interrupts(rflags);
            None
        }
    }
}

pub struct SpinLockGuard<'a, T: ?Sized> {
    lock: &'a AtomicBool,
    rflags: u64,
    data: &'a mut T,
}

impl<'a, T: ?Sized> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
        restore_interrupts(self.rflags);
    }
}

#[inline(always)]
fn read_and_disable_interrupts() -> u64 {
    let rflags: u64;
    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
        cli();
    }
    rflags
}

#[inline(always)]
fn restore_interrupts(rflags: u64) {
    // If interrupts were previously enabled (bit 9 of RFLAGS), re-enable them
    if (rflags & (1 << 9)) != 0 {
        sti();
    }
}

/// Kernel Event synchronization primitive for wait and signal patterns.
pub struct KernelEvent {
    signaled: AtomicBool,
    name: &'static str,
}

impl KernelEvent {
    pub const fn new(name: &'static str, initial: bool) -> Self {
        Self {
            signaled: AtomicBool::new(initial),
            name,
        }
    }

    pub fn set(&self) {
        self.signaled.store(true, Ordering::Release);
    }

    pub fn reset(&self) {
        self.signaled.store(false, Ordering::Release);
    }

    pub fn is_signaled(&self) -> bool {
        self.signaled.load(Ordering::Acquire)
    }

    pub fn wait(&self) {
        let mut spins: u64 = 0;
        while !self.is_signaled() {
            core::hint::spin_loop();
            spins += 1;
            if spins == SPIN_DEADLOCK_THRESHOLD {
                serial::write_str("\n[EVENT WAIT WARNING] Extended wait on KernelEvent '");
                serial::write_str(self.name);
                serial::write_str("'\n");
                spins = 0;
            }
        }
    }
}
