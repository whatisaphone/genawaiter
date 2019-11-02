#[cfg_attr(test, derive(PartialEq, Debug))]
#[allow(clippy::module_name_repetitions)]
pub enum GeneratorState<Y, R> {
    Yielded(Y),
    Complete(R),
}
