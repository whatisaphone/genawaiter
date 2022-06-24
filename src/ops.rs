use std::pin::Pin;

/// A trait implemented for generator types.
///
/// This is modeled after the stdlib's nightly-only [`core::ops::Generator`].
pub trait Generator<R = ()> {
    /// The type of value this generator yields.
    type Yield;

    /// The type of value this generator returns upon completion.
    type Return;

    /// Resumes the execution of this generator.
    fn resume(self: Pin<&mut Self>, arg: R) -> GeneratorState<Self::Yield, Self::Return>;
}

/// The result of a generator resumption.
///
/// This is modeled after the stdlib's nightly-only
/// [`core::ops::GeneratorState`].
///
/// This enum is returned from the [`Generator::resume`] method and indicates
/// the possible return values of a generator. Currently this corresponds to
/// either a suspension point (Yielded) or a termination point (Complete).
#[derive(PartialEq, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum GeneratorState<Y, R> {
    /// The generator suspended with a value.
    ///
    /// This state indicates that a generator has been suspended, and typically
    /// corresponds to a yield statement. The value provided in this variant
    /// corresponds to the expression passed to yield and allows generators to
    /// provide a value each time they yield.
    Yielded(Y),

    /// The generator completed with a return value.
    ///
    /// This state indicates that a generator has finished execution with the
    /// provided value. Once a generator has returned Complete it is considered
    /// a programmer error to call resume again.
    Complete(R),
}
