use std::{
    future::Future,
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr,
};

use crate::{
    core::{advance, async_advance, Airlock as _, Next},
    ops::{Coroutine, GeneratorState},
    stack::engine::{Airlock, Co},
};

/// This data structure holds the transient state of an executing generator.
///
/// It's called "Shelf", rather than "State", to avoid confusion with the
/// `GeneratorState` enum.
///
/// [_See the module-level docs for examples._](.)
pub struct Shelf<Y, R, F: Future> {
    airlock: Airlock<Y, R>,
    future: MaybeUninit<F>,
}

impl<Y, R, F: Future> Shelf<Y, R, F> {
    /// Creates a new, empty `Shelf`.
    ///
    /// [_See the module-level docs for examples._](.)
    #[must_use]
    pub fn new() -> Self {
        Self {
            airlock: Airlock::default(),
            // Safety: The lifetime of the data is controlled by a `Gen`, which constructs
            // it in place, and holds a mutable reference right up until dropping it in
            // place. Thus, the data inside is pinned and can never be moved.
            future: MaybeUninit::uninit(),
        }
    }
}

impl<Y, R, F: Future> Default for Shelf<Y, R, F> {
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

/// This is a generator which can be stack-allocated.
///
/// [_See the module-level docs for examples._](.)
pub struct Gen<'s, Y, R, F: Future> {
    airlock: &'s Airlock<Y, R>,
    future: ManuallyDrop<Pin<&'s mut F>>,
}

impl<'s, Y, R, F: Future> Gen<'s, Y, R, F> {
    /// Creates a new generator from a function.
    ///
    /// The state of the generator is stored in `shelf`, which will be pinned in
    /// place while this generator exists. The generator itself is movable,
    /// since it just holds a reference to the pinned state.
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
    /// # Safety
    ///
    /// The `Co` object must not outlive the returned `Gen`. By time the
    /// generator completes (i.e., by time the producer's Future returns
    /// `Poll::Ready`), the `Co` object should already have been dropped. If
    /// this invariant is not upheld, memory unsafety can result.
    ///
    /// Afaik, the Rust compiler [is not flexible enough][hrtb-thread] to let
    /// you express this invariant in the type system, but I would love to be
    /// proven wrong!
    ///
    /// [hrtb-thread]: https://users.rust-lang.org/t/hrtb-on-multiple-generics/34255
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use genawaiter::stack::{Co, Gen, Shelf};
    /// #
    /// # async fn producer(co: Co<'_, i32>) { /* ... */ }
    /// #
    /// let mut shelf = Shelf::new();
    /// let gen = unsafe { Gen::new(&mut shelf, producer) };
    /// ```
    pub unsafe fn new(
        shelf: &'s mut Shelf<Y, R, F>,
        producer: impl FnOnce(Co<'s, Y, R>) -> F,
    ) -> Self {
        // By splitting the mutable `shelf` into a shared `airlock` and a unique
        // pinned `future` reference we ensure the aliasing rules are not violated.
        let airlock = &shelf.airlock;
        // Safety: Initializes the future in-place using `ptr::write`, which is
        // the correct way to initialize a `MaybeUninit`
        shelf.future.as_mut_ptr().write(producer(Co::new(airlock)));
        // Safety: The `MaybeUninit` is initialized by now, so its safe to create
        // a reference to the future itself
        // todo: can be replaced by `MaybeUninit::get_mut` once stabilized
        let init = &mut *shelf.future.as_mut_ptr();
        // Safety: The `shelf` remains borrowed during the entire lifetime of
        // the `Gen`and is hence pinned.
        let future = ManuallyDrop::new(Pin::new_unchecked(init));

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
    pub fn resume_with(&mut self, arg: R) -> GeneratorState<Y, F::Output> {
        self.airlock.replace(Next::Resume(arg));
        advance(self.future.as_mut(), &self.airlock)
    }
}

impl<'s, Y, R, F: Future> Drop for Gen<'s, Y, R, F> {
    fn drop(&mut self) {
        // Safety: `future` itself is a `MaybeUninit`, which is guaranteed to be
        // initialized, because the only way to construct a `Gen` is with
        // `Gen::new`, which initializes it.
        //
        // The pinned reference to the initialized future is wrapped in a
        // `ManuallyDrop` because `Pin::get_unchecked_mut` consumes the `Pin`,
        // which would require moving it (the pin) out of the `Gen` first.
        //
        // Drop `future` in place first (it likely contains a reference to airlock).
        // Since we drop it in place, the `Pin` invariants are not violated.
        // The airlock is regularly dropped when the `Shelf` goes out of scope.
        unsafe {
            // todo: can be replaced by `ManuallyDrop::take`, stabilized in 1.42
            let future = ptr::read(&*self.future);
            ptr::drop_in_place(future.get_unchecked_mut());
        }
    }
}

impl<'s, Y, F: Future> Gen<'s, Y, (), F> {
    /// Resumes execution of the generator.
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    ///
    /// [_See the module-level docs for examples._](.)
    pub fn resume(&mut self) -> GeneratorState<Y, F::Output> {
        self.resume_with(())
    }

    /// Resumes execution of the generator.
    ///
    /// If the generator pauses without yielding, `Poll::Pending` is returned.
    /// If the generator yields a value, `Poll::Ready(Yielded)` is returned.
    /// Otherwise, `Poll::Ready(Completed)` is returned.
    ///
    /// [_See the module-level docs for examples._](.)
    pub fn async_resume(
        &mut self,
    ) -> impl Future<Output = GeneratorState<Y, F::Output>> + '_ {
        self.airlock.replace(Next::Resume(()));
        async_advance(self.future.as_mut(), self.airlock)
    }
}

impl<'s, Y, R, F: Future> Coroutine for Gen<'s, Y, R, F> {
    type Yield = Y;
    type Resume = R;
    type Return = F::Output;

    fn resume_with(
        self: Pin<&mut Self>,
        arg: R,
    ) -> GeneratorState<Self::Yield, Self::Return> {
        // Safety: `Gen::resume_with` does not move `self`.
        let this = unsafe { self.get_unchecked_mut() };
        this.resume_with(arg)
    }
}
