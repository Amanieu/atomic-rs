// Copyright 2016 Amanieu d'Antras
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::mem;
use core::num::Wrapping;
use core::ops;
use core::sync::atomic::{AtomicUsize, Ordering};

#[path = "fallback.rs"]
mod fallback;

#[inline]
pub fn atomic_is_lock_free<T>() -> bool {
    mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
}

#[inline]
pub unsafe fn atomic_load<T>(dst: *mut T, order: Ordering) -> T {
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        mem::transmute_copy(&a.load(order))
    } else {
        fallback::atomic_load(dst)
    }
}

#[inline]
pub unsafe fn atomic_store<T>(dst: *mut T, val: T, order: Ordering) {
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        a.store(mem::transmute_copy(&val), order);
    } else {
        fallback::atomic_store(dst, val);
    }
}

#[inline]
pub unsafe fn atomic_swap<T>(dst: *mut T, val: T, order: Ordering) -> T {
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        mem::transmute_copy(&a.swap(mem::transmute_copy(&val), order))
    } else {
        fallback::atomic_swap(dst, val)
    }
}

#[inline]
pub unsafe fn atomic_compare_exchange<T>(
    dst: *mut T,
    current: T,
    new: T,
    success: Ordering,
    _: Ordering,
) -> Result<T, T> {
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        let current_val: usize = mem::transmute_copy(&current);
        let result_val = a.compare_and_swap(current_val, mem::transmute_copy(&new), success);
        if current_val == result_val {
            Ok(mem::transmute_copy(&result_val))
        } else {
            Err(mem::transmute_copy(&result_val))
        }
    } else {
        fallback::atomic_compare_exchange(dst, current, new)
    }
}

#[inline]
pub unsafe fn atomic_compare_exchange_weak<T>(
    dst: *mut T,
    current: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
    atomic_compare_exchange(dst, current, new, success, failure)
}

#[inline]
pub unsafe fn atomic_add<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
where
    Wrapping<T>: ops::Add<Output = Wrapping<T>>,
{
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        mem::transmute_copy(&a.fetch_add(mem::transmute_copy(&val), order))
    } else {
        fallback::atomic_add(dst, val)
    }
}

#[inline]
pub unsafe fn atomic_sub<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
where
    Wrapping<T>: ops::Sub<Output = Wrapping<T>>,
{
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        mem::transmute_copy(&a.fetch_sub(mem::transmute_copy(&val), order))
    } else {
        fallback::atomic_sub(dst, val)
    }
}

#[inline]
pub unsafe fn atomic_and<T: Copy + ops::BitAnd<Output = T>>(
    dst: *mut T,
    val: T,
    order: Ordering,
) -> T {
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        mem::transmute_copy(&a.fetch_and(mem::transmute_copy(&val), order))
    } else {
        fallback::atomic_and(dst, val)
    }
}

#[inline]
pub unsafe fn atomic_or<T: Copy + ops::BitOr<Output = T>>(
    dst: *mut T,
    val: T,
    order: Ordering,
) -> T {
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        mem::transmute_copy(&a.fetch_or(mem::transmute_copy(&val), order))
    } else {
        fallback::atomic_or(dst, val)
    }
}

#[inline]
pub unsafe fn atomic_xor<T: Copy + ops::BitXor<Output = T>>(
    dst: *mut T,
    val: T,
    order: Ordering,
) -> T {
    if mem::size_of::<T>() == mem::size_of::<AtomicUsize>()
        && mem::align_of::<T>() >= mem::size_of::<AtomicUsize>()
    {
        assert_eq!(mem::size_of::<AtomicUsize>(), mem::size_of::<usize>());
        let a = &*(dst as *const AtomicUsize);
        mem::transmute_copy(&a.fetch_xor(mem::transmute_copy(&val), order))
    } else {
        fallback::atomic_xor(dst, val)
    }
}
