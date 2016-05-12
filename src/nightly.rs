// Copyright 2016 Amanieu d'Antras
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::intrinsics;
use core::mem;
use core::ops;
use core::num::Wrapping;
use core::sync::atomic::Ordering;

mod fallback;

#[inline]
pub fn atomic_is_lock_free<T>() -> bool {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 if mem::align_of::<T>() >= 1 => true,
        #[cfg(target_has_atomic = "16")]
        2 if mem::align_of::<T>() >= 2 => true,
        #[cfg(target_has_atomic = "32")]
        4 if mem::align_of::<T>() >= 4 => true,
        #[cfg(target_has_atomic = "64")]
        8 if mem::align_of::<T>() >= 8 => true,
        _ => false,
    }
}

#[inline]
unsafe fn atomic_load_raw<T>(dst: *mut T, order: Ordering) -> T {
    match order {
        Ordering::Acquire => intrinsics::atomic_load_acq(dst),
        Ordering::Relaxed => intrinsics::atomic_load_relaxed(dst),
        Ordering::SeqCst => intrinsics::atomic_load(dst),
        Ordering::Release => panic!("there is no such thing as a release load"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire/release load"),
    }
}
#[inline]
pub unsafe fn atomic_load<T>(dst: *mut T, order: Ordering) -> T {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 if mem::align_of::<T>() >= 1 => {
            mem::transmute_copy(&atomic_load_raw(dst as *mut u8, order))
        }
        #[cfg(target_has_atomic = "16")]
        2 if mem::align_of::<T>() >= 2 => {
            mem::transmute_copy(&atomic_load_raw(dst as *mut u16, order))
        }
        #[cfg(target_has_atomic = "32")]
        4 if mem::align_of::<T>() >= 4 => {
            mem::transmute_copy(&atomic_load_raw(dst as *mut u32, order))
        }
        #[cfg(target_has_atomic = "64")]
        8 if mem::align_of::<T>() >= 8 => {
            mem::transmute_copy(&atomic_load_raw(dst as *mut u64, order))
        }
        _ => fallback::atomic_load(dst),
    }
}

#[inline]
unsafe fn atomic_store_raw<T>(dst: *mut T, val: T, order: Ordering) {
    match order {
        Ordering::Release => intrinsics::atomic_store_rel(dst, val),
        Ordering::Relaxed => intrinsics::atomic_store_relaxed(dst, val),
        Ordering::SeqCst => intrinsics::atomic_store(dst, val),
        Ordering::Acquire => panic!("there is no such thing as an acquire store"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire/release store"),
    }
}
#[inline]
pub unsafe fn atomic_store<T>(dst: *mut T, val: T, order: Ordering) {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 if mem::align_of::<T>() >= 1 => {
            atomic_store_raw(dst as *mut u8, mem::transmute_copy(&val), order)
        }
        #[cfg(target_has_atomic = "16")]
        2 if mem::align_of::<T>() >= 2 => {
            atomic_store_raw(dst as *mut u16, mem::transmute_copy(&val), order)
        }
        #[cfg(target_has_atomic = "32")]
        4 if mem::align_of::<T>() >= 4 => {
            atomic_store_raw(dst as *mut u32, mem::transmute_copy(&val), order)
        }
        #[cfg(target_has_atomic = "64")]
        8 if mem::align_of::<T>() >= 8 => {
            atomic_store_raw(dst as *mut u64, mem::transmute_copy(&val), order)
        }
        _ => fallback::atomic_store(dst, val),
    }
}

#[inline]
unsafe fn atomic_swap_raw<T>(dst: *mut T, val: T, order: Ordering) -> T {
    match order {
        Ordering::Acquire => intrinsics::atomic_xchg_acq(dst, val),
        Ordering::Release => intrinsics::atomic_xchg_rel(dst, val),
        Ordering::AcqRel => intrinsics::atomic_xchg_acqrel(dst, val),
        Ordering::Relaxed => intrinsics::atomic_xchg_relaxed(dst, val),
        Ordering::SeqCst => intrinsics::atomic_xchg(dst, val),
    }
}
#[inline]
pub unsafe fn atomic_swap<T>(dst: *mut T, val: T, order: Ordering) -> T {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 if mem::align_of::<T>() >= 1 => {
            mem::transmute_copy(&atomic_swap_raw(dst as *mut u8, mem::transmute_copy(&val), order))
        }
        #[cfg(target_has_atomic = "16")]
        2 if mem::align_of::<T>() >= 2 => {
            mem::transmute_copy(&atomic_swap_raw(dst as *mut u16, mem::transmute_copy(&val), order))
        }
        #[cfg(target_has_atomic = "32")]
        4 if mem::align_of::<T>() >= 4 => {
            mem::transmute_copy(&atomic_swap_raw(dst as *mut u32, mem::transmute_copy(&val), order))
        }
        #[cfg(target_has_atomic = "64")]
        8 if mem::align_of::<T>() >= 8 => {
            mem::transmute_copy(&atomic_swap_raw(dst as *mut u64, mem::transmute_copy(&val), order))
        }
        _ => fallback::atomic_swap(dst, val),
    }
}

#[inline]
unsafe fn atomic_compare_exchange_raw<T>(dst: *mut T,
                                         current: T,
                                         new: T,
                                         success: Ordering,
                                         failure: Ordering)
                                         -> Result<T, T> {
    let (val, ok) = match (success, failure) {
        (Ordering::Acquire, Ordering::Acquire) => intrinsics::atomic_cxchg_acq(dst, current, new),
        (Ordering::Release, Ordering::Relaxed) => intrinsics::atomic_cxchg_rel(dst, current, new),
        (Ordering::AcqRel, Ordering::Acquire) => intrinsics::atomic_cxchg_acqrel(dst, current, new),
        (Ordering::Relaxed, Ordering::Relaxed) => {
            intrinsics::atomic_cxchg_relaxed(dst, current, new)
        }
        (Ordering::SeqCst, Ordering::SeqCst) => intrinsics::atomic_cxchg(dst, current, new),
        (Ordering::Acquire, Ordering::Relaxed) => {
            intrinsics::atomic_cxchg_acq_failrelaxed(dst, current, new)
        }
        (Ordering::AcqRel, Ordering::Relaxed) => {
            intrinsics::atomic_cxchg_acqrel_failrelaxed(dst, current, new)
        }
        (Ordering::SeqCst, Ordering::Relaxed) => {
            intrinsics::atomic_cxchg_failrelaxed(dst, current, new)
        }
        (Ordering::SeqCst, Ordering::Acquire) => {
            intrinsics::atomic_cxchg_failacq(dst, current, new)
        }
        (_, Ordering::Release) => {
            panic!("there is no such thing as an acquire/release failure ordering")
        }
        (_, Ordering::AcqRel) => panic!("there is no such thing as a release failure ordering"),
        _ => panic!("a failure ordering can't be stronger than a success ordering"),
    };
    if ok {
        Ok(val)
    } else {
        Err(val)
    }
}
#[inline]
pub unsafe fn atomic_compare_exchange<T>(dst: *mut T,
                                         current: T,
                                         new: T,
                                         success: Ordering,
                                         failure: Ordering)
                                         -> Result<T, T> {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 if mem::align_of::<T>() >= 1 => {
            mem::transmute_copy(&atomic_compare_exchange_raw(dst as *mut u8,
                                                             mem::transmute_copy(&current),
                                                             mem::transmute_copy(&new),
                                                             success,
                                                             failure))
        }
        #[cfg(target_has_atomic = "16")]
        2 if mem::align_of::<T>() >= 2 => {
            mem::transmute_copy(&atomic_compare_exchange_raw(dst as *mut u16,
                                                             mem::transmute_copy(&current),
                                                             mem::transmute_copy(&new),
                                                             success,
                                                             failure))
        }
        #[cfg(target_has_atomic = "32")]
        4 if mem::align_of::<T>() >= 4 => {
            mem::transmute_copy(&atomic_compare_exchange_raw(dst as *mut u32,
                                                             mem::transmute_copy(&current),
                                                             mem::transmute_copy(&new),
                                                             success,
                                                             failure))
        }
        #[cfg(target_has_atomic = "64")]
        8 if mem::align_of::<T>() >= 8 => {
            mem::transmute_copy(&atomic_compare_exchange_raw(dst as *mut u64,
                                                             mem::transmute_copy(&current),
                                                             mem::transmute_copy(&new),
                                                             success,
                                                             failure))
        }
        _ => fallback::atomic_compare_exchange(dst, current, new),
    }
}

#[inline]
unsafe fn atomic_compare_exchange_weak_raw<T>(dst: *mut T,
                                              current: T,
                                              new: T,
                                              success: Ordering,
                                              failure: Ordering)
                                              -> Result<T, T> {
    let (val, ok) = match (success, failure) {
        (Ordering::Acquire, Ordering::Acquire) => {
            intrinsics::atomic_cxchgweak_acq(dst, current, new)
        }
        (Ordering::Release, Ordering::Relaxed) => {
            intrinsics::atomic_cxchgweak_rel(dst, current, new)
        }
        (Ordering::AcqRel, Ordering::Acquire) => {
            intrinsics::atomic_cxchgweak_acqrel(dst, current, new)
        }
        (Ordering::Relaxed, Ordering::Relaxed) => {
            intrinsics::atomic_cxchgweak_relaxed(dst, current, new)
        }
        (Ordering::SeqCst, Ordering::SeqCst) => intrinsics::atomic_cxchgweak(dst, current, new),
        (Ordering::Acquire, Ordering::Relaxed) => {
            intrinsics::atomic_cxchgweak_acq_failrelaxed(dst, current, new)
        }
        (Ordering::AcqRel, Ordering::Relaxed) => {
            intrinsics::atomic_cxchgweak_acqrel_failrelaxed(dst, current, new)
        }
        (Ordering::SeqCst, Ordering::Relaxed) => {
            intrinsics::atomic_cxchgweak_failrelaxed(dst, current, new)
        }
        (Ordering::SeqCst, Ordering::Acquire) => {
            intrinsics::atomic_cxchgweak_failacq(dst, current, new)
        }
        (_, Ordering::Release) => {
            panic!("there is no such thing as an acquire/release failure ordering")
        }
        (_, Ordering::AcqRel) => panic!("there is no such thing as a release failure ordering"),
        _ => panic!("a failure ordering can't be stronger than a success ordering"),
    };
    if ok {
        Ok(val)
    } else {
        Err(val)
    }
}
#[inline]
pub unsafe fn atomic_compare_exchange_weak<T>(dst: *mut T,
                                              current: T,
                                              new: T,
                                              success: Ordering,
                                              failure: Ordering)
                                              -> Result<T, T> {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 if mem::align_of::<T>() >= 1 => {
            mem::transmute_copy(&atomic_compare_exchange_weak_raw(dst as *mut u8,
                                                                  mem::transmute_copy(&current),
                                                                  mem::transmute_copy(&new),
                                                                  success,
                                                                  failure))
        }
        #[cfg(target_has_atomic = "16")]
        2 if mem::align_of::<T>() >= 2 => {
            mem::transmute_copy(&atomic_compare_exchange_weak_raw(dst as *mut u16,
                                                                  mem::transmute_copy(&current),
                                                                  mem::transmute_copy(&new),
                                                                  success,
                                                                  failure))
        }
        #[cfg(target_has_atomic = "32")]
        4 if mem::align_of::<T>() >= 4 => {
            mem::transmute_copy(&atomic_compare_exchange_weak_raw(dst as *mut u32,
                                                                  mem::transmute_copy(&current),
                                                                  mem::transmute_copy(&new),
                                                                  success,
                                                                  failure))
        }
        #[cfg(target_has_atomic = "64")]
        8 if mem::align_of::<T>() >= 8 => {
            mem::transmute_copy(&atomic_compare_exchange_weak_raw(dst as *mut u64,
                                                                  mem::transmute_copy(&current),
                                                                  mem::transmute_copy(&new),
                                                                  success,
                                                                  failure))
        }
        _ => fallback::atomic_compare_exchange(dst, current, new),
    }
}

#[inline]
unsafe fn atomic_add_raw<T>(dst: *mut T, val: T, order: Ordering) -> T {
    match order {
        Ordering::Acquire => intrinsics::atomic_xadd_acq(dst, val),
        Ordering::Release => intrinsics::atomic_xadd_rel(dst, val),
        Ordering::AcqRel => intrinsics::atomic_xadd_acqrel(dst, val),
        Ordering::Relaxed => intrinsics::atomic_xadd_relaxed(dst, val),
        Ordering::SeqCst => intrinsics::atomic_xadd(dst, val),
    }
}
#[inline]
pub unsafe fn atomic_add<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
    where Wrapping<T>: ops::Add<Output = Wrapping<T>>
{
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 => atomic_add_raw(dst, val, order),
        #[cfg(target_has_atomic = "16")]
        2 => atomic_add_raw(dst, val, order),
        #[cfg(target_has_atomic = "32")]
        4 => atomic_add_raw(dst, val, order),
        #[cfg(target_has_atomic = "64")]
        8 => atomic_add_raw(dst, val, order),
        _ => fallback::atomic_add(dst, val),
    }
}

#[inline]
unsafe fn atomic_sub_raw<T>(dst: *mut T, val: T, order: Ordering) -> T {
    match order {
        Ordering::Acquire => intrinsics::atomic_xsub_acq(dst, val),
        Ordering::Release => intrinsics::atomic_xsub_rel(dst, val),
        Ordering::AcqRel => intrinsics::atomic_xsub_acqrel(dst, val),
        Ordering::Relaxed => intrinsics::atomic_xsub_relaxed(dst, val),
        Ordering::SeqCst => intrinsics::atomic_xsub(dst, val),
    }
}
#[inline]
pub unsafe fn atomic_sub<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
    where Wrapping<T>: ops::Sub<Output = Wrapping<T>>
{
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 => atomic_sub_raw(dst, val, order),
        #[cfg(target_has_atomic = "16")]
        2 => atomic_sub_raw(dst, val, order),
        #[cfg(target_has_atomic = "32")]
        4 => atomic_sub_raw(dst, val, order),
        #[cfg(target_has_atomic = "64")]
        8 => atomic_sub_raw(dst, val, order),
        _ => fallback::atomic_sub(dst, val),
    }
}

#[inline]
unsafe fn atomic_and_raw<T>(dst: *mut T, val: T, order: Ordering) -> T {
    match order {
        Ordering::Acquire => intrinsics::atomic_and_acq(dst, val),
        Ordering::Release => intrinsics::atomic_and_rel(dst, val),
        Ordering::AcqRel => intrinsics::atomic_and_acqrel(dst, val),
        Ordering::Relaxed => intrinsics::atomic_and_relaxed(dst, val),
        Ordering::SeqCst => intrinsics::atomic_and(dst, val),
    }
}
#[inline]
pub unsafe fn atomic_and<T: Copy + ops::BitAnd<Output = T>>(dst: *mut T,
                                                            val: T,
                                                            order: Ordering)
                                                            -> T {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 => atomic_and_raw(dst, val, order),
        #[cfg(target_has_atomic = "16")]
        2 => atomic_and_raw(dst, val, order),
        #[cfg(target_has_atomic = "32")]
        4 => atomic_and_raw(dst, val, order),
        #[cfg(target_has_atomic = "64")]
        8 => atomic_and_raw(dst, val, order),
        _ => fallback::atomic_and(dst, val),
    }
}

#[inline]
unsafe fn atomic_or_raw<T>(dst: *mut T, val: T, order: Ordering) -> T {
    match order {
        Ordering::Acquire => intrinsics::atomic_or_acq(dst, val),
        Ordering::Release => intrinsics::atomic_or_rel(dst, val),
        Ordering::AcqRel => intrinsics::atomic_or_acqrel(dst, val),
        Ordering::Relaxed => intrinsics::atomic_or_relaxed(dst, val),
        Ordering::SeqCst => intrinsics::atomic_or(dst, val),
    }
}
#[inline]
pub unsafe fn atomic_or<T: Copy + ops::BitOr<Output = T>>(dst: *mut T,
                                                          val: T,
                                                          order: Ordering)
                                                          -> T {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 => atomic_or_raw(dst, val, order),
        #[cfg(target_has_atomic = "16")]
        2 => atomic_or_raw(dst, val, order),
        #[cfg(target_has_atomic = "32")]
        4 => atomic_or_raw(dst, val, order),
        #[cfg(target_has_atomic = "64")]
        8 => atomic_or_raw(dst, val, order),
        _ => fallback::atomic_or(dst, val),
    }
}

#[inline]
unsafe fn atomic_xor_raw<T>(dst: *mut T, val: T, order: Ordering) -> T {
    match order {
        Ordering::Acquire => intrinsics::atomic_xor_acq(dst, val),
        Ordering::Release => intrinsics::atomic_xor_rel(dst, val),
        Ordering::AcqRel => intrinsics::atomic_xor_acqrel(dst, val),
        Ordering::Relaxed => intrinsics::atomic_xor_relaxed(dst, val),
        Ordering::SeqCst => intrinsics::atomic_xor(dst, val),
    }
}
#[inline]
pub unsafe fn atomic_xor<T: Copy + ops::BitXor<Output = T>>(dst: *mut T,
                                                            val: T,
                                                            order: Ordering)
                                                            -> T {
    match mem::size_of::<T>() {
        #[cfg(target_has_atomic = "8")]
        1 => atomic_xor_raw(dst, val, order),
        #[cfg(target_has_atomic = "16")]
        2 => atomic_xor_raw(dst, val, order),
        #[cfg(target_has_atomic = "32")]
        4 => atomic_xor_raw(dst, val, order),
        #[cfg(target_has_atomic = "64")]
        8 => atomic_xor_raw(dst, val, order),
        _ => fallback::atomic_xor(dst, val),
    }
}
