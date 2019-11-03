use std::pin::Pin;

pub trait Generator {
    type Yield;
    type Return;
    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return>;
}

#[cfg_attr(test, derive(PartialEq, Debug))]
#[allow(clippy::module_name_repetitions)]
pub enum GeneratorState<Y, R> {
    Yielded(Y),
    Complete(R),
}
