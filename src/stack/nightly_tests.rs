// These tests can't be parsed on non-nightly compilers, so move them to a
// separate file.

use crate::ops::GeneratorState;

#[test]
fn async_closure() {
    unsafe_create_generator!(gen, async move |co| {
        co.yield_(10).await;
        "done"
    });
    assert_eq!(gen.as_mut().resume(), GeneratorState::Yielded(10));
    assert_eq!(gen.as_mut().resume(), GeneratorState::Complete("done"));
}
