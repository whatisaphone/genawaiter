#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(missing_docs, clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use genawaiter::stack::{let_gen_using, Co, Gen, Shelf};

async fn odd_numbers_less_than_ten(co: Co<'_, i32>) {
    for n in (1..).step_by(2).take_while(|&n| n < 10) {
        co.yield_(n).await;
    }
}

#[test]
fn test_basic() {
    let_gen_using!(gen, odd_numbers_less_than_ten);

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

    let_gen_using!(gen, odd_numbers_less_than_ten);
    let xs: Vec<_> = block_on_stream(gen).collect();
    assert_eq!(xs, [1, 3, 5, 7, 9]);
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_proc_macro_fn() {
    use genawaiter::{stack::producer_fn, yield_};
    #[producer_fn(u8)]
    async fn odds() {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            let _x = yield_!(n);
        }
    }
    let_gen_using!(gen, odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_yield_a_func_call() {
    use genawaiter::{stack::producer_fn, yield_};

    fn pass_thru(n: u8) -> u8 {
        n
    }
    #[producer_fn(u8)]
    async fn odds() {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            if true {
                yield_!(pass_thru(n))
            }
        }
    }
    let_gen_using!(gen, odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_yield_loop_break() {
    use genawaiter::{stack::producer_fn, yield_};

    #[producer_fn(u8)]
    async fn odds() {
        let mut n = 0_u8;
        loop {
            if n == 9 {
                break;
            }
            loop {
                n += 1;
                if n % 2 != 0 {
                    break yield_!(n);
                }
            }
        }
    }
    let_gen_using!(gen, odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_yield_match() {
    use genawaiter::{stack::producer_fn, yield_};

    #[producer_fn(u8)]
    async fn odds() {
        for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
            match Some(n) {
                Some(n) if n % 2 != 0 => yield_!(n),
                _ => {}
            }
        }
    }
    let_gen_using!(gen, odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_yield_closure() {
    use genawaiter::{stack_producer, yield_};

    let mut shelf = genawaiter::stack::Shelf::new();
    let gen = unsafe {
        Gen::new(
            &mut shelf,
            stack_producer!({
                let mut n = 1_u8;
                while n < 10 {
                    yield_!(n);
                    n += 2;
                }
            }),
        )
    };
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_convenience_macro() {
    use genawaiter::{stack::let_gen, yield_};

    let_gen!(generator, {
        let mut n = 1;
        while n < 10 {
            yield_!(n);
            n += 2;
        }
    });
    let res = generator.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_convenience_macro_resume() {
    use genawaiter::{stack::let_gen, yield_, GeneratorState};

    let_gen!(gen, {
        let resume_arg = yield_!(10_u8);
        assert_eq!(resume_arg, "abc");
        let resume_arg = yield_!(20_u8);
        assert_eq!(resume_arg, "def");
    });

    assert_eq!(gen.resume_with("ignored"), GeneratorState::Yielded(10));
    assert_eq!(gen.resume_with("abc"), GeneratorState::Yielded(20));
    assert_eq!(gen.resume_with("def"), GeneratorState::Complete(()));
}
