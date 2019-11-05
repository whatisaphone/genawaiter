use std::pin::Pin;

/// A trait implemented for generator types.
///
/// This is modeled after the stdlib's nightly-only [`std::ops::Generator`].
pub trait Generator {
    /// The type of value this generator yields.
    type Yield;

    /// The type of value this generator returns.
    type Return;

    /// Resumes the execution of this generator.
    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return>;
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
