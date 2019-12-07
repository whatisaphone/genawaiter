// These tests can't be parsed on non-nightly compilers, so move them to a
// separate file.

use crate::{
    ops::GeneratorState,
    stack::{yielder_cls, Shelf},
    yield_,
};

#[test]
fn async_closure() {
    generator_mut!(gen, async move |co| {
        co.yield_(10).await;
        "done"
    });
    assert_eq!(gen.resume(), GeneratorState::Yielded(10));
    assert_eq!(gen.resume(), GeneratorState::Complete("done"));
}

#[test]
fn stack__closure() {
    let mut shelf = Shelf::new();
    #[yielder_cls(u8)]
    let gen = unsafe {
        Gen::new(&mut shelf, async move || {
            let mut n = 1_u8;
            while n < 10 {
                yield_! { n };
                n += 2;
            }
        })
    };
}
