use std::{
    ptr,
    task::{RawWaker, RawWakerVTable, Waker},
};

pub fn create() -> Waker {
    unsafe { Waker::from_raw(RAW_WAKER) }
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    /* clone */ |_| panic!(),
    /* wake */ |_| {},
    /* wake_by_ref */ |_| {},
    /* drop */ |_| {},
);

const RAW_WAKER: RawWaker = RawWaker::new(ptr::null(), &VTABLE);
