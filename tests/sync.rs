#![cfg_attr(feature = "nightly", feature(async_await, async_closure, proc_macro_hygiene))]
#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(missing_docs, clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use genawaiter::{sync::{Co, Gen}, yield_, yielder_fn_sync};

async fn odd_numbers_less_than_ten(co: Co<i32>) {
    for n in (1..).step_by(2).take_while(|&n| n < 10) {
        co.yield_(n).await;
    }
}

#[test]
fn test_basic() {
    let gen = Gen::new(odd_numbers_less_than_ten);
    let xs: Vec<_> = gen.into_iter().collect();
    assert_eq!(xs, [1, 3, 5, 7, 9]);
}

#[test]
fn test_boxed() {
    let gen = Gen::new_boxed(odd_numbers_less_than_ten);
    let xs: Vec<_> = gen.into_iter().collect();
    assert_eq!(xs, [1, 3, 5, 7, 9]);
}

#[cfg(feature = "futures03")]
#[test]
fn test_stream() {
    use futures::executor::block_on_stream;

    let gen = Gen::new(odd_numbers_less_than_ten);
    let xs: Vec<_> = block_on_stream(gen).collect();
    assert_eq!(xs, [1, 3, 5, 7, 9]);
}

#[test]
fn sync_proc_macro_fn() {
    #[yielder_fn_sync(u8)]
    async fn odds() {
        for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
            yield_! { n };
        }
    }
    let gen = Gen::new(odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}
