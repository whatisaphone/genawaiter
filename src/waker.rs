use core::{
    ptr,
    task::{RawWaker, RawWakerVTable, Waker},
};

pub fn create() -> Waker {
    // Safety: The waker points to a vtable with functions that do nothing. Doing
    // nothing is memory-safe.
    unsafe { Waker::from_raw(RAW_WAKER) }
}

const RAW_WAKER: RawWaker = RawWaker::new(ptr::null(), &VTABLE);
const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(_: *const ()) -> RawWaker {
    RAW_WAKER
}

unsafe fn wake(_: *const ()) {}

unsafe fn wake_by_ref(_: *const ()) {}

unsafe fn drop(_: *const ()) {}
