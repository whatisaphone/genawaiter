use crate::{
    ops::{Coroutine, GeneratorState},
    stack::engine::{advance, Airlock, Co, Next},
};
use std::{cell::UnsafeCell, future::Future, mem, pin::Pin, ptr};

/// This is a generator which stores all its state without any allocation.
///
/// _See the module-level docs for more details._
pub struct Gen<Y, R, F: Future> {
    state: State<Y, R, F>,
}

struct State<Y, R, F: Future> {
    airlock: Airlock<Y, R>,
    future: F,
}

impl<Y, R, F: Future> Gen<Y, R, F> {
    // These lifetimes are not quite right, but I think it's the closest we cae get.
    // See https://users.rust-lang.org/t/hrtb-on-multiple-generics/34255.
    #[doc(hidden)]
    pub unsafe fn __macro_internal_popuate<'y>(
        this: &mut mem::MaybeUninit<Self>,
        start: impl FnOnce(Co<'y, Y, R>) -> F,
    ) where
        Y: 'y,
        R: 'y,
    {
        let p = &mut (*this.as_mut_ptr()).state as *mut State<Y, R, F>;

        let airlock = UnsafeCell::new(Next::Empty);
        ptr::write(&mut (*p).airlock, airlock);

        let future = start(Co {
            airlock: &(*p).airlock,
        });
        ptr::write(&mut (*p).future, future);
    }

    /// Resumes execution of the generator.
    ///
    /// The argument will become the output of the future returned from
    /// [`Co::yield_`][stack::Co::yield].
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    pub fn resume_with(self: Pin<&mut Self>, arg: R) -> GeneratorState<Y, F::Output> {
        Coroutine::resume_with(self, arg)
    }
}

impl<Y, R, F: Future> Drop for Gen<Y, R, F> {
    fn drop(&mut self) {
        let state: *mut State<Y, R, F> = &self.state as *const _ as *mut _;
        unsafe {
            ptr::drop_in_place(&mut (*state).future);
            ptr::drop_in_place(&mut (*state).airlock);
        }
    }
}

impl<Y, F: Future> Gen<Y, (), F> {
    /// Resumes execution of the generator.
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    pub fn resume(self: Pin<&mut Self>) -> GeneratorState<Y, F::Output> {
        Coroutine::resume_with(self, ())
    }
}

impl<Y, R, F: Future> Coroutine for Gen<Y, R, F> {
    type Yield = Y;
    type Resume = R;
    type Return = F::Output;

    fn resume_with(
        self: Pin<&mut Self>,
        arg: R,
    ) -> GeneratorState<Self::Yield, Self::Return> {
        let (future, airlock);
        unsafe {
            // Safety: Do not move out of the reference.
            let state = &mut self.get_unchecked_mut().state;
            // Safety: Do not move out of the reference.
            future = Pin::new_unchecked(&mut state.future);
            airlock = &state.airlock;
        }
        advance(future, airlock, arg)
    }
}
