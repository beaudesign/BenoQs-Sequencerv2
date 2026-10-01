//! A counting global allocator, for spike S2 and the test that `render` allocates nothing
//! (ADR-0008 decision 3). Only built with the `measure` feature, which the shipped module lacks.
//! It forwards every call to `System` and adds one to a thread-local counter on each allocation,
//! as the test allocator in `octoffi` does.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

pub struct CountingAlloc;

thread_local! {
    static ALLOCS: Cell<u32> = const { Cell::new(0) };
}

fn bump() {
    ALLOCS.with(|c| c.set(c.get().wrapping_add(1)));
}

/// Allocations made on this thread since it started. Wraps at `u32::MAX`.
pub fn allocations() -> u32 {
    ALLOCS.with(Cell::get)
}

// SAFETY: every method forwards to `System` with its arguments unchanged, and only bumps a
// thread-local counter, which needs no allocation of its own.
unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        bump();
        System.alloc(l)
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        bump();
        System.alloc_zeroed(l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        bump();
        System.realloc(p, l, n)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        System.dealloc(p, l)
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;
