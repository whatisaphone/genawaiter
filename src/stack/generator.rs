use crate::{
    ops::{Generator, GeneratorState},
    stack::engine::{advance, Airlock, Co},
};
use std::{cell::UnsafeCell, future::Future, mem, pin::Pin, ptr};

/// This is a generator which stores all its state without any allocation.
///
/// _See the module-level docs for more details._
pub struct Gen<Y, F: Future> {
    state: State<Y, F>,
}

struct State<Y, F: Future> {
    airlock: Airlock<Y>,
    future: F,
}

impl<Y, F: Future> Gen<Y, F> {
    #[doc(hidden)]
    pub unsafe fn __macro_internal_popuate<'y>(
        this: &mut mem::MaybeUninit<Self>,
        start: impl FnOnce(Co<'y, Y>) -> F,
    ) where
        Y: 'y,
    {
        let p = &mut (*this.as_mut_ptr()).state as *mut State<Y, F>;

        let airlock = UnsafeCell::new(None);
        ptr::write(&mut (*p).airlock, airlock);

        let future = start(Co {
            airlock: &(*p).airlock,
        });
        ptr::write(&mut (*p).future, future);
    }

    /// Resumes execution of the generator.
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    pub fn resume(self: Pin<&mut Self>) -> GeneratorState<Y, F::Output> {
        Generator::resume(self)
    }
}

impl<Y, F: Future> Drop for Gen<Y, F> {
    fn drop(&mut self) {
        let state: *mut State<Y, F> = &self.state as *const _ as *mut _;
        unsafe {
            ptr::drop_in_place(&mut (*state).future);
            ptr::drop_in_place(&mut (*state).airlock);
        }
    }
}

impl<Y, F: Future> Generator for Gen<Y, F> {
    type Yield = Y;
    type Return = F::Output;

    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        let (future, airlock);
        unsafe {
            // Safety: Do not move out of the reference.
            let state = &mut self.get_unchecked_mut().state;
            // Safety: Do not move out of the reference.
            future = Pin::new_unchecked(&mut state.future);
            airlock = &state.airlock;
        }
        advance(future, airlock)
    }
}
