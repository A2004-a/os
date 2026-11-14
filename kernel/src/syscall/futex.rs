//! NovaOS Apex — Fast Userspace Mutex (Futex) Subsystem (M12)
//! Provides kernel-assisted synchronization primitives for userspace mutexes, condition variables,
//! and semaphores with atomic wait/wake operations and zero uncontended overhead.

extern crate alloc;
use alloc::vec::Vec;
use crate::scheduler::sync::SpinLock;
use crate::scheduler::thread::{ThreadId, ThreadState};
use crate::scheduler::{get_current_pid, get_current_tid, set_thread_state};
use crate::syscall::validate::validate_user_buffer;

pub const FUTEX_WAIT: u32 = 0;
pub const FUTEX_WAKE: u32 = 1;
pub const FUTEX_REQUEUE: u32 = 2;
pub const FUTEX_CMP_REQUEUE: u32 = 3;

pub const FUTEX_BUCKETS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FutexWaiter {
    pub tid: ThreadId,
    pub pid: u32,
    pub uaddr: u64,
}

pub struct FutexManager {
    buckets: [Vec<FutexWaiter>; FUTEX_BUCKETS],
}

impl FutexManager {
    pub fn new() -> Self {
        // Initialize 64 empty buckets
        let buckets: [Vec<FutexWaiter>; FUTEX_BUCKETS] = core::array::from_fn(|_| Vec::new());
        Self { buckets }
    }

    fn hash(pid: u32, uaddr: u64) -> usize {
        let combined = (pid as u64) ^ (uaddr >> 2);
        (combined % (FUTEX_BUCKETS as u64)) as usize
    }

    pub fn wait(&mut self, pid: u32, tid: ThreadId, uaddr: u64, expected: u32) -> Result<i64, i32> {
        // 1. Natural 4-byte alignment check
        if (uaddr & 0x03) != 0 {
            return Err(-22); // EINVAL
        }

        // 2. Canonical user address validation
        if uaddr >= 0x0000_8000_0000_0000 {
            return Err(-14); // EFAULT
        }

        if let Err(_) = validate_user_buffer(uaddr, 4, false) {
            return Err(-14); // EFAULT
        }

        // 3. Atomically check value
        // SAFETY: Pointer is verified to be within canonical userspace and mapped
        let current_val = unsafe { core::ptr::read_volatile(uaddr as *const u32) };
        if current_val != expected {
            return Err(-11); // EAGAIN
        }

        // 4. Enqueue waiter into hash bucket
        let bucket_idx = Self::hash(pid, uaddr);
        self.buckets[bucket_idx].push(FutexWaiter { tid, pid, uaddr });

        // 5. Suspend thread
        let _ = set_thread_state(tid, ThreadState::Blocked);
        Ok(0)
    }

    pub fn wake(&mut self, pid: u32, uaddr: u64, count: u32) -> Result<i64, i32> {
        if (uaddr & 0x03) != 0 {
            return Err(-22); // EINVAL
        }
        if uaddr >= 0x0000_8000_0000_0000 {
            return Err(-14); // EFAULT
        }

        let bucket_idx = Self::hash(pid, uaddr);
        let bucket = &mut self.buckets[bucket_idx];
        let mut woken = 0u32;
        let mut i = 0;

        while i < bucket.len() && woken < count {
            if bucket[i].pid == pid && bucket[i].uaddr == uaddr {
                let waiter = bucket.remove(i);
                let _ = set_thread_state(waiter.tid, ThreadState::Ready);
                woken += 1;
            } else {
                i += 1;
            }
        }

        Ok(woken as i64)
    }

    pub fn requeue(&mut self, pid: u32, uaddr: u64, count: u32, uaddr2: u64, count2: u32) -> Result<i64, i32> {
        if (uaddr & 0x03) != 0 || (uaddr2 & 0x03) != 0 {
            return Err(-22); // EINVAL
        }
        if uaddr >= 0x0000_8000_0000_0000 || uaddr2 >= 0x0000_8000_0000_0000 {
            return Err(-14); // EFAULT
        }

        let b1 = Self::hash(pid, uaddr);
        let b2 = Self::hash(pid, uaddr2);

        let mut woken = 0u32;
        let mut requeued = 0u32;

        let mut to_requeue = Vec::new();

        {
            let bucket = &mut self.buckets[b1];
            let mut i = 0;
            while i < bucket.len() {
                if bucket[i].pid == pid && bucket[i].uaddr == uaddr {
                    if woken < count {
                        let waiter = bucket.remove(i);
                        let _ = set_thread_state(waiter.tid, ThreadState::Ready);
                        woken += 1;
                    } else if requeued < count2 {
                        let mut waiter = bucket.remove(i);
                        waiter.uaddr = uaddr2;
                        to_requeue.push(waiter);
                        requeued += 1;
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
        }

        for w in to_requeue {
            self.buckets[b2].push(w);
        }

        Ok(woken as i64)
    }

    pub fn waiting_count(&self, pid: u32, uaddr: u64) -> usize {
        let b = Self::hash(pid, uaddr);
        self.buckets[b].iter().filter(|w| w.pid == pid && w.uaddr == uaddr).count()
    }
}

pub static FUTEX: SpinLock<FutexManager> = SpinLock::new("futex", FutexManager {
    buckets: [const { Vec::new() }; FUTEX_BUCKETS],
});

pub fn sys_futex(uaddr: u64, op: u32, val: u32, _timeout_ptr: u64, uaddr2: u64, val3: u32) -> Result<i64, i32> {
    let current_tid = get_current_tid();
    let current_pid = get_current_pid();

    let mut futex = FUTEX.lock();
    let cmd = op & 0x7F;

    match cmd {
        FUTEX_WAIT => futex.wait(current_pid, current_tid, uaddr, val),
        FUTEX_WAKE => futex.wake(current_pid, uaddr, val),
        FUTEX_REQUEUE | FUTEX_CMP_REQUEUE => futex.requeue(current_pid, uaddr, val, uaddr2, val3),
        _ => Err(-38), // ENOSYS
    }
}
