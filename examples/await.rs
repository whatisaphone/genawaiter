#![feature(async_closure)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(dead_code)]

#[cfg(feature = "proc_macro")]
mod mac {
    use proc_macro_hack;
    use genawaiter::{
        generator_mut,
        stack::{Co, Gen, Shelf},
        yield_,
    };

    #[proc_macro_hack::proc_macro_hack]
    use genawaiter_proc_macro::stack_yield_cls;
    use genawaiter_proc_macro::stack_yield_fn;

    #[stack_yield_fn(u8)]
    async fn odds() {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            genawaiter::yield_! {n}
            // this is still `()` but for completeness you can do it.
            let x = genawaiter::yield_!(n);
            genawaiter::yield_!(n)
        }
    }

    #[stack_yield_fn(u8)]
    async fn odds2() {
        yield_!(10)
    }

    #[stack_yield_fn(u8)]
    async fn odds3() {
        for n in (1..).step_by(2).take_while(|&n| n < 10) {
            if true {
                yield_!(n)
            }
        }
    }

    pub fn main() {
        // let closure = stack_yield_fn!(u8 in async move || {
        //     yield_!(5);
        // });

        // let hello = stack_yield_fn!(&str in async move || {
        //     yield_!("");
        // });

        let mut shelf = Shelf::new();
        let gen = unsafe {
            Gen::new(&mut shelf, stack_yield_cls!(
                u8 in async move || {
                    let mut n = 1_u8;
                    while n < 10 {
                        yield_!(n);
                        n += 2;
                        yield_!(n - 1)
                    }
                }
            ))
        };
        for x in gen {
            println!("{}", x);
        }
        // let gen = Gen::new(odds);

        generator_mut!(gen, odds);

        for x in gen {
            println!("{}", x);
        }
    }
}
fn main() {
    #[cfg(feature = "proc_macro")]
    mac::main();
}
