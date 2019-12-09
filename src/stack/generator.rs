use crate::{
    core::{advance, async_advance, Airlock as _, Next},
    ext::MaybeUninitExt,
    ops::{Coroutine, GeneratorState},
    stack::engine::{Airlock, Co},
};
use std::{future::Future, mem, pin::Pin, ptr};

/// This data structure holds the transient state of an executing generator.
///
/// It's called "Shelf", rather than "State", to avoid confusion with the
/// `GeneratorState` enum.
///
/// _See the module-level docs for examples._
// Safety: The lifetime of the data is controlled by a `Gen`, which constructs
// it in place, and holds a mutable reference right up until dropping it in
// place. Thus, the data inside is pinned and can never be moved.
pub struct Shelf<Y, R, F: Future>(mem::MaybeUninit<State<Y, R, F>>);

struct State<Y, R, F: Future> {
    airlock: Airlock<Y, R>,
    future: F,
}

impl<Y, R, F: Future> Shelf<Y, R, F> {
    /// Creates a new, empty `Shelf`.
    ///
    /// _See the module-level docs for examples._
    pub fn new() -> Self {
        Self(mem::MaybeUninit::uninit())
    }
}

impl<Y, R, F: Future> Default for Shelf<Y, R, F> {
    fn default() -> Self {
        Self::new()
    }
}

/// This is a generator which can be stack-allocated.
///
/// _See the module-level docs for examples._
pub struct Gen<'s, Y, R, F: Future> {
    state: Pin<&'s mut State<Y, R, F>>,
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
    /// _See the module-level docs for examples._
    ///
    /// # Safety
    ///
    /// Do not let the `Co` object escape the scope of the generator. By the
    /// time the generator completes, the `Co` object should already have
    /// been dropped. If this invariant is not upheld, memory unsafety will
    /// result.
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
        start: impl FnOnce(Co<'s, Y, R>) -> F,
    ) -> Self {
        // Safety: Build the struct in place, by assigning the fields in order.
        let p = &mut *shelf.0.as_mut_ptr() as *mut State<Y, R, F>;

        let airlock = Airlock::default();
        ptr::write(&mut (*p).airlock, airlock);

        let future = start(Co::new(&(*p).airlock));
        ptr::write(&mut (*p).future, future);

        // Safety: the state can never be moved again, because we store it inside a
        // `Pin` until `Gen::drop`, where the contents are immediately dropped.
        let state = Pin::new_unchecked(shelf.0.assume_init_get_mut());
        Self { state }
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
    /// _See the module-level docs for examples._
    pub fn resume_with(&mut self, arg: R) -> GeneratorState<Y, F::Output> {
        let (future, airlock) = self.project();
        airlock.replace(Next::Resume(arg));
        advance(future, &airlock)
    }

    fn project(&mut self) -> (Pin<&mut F>, &Airlock<Y, R>) {
        unsafe {
            // Safety: `future` is pinned, but never moved. `airlock` is never pinned.
            let state = self.state.as_mut().get_unchecked_mut();

            let future = Pin::new_unchecked(&mut state.future);
            let airlock = &state.airlock;
            (future, airlock)
        }
    }
}

impl<'s, Y, R, F: Future> Drop for Gen<'s, Y, R, F> {
    fn drop(&mut self) {
        // Safety: Drop the struct in place, by dropping the fields in reverse order.
        // Since we drop the fields in place, the `Pin` invariants are not violated.
        unsafe {
            let state = self.state.as_mut().get_unchecked_mut();
            ptr::drop_in_place(&mut state.future);
            ptr::drop_in_place(&mut state.airlock);
        }
    }
}

impl<'s, Y, F: Future> Gen<'s, Y, (), F> {
    /// Resumes execution of the generator.
    ///
    /// If the generator yields a value, `Yielded` is returned. Otherwise,
    /// `Completed` is returned.
    ///
    /// _See the module-level docs for examples._
    pub fn resume(&mut self) -> GeneratorState<Y, F::Output> {
        self.resume_with(())
    }

    /// Resumes execution of the generator.
    ///
    /// If the generator pauses without yielding, `Poll::Pending` is returned.
    /// If the generator yields a value, `Poll::Ready(Yielded)` is returned.
    /// Otherwise, `Poll::Ready(Completed)` is returned.
    ///
    /// _See the module-level docs for examples._
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
