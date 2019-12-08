#![cfg_attr(feature = "nightly", feature(async_await, async_closure))]
#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(missing_docs, clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use genawaiter::{
    generator_mut,
    stack::{Co, Gen, Shelf},
};

async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
    for n in (1..).step_by(2).take_while(|&n| n < 10) {
        co.yield_(n).await;
    }
}

#[test]
fn test_basic() {
    generator_mut!(gen, odd_numbers_less_than_ten);

    let xs: Vec<_> = gen.into_iter().collect();
    assert_eq!(xs, [1, 3, 5, 7, 9]);
}

#[test]
fn test_shelf() {
    let mut shelf = Shelf::new();
    let gen = unsafe { Gen::new(&mut shelf, odd_numbers_less_than_ten) };

    let xs: Vec<_> = gen.into_iter().collect();
    assert_eq!(xs, [1, 3, 5, 7, 9]);
}

#[cfg(feature = "futures03")]
#[test]
fn test_stream() {
    use futures::executor::block_on_stream;

    generator_mut!(gen, odd_numbers_less_than_ten);
    let xs: Vec<_> = block_on_stream(gen).collect();
    assert_eq!(xs, [1, 3, 5, 7, 9]);
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_proc_macro_fn() {
    #[genawaiter::stack::stack_yield_fn(u8)]
    async fn odds() {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            genawaiter::yield_!(n);
        }
    }
    generator_mut!(gen, odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}
