use std::{future::Future, mem::MaybeUninit, pin::Pin, ptr};

use crate::{
    core::{advance, async_advance, Airlock as _, Next},
    ops::{Coroutine, GeneratorState},
    stack::engine::{Airlock, Co},
};
use std::cell::UnsafeCell;

/// This data structure holds the transient state of an executing generator.
///
/// It's called "Shelf", rather than "State", to avoid confusion with the
/// `GeneratorState` enum.
///
/// [_See the module-level docs for examples._](.)
pub struct Shelf<Y, R, F: Future>(UnsafeCell<State<Y, R, F>>);

struct State<Y, R, F> {
    airlock: Airlock<Y, R>,
    // Safety: The lifetime of the data is controlled by a `Gen`, which constructs
    // it in place, and holds a mutable reference right up until dropping it in
    // place. Thus, the data inside is pinned and can never be moved.
    future: MaybeUninit<F>,
}

impl<Y, R, F: Future> Shelf<Y, R, F> {
    /// Creates a new, empty `Shelf`.
    ///
    /// [_See the module-level docs for examples._](.)
    #[must_use]
    pub fn new() -> Self {
        Self(UnsafeCell::new(State {
            airlock: Airlock::default(),
            future: MaybeUninit::uninit(),
        }))
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
    state: Pin<&'s mut Shelf<Y, R, F>>,
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
        let state = shelf.0.get();
        let future = producer(Co::new(&(*state).airlock));
        // initializes the future in-place
        (*state).future.as_mut_ptr().write(future);

        Self {
            // Safety: The shelf is borrowed by the resulting `Gen` is hence
            // pinned
            state: Pin::new_unchecked(shelf),
        }
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
        let (future, airlock) = self.project();
        airlock.replace(Next::Resume(arg));
        advance(future, &airlock)
    }

    fn project(&mut self) -> (Pin<&mut F>, &Airlock<Y, R>) {
        unsafe {
            // Safety: This is a pin projection. `future` is pinned, but never moved.
            // `airlock` is never pinned.
            let state = self.state.0.get();

            let future = Pin::new_unchecked(&mut *(*state).future.as_mut_ptr());
            let airlock = &(*state).airlock;
            (future, airlock)
        }
    }
}

impl<'s, Y, R, F: Future> Drop for Gen<'s, Y, R, F> {
    fn drop(&mut self) {
        // Safety: `future` is a `MaybeUninit` which is guaranteed to be initialized,
        // because the only way to construct a `Gen` is with `Gen::new`, which
        // initializes it.
        //
        // Drop `future` in place first (likely contains a reference to airlock),
        // Since we drop it in place, the `Pin` invariants are not violated.
        // The airlock is regularly dropped when the `Shelf` goes out of scope.
        unsafe {
            ptr::drop_in_place((*self.state.0.get()).future.as_mut_ptr());
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
        let (future, airlock) = self.project();
        airlock.replace(Next::Resume(()));
        async_advance(future, airlock)
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
