// Copyright 2016 Amanieu d'Antras
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::cmp;
use core::hint;
use core::num::Wrapping;
use core::ops;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

use bytemuck::NoUninit;

// We use an AtomicUsize instead of an AtomicBool because it performs better
// on architectures that don't have byte-sized atomics.
//
// We give each spinlock its own cache line to avoid false sharing.
#[repr(align(64))]
struct SpinLock(AtomicUsize);

impl SpinLock {
    fn lock(&self, order: Ordering) {
        // If the corresponding atomic operation is `SeqCst`, acquire the lock
        // with `SeqCst` ordering to ensure sequential consistency.
        let success_order = match order {
            Ordering::SeqCst => Ordering::SeqCst,
            _ => Ordering::Acquire,
        };
        while self
            .0
            .compare_exchange_weak(0, 1, success_order, Ordering::Relaxed)
            .is_err()
        {
            while self.0.load(Ordering::Relaxed) != 0 {
                hint::spin_loop();
            }
        }
    }

    fn unlock(&self, order: Ordering) {
        self.0.store(
            0,
            // As with acquiring the lock, release the lock with `SeqCst`
            // ordering if the corresponding atomic operation was `SeqCst`.
            match order {
                Ordering::SeqCst => Ordering::SeqCst,
                _ => Ordering::Release,
            },
        );
    }
}

// A big array of spinlocks which we use to guard atomic accesses. A spinlock is
// chosen based on a hash of the address of the atomic object, which helps to
// reduce contention compared to a single global lock.
macro_rules! array {
    (@accum (0, $($_es:expr),*) -> ($($body:tt)*))
        => {array!(@as_expr [$($body)*])};
    (@accum (1, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (0, $($es),*) -> ($($body)* $($es,)*))};
    (@accum (2, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (0, $($es),*) -> ($($body)* $($es,)* $($es,)*))};
    (@accum (4, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (2, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (8, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (16, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (8, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (32, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (16, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (64, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (32, $($es,)* $($es),*) -> ($($body)*))};

    (@as_expr $e:expr) => {$e};

    [$e:expr; $n:tt] => { array!(@accum ($n, $e) -> ()) };
}
static SPINLOCKS: [SpinLock; 64] = array![SpinLock(AtomicUsize::new(0)); 64];

// Spinlock pointer hashing function from compiler-rt
#[inline]
fn lock_for_addr(addr: usize) -> &'static SpinLock {
    // Disregard the lowest 4 bits.  We want all values that may be part of the
    // same memory operation to hash to the same value and therefore use the same
    // lock.
    let mut hash = addr >> 4;
    // Use the next bits as the basis for the hash
    let low = hash & (SPINLOCKS.len() - 1);
    // Now use the high(er) set of bits to perturb the hash, so that we don't
    // get collisions from atomic fields in a single object
    hash >>= 16;
    hash ^= low;
    // Return a pointer to the lock to use
    &SPINLOCKS[hash & (SPINLOCKS.len() - 1)]
}

#[inline]
fn lock(addr: usize, order: Ordering) -> LockGuard {
    let lock = lock_for_addr(addr);
    lock.lock(order);
    LockGuard {
        lock,
        order,
    }
}

struct LockGuard {
    lock: &'static SpinLock,
    /// The ordering of the atomic operation for which the lock was obtained.
    order: Ordering,
}

impl Drop for LockGuard {
    #[inline]
    fn drop(&mut self) {
        self.lock.unlock(self.order);
    }
}

#[inline]
pub unsafe fn atomic_load<T>(dst: *mut T, order: Ordering) -> T {
    let _l = lock(dst as usize, order);
    ptr::read(dst)
}

#[inline]
pub unsafe fn atomic_store<T>(dst: *mut T, val: T, order: Ordering) {
    let _l = lock(dst as usize, order);
    ptr::write(dst, val);
}

#[inline]
pub unsafe fn atomic_swap<T>(dst: *mut T, val: T, order: Ordering) -> T {
    let _l = lock(dst as usize, order);
    ptr::replace(dst, val)
}

#[inline]
pub unsafe fn atomic_compare_exchange<T: NoUninit>(
    dst: *mut T,
    current: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
    let mut l = lock(dst as usize, success);
    let result = ptr::read(dst);
    // compare_exchange compares with memcmp instead of Eq
    let a = bytemuck::bytes_of(&result);
    let b = bytemuck::bytes_of(&current);
    if a == b {
        ptr::write(dst, new);
        Ok(result)
    } else {
        // Use the failure ordering instead in this case.
        l.order = failure;
        Err(result)
    }
}

#[inline]
pub unsafe fn atomic_add<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
where
    Wrapping<T>: ops::Add<Output = Wrapping<T>>,
{
    let _l = lock(dst as usize, order);
    let result = ptr::read(dst);
    ptr::write(dst, (Wrapping(result) + Wrapping(val)).0);
    result
}

#[inline]
pub unsafe fn atomic_sub<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
where
    Wrapping<T>: ops::Sub<Output = Wrapping<T>>,
{
    let _l = lock(dst as usize, order);
    let result = ptr::read(dst);
    ptr::write(dst, (Wrapping(result) - Wrapping(val)).0);
    result
}

#[inline]
pub unsafe fn atomic_and<T: Copy + ops::BitAnd<Output = T>>(dst: *mut T, val: T, order: Ordering) -> T {
    let _l = lock(dst as usize, order);
    let result = ptr::read(dst);
    ptr::write(dst, result & val);
    result
}

#[inline]
pub unsafe fn atomic_or<T: Copy + ops::BitOr<Output = T>>(dst: *mut T, val: T, order: Ordering) -> T {
    let _l = lock(dst as usize, order);
    let result = ptr::read(dst);
    ptr::write(dst, result | val);
    result
}

#[inline]
pub unsafe fn atomic_xor<T: Copy + ops::BitXor<Output = T>>(dst: *mut T, val: T, order: Ordering) -> T {
    let _l = lock(dst as usize, order);
    let result = ptr::read(dst);
    ptr::write(dst, result ^ val);
    result
}

#[inline]
pub unsafe fn atomic_min<T: Copy + cmp::Ord>(dst: *mut T, val: T, order: Ordering) -> T {
    let _l = lock(dst as usize, order);
    let result = ptr::read(dst);
    ptr::write(dst, cmp::min(result, val));
    result
}

#[inline]
pub unsafe fn atomic_max<T: Copy + cmp::Ord>(dst: *mut T, val: T, order: Ordering) -> T {
    let _l = lock(dst as usize, order);
    let result = ptr::read(dst);
    ptr::write(dst, cmp::max(result, val));
    result
}
