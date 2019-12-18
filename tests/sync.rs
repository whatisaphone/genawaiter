#![cfg_attr(feature = "nightly", feature(async_await, async_closure))]
#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(missing_docs, clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use genawaiter::sync::{Co, Gen};

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

#[cfg(feature = "proc_macro")]
#[test]
fn sync_proc_macro_fn() {
    #[genawaiter::sync::sync_yield_fn(u8)]
    async fn odds() {
        for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
            genawaiter::yield_!(n);
        }
    }
    let gen = Gen::new(odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[cfg(feature = "nightly")]
#[test]
fn sync_proc_macro_closure() {
    use genawaiter::sync_yield_cls;

    let gen = Gen::new(sync_yield_cls!(
        u8 in async move || {
            let mut n = 1_u8;
            while n < 10 {
                genawaiter::yield_!(n);
                n += 2;
            }
        }
    ));
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn sync_proc_macro_fn_method_call() {
    #[genawaiter::sync::sync_yield_fn(u8)]
    async fn odds() {
        for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
            let _cloned_resume_arg = genawaiter::yield_!(n).clone();
        }
    }
    let gen = genawaiter::sync::Gen::new(odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}
