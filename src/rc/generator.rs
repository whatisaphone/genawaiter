use crate::{
    core::{advance, async_advance, Airlock as _, Next},
    ops::{Generator, GeneratorState},
    rc::{engine::Airlock, Co},
};
use std::{future::Future, pin::Pin};

/// This is a generator which stores its state on the heap.
///
/// [_See the module-level docs for examples._](.)
pub struct Gen<Y, R, F: Future> {
    airlock: Airlock<Y, R>,
    future: Pin<Box<F>>,
}

impl<Y, R, F: Future> Gen<Y, R, F> {
    /// Creates a new generator from a function.
    ///
    /// The function accepts a [`Co`] object, and returns a future. Every time
    /// the generator is resumed, the future is polled. Each time the future is
    /// polled, it should do one of two things:
    ///
    /// - Call `co.yield_()`, and then return `Poll::Pending`.
    /// - Drop the `Co`, and then return `Poll::Ready`.
    ///
    /// Typically this exchange will happen in the context of an `async fn`.
    ///
    /// [_See the module-level docs for examples._](.)
    pub fn new(producer: impl FnOnce(Co<Y, R>) -> F) -> Self {
        let airlock = Airlock::default();
        let future = { Box::pin(producer(Co::new(airlock.clone()))) };
        Self { airlock, future }
    }

    /// Resumes execution of the generator.
    ///
    /// `arg` is the resume argument. If the generator was previously paused by
    /// awaiting a future returned from `co.yield()`, that future will complete,
    /// and return `arg`.
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    ///
    /// [_See the module-level docs for examples._](.)
    pub fn resume(&mut self, arg: R) -> GeneratorState<Y, F::Output> {
        self.airlock.replace(Next::Resume(arg));
        advance(self.future.as_mut(), &self.airlock)
    }
}

impl<Y, R, F: Future> Gen<Y, R, F> {
    /// Resumes execution of the generator.
    ///
    /// If the generator pauses without yielding, `Poll::Pending` is returned.
    /// If the generator yields a value, `Poll::Ready(Yielded)` is returned.
    /// Otherwise, `Poll::Ready(Completed)` is returned.
    ///
    /// [_See the module-level docs for examples._](.)
    pub fn async_resume(
        &mut self,
        arg: R,
    ) -> impl Future<Output = GeneratorState<Y, F::Output>> + '_ {
        self.airlock.replace(Next::Resume(arg));
        async_advance(self.future.as_mut(), self.airlock.clone())
    }
}

impl<Y, R, F: Future> Generator<R> for Gen<Y, R, F> {
    type Yield = Y;
    type Return = F::Output;

    fn resume(
        mut self: Pin<&mut Self>,
        arg: R,
    ) -> GeneratorState<Self::Yield, Self::Return> {
        Self::resume(&mut *self, arg)
    }
}
