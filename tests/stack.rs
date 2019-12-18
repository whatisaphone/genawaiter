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

#[cfg(feature = "proc_macro")]
#[test]
fn stack_yield_a_func_call() {
    fn pass_thru(n: u8) -> u8 {
        n
    }

    #[genawaiter::stack::stack_yield_fn(u8)]
    async fn odds() {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            if true {
                genawaiter::yield_!(pass_thru(n))
            }
        }
    }
    genawaiter::generator_mut!(gen, odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_yield_loop_break() {
    #[genawaiter::stack::stack_yield_fn(u8)]
    async fn odds() {
        let mut n = 0_u8;
        loop {
            if n == 9 {
                break;
            }
            loop {
                n += 1;
                if n % 2 != 0 {
                    break genawaiter::yield_!(n);
                }
            }
        }
    }
    genawaiter::generator_mut!(gen, odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[test]
fn stack_yield_match() {
    #[genawaiter::stack::stack_yield_fn(u8)]
    async fn odds() {
        for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
            match Some(n) {
                Some(n) if n % 2 != 0 => {
                    println!("{}", n);
                    genawaiter::yield_!(n)
                }
                _ => {}
            }
        }
    }
    genawaiter::generator_mut!(gen, odds);
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 3, 5, 7, 9], res)
}

#[cfg(feature = "proc_macro")]
#[cfg(feature = "nightly")]
#[test]
fn stack_yield_closure() {
    let mut shelf = genawaiter::stack::Shelf::new();
    let gen = unsafe {
        genawaiter::stack::Gen::new(
            &mut shelf,
            genawaiter::stack_yield_cls!(
                u8 in async move || {
                    let mut n = 1_u8;
                    while n < 10 {
                        genawaiter::yield_!(n);
                        n += 2;
                        let _ = yield_!(n - 1).clone();
                    }
                }
            ),
        )
    };
    let res = gen.into_iter().collect::<Vec<_>>();
    assert_eq!(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], res)
}
