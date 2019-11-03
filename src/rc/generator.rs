use crate::{
    ops::{Generator, GeneratorState},
    rc::{
        engine::{advance, Airlock},
        Co,
    },
};
use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc};

/// This is a generator which stores its state on the heap.
///
/// _See the module-level docs for more details._
pub struct Gen<Y, F: Future> {
    airlock: Airlock<Y>,
    future: Pin<Box<F>>,
}

impl<Y, F: Future> Gen<Y, F> {
    /// Creates a new generator from a function.
    ///
    /// The function accepts a [`Co`] object, and returns a future. Every time
    /// the generator is resumed, the future is polled. Each time the future is
    /// polled, it should do one of two things:
    ///
    /// - Call `Co::yield_()`, and then return `Poll::Pending`.
    /// - Drop the `Co`, and then return `Poll::Ready`.
    ///
    /// Typically this exchange will happen in the context of an `async fn`.
    ///
    /// _See the module-level docs for more details._
    pub fn new(start: impl FnOnce(Co<Y>) -> F) -> Self {
        let airlock = Rc::new(RefCell::new(None));
        let future = {
            let airlock = airlock.clone();
            Box::pin(start(Co { airlock }))
        };
        Self { airlock, future }
    }

    /// Resumes execution of the generator.
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    pub fn resume(&mut self) -> GeneratorState<Y, F::Output> {
        advance(self.future.as_mut(), &self.airlock)
    }
}

impl<Y, F: Future> Generator for Gen<Y, F> {
    type Yield = Y;
    type Return = F::Output;

    fn resume(mut self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        let this: &mut Self = &mut *self;
        advance(this.future.as_mut(), &this.airlock)
    }
}
