use std::pin::Pin;

/// A trait implemented for coroutines.
///
/// A `Coroutine` is a generalization of a `Generator`. A `Generator` constrains
/// the resume argument type to `()`, but in a `Coroutine` it can be anything.
pub trait Coroutine {
    /// The type of value this generator yields.
    type Yield;

    /// The type of value this generator accepts as a resume argument.
    type Resume;

    /// The type of value this generator returns upon completion.
    type Return;

    /// Resumes the execution of this generator.
    ///
    /// The argument will be passed into the coroutine as a resume argument.
    fn resume_with(
        self: Pin<&mut Self>,
        arg: Self::Resume,
    ) -> GeneratorState<Self::Yield, Self::Return>;
}

/// A trait implemented for generator types.
///
/// This is modeled after the stdlib's nightly-only [`std::ops::Generator`].
pub trait Generator {
    /// The type of value this generator yields.
    type Yield;

    /// The type of value this generator returns upon completion.
    type Return;

    /// Resumes the execution of this generator.
    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return>;
}

impl<C: Coroutine<Resume = ()>> Generator for C {
    type Yield = <Self as Coroutine>::Yield;
    type Return = <Self as Coroutine>::Return;

    #[must_use]
    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        self.resume_with(())
    }
}

/// The result of a generator resumption.
///
/// This is modeled after the stdlib's nightly-only
/// [`std::ops::GeneratorState`].
#[derive(PartialEq, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum GeneratorState<Y, R> {
    /// The generator suspended with a value.
    Yielded(Y),

    /// The generator completed with a return value.
    Complete(R),
}
