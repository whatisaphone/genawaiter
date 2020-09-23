#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use genawaiter::rc::{Co, Gen};

async fn odd_numbers_less_than_ten(mut co: Co<i32>) {
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

#[cfg(feature = "futures03")]
#[test]
fn test_stream() {
    use futures_executor::block_on_stream;

    let gen = Gen::new(odd_numbers_less_than_ten);
    let xs: Vec<_> = block_on_stream(gen).collect();
    assert_eq!(xs, [1, 3, 5, 7, 9]);
}

#[cfg(feature = "proc_macro")]
#[test]
fn rc_proc_macro_fn() {
    use genawaiter::{rc::producer_fn, yield_};

    #[producer_fn(u8)]
    async fn odds() {
        for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
            yield_!(n);
        }
    }
    let gen = Gen::new(odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn rc_yield_a_func_method_call() {
    use genawaiter::{rc::producer_fn, yield_};

    fn pass_thru(n: u8) -> u8 {
        n
    }
    #[producer_fn(u8)]
    async fn odds() {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            if true {
                yield_!(pass_thru(n)).clone()
            }
        }
    }
    let gen = Gen::new(odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn rc_proc_macro_closure() {
    use genawaiter::{rc_producer, yield_};

    let gen = Gen::new(rc_producer!({
        let mut n = 1_u8;
        while n < 10 {
            yield_!(n);
            n += 2;
        }
    }));
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn rc_proc_macro_closure_yield2() {
    use genawaiter::{rc_producer, yield_};

    let gen = Gen::new(rc_producer!({
        let mut n = 1_u8;
        while n < 10 {
            yield_!(n);
            n += 2;
            yield_!(n - 1);
        }
    }));
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn rc_convenience_macro() {
    use genawaiter::{rc::gen, yield_};

    let g = gen!({
        let mut n = 1;
        while n < 10 {
            yield_!(n);
            n += 2;
        }
    });
    let res = g.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn rc_convenience_macro_resume() {
    use genawaiter::{rc::gen, yield_, GeneratorState};

    let mut gen = gen!({
        let mut resume_arg = yield_!(10_u8);
        assert_eq!(resume_arg, "abc");
        resume_arg = yield_!(20_u8);
        assert_eq!(resume_arg, "def");
    });

    assert_eq!(gen.resume_with("ignored"), GeneratorState::Yielded(10));
    assert_eq!(gen.resume_with("abc"), GeneratorState::Yielded(20));
    assert_eq!(gen.resume_with("def"), GeneratorState::Complete(()));
}
