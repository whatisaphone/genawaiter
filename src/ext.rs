#![allow(clippy::module_name_repetitions)]

use core::mem;

pub trait MaybeUninitExt<T> {
    unsafe fn assume_init_get_mut(&mut self) -> &mut T;
}

impl<T> MaybeUninitExt<T> for mem::MaybeUninit<T> {
    // Copy of unstable method `MaybeUninit::get_mut`.
    unsafe fn assume_init_get_mut(&mut self) -> &mut T {
        self.as_mut_ptr().as_mut().unwrap()
    }
}
