// These tests can't be parsed on non-nightly compilers, so move them to a
// separate file.

use crate::{ops::GeneratorState, rc::Gen};

#[test]
fn async_closure() {
    let mut gen = Gen::new(async move |mut co| {
        co.yield_(10).await;
        "done"
    });
    assert_eq!(gen.resume(), GeneratorState::Yielded(10));
    assert_eq!(gen.resume(), GeneratorState::Complete("done"));
}
