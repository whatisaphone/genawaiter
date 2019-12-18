// Remove to test otherwise CI fails because of the async closures
#![feature(async_closure)]
#[cfg(feature = "proc_macro_example")]
mod mac {
    fn stack_yield_fn() {
        #[genawaiter::stack::stack_yield_fn(u8)]
        async fn odds() {
            for n in (1..).step_by(2).take_while(|&n| n < 10) {
                genawaiter::yield_! {n}
                // this is still `()` but for completeness you can do it.
                let _x = genawaiter::yield_!(n);
                genawaiter::yield_!(n)
            }
        }
        genawaiter::generator_mut!(gen, odds);
        let res = gen.into_iter().collect::<Vec<_>>();
        assert_eq!(vec![1, 1, 1, 3, 3, 3, 5, 5, 5, 7, 7, 7, 9, 9, 9], res)
    }

    fn rc_yield_a_func_method_call() {
        fn pass_thru(n: u8) -> u8 {
            n
        }

        #[genawaiter::rc::rc_yield_fn(u8)]
        async fn odds() {
            for n in (1..).step_by(2).take_while(|&n| n < 10) {
                if true {
                    genawaiter::yield_!(pass_thru(n)).clone()
                }
            }
        }
        let gen = genawaiter::rc::Gen::new(odds);
        let res = gen.into_iter().collect::<Vec<_>>();
        assert_eq!(vec![1, 3, 5, 7, 9], res)
    }

    fn sync_proc_macro_fn() {
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

    fn stack_loop_break() {
        #[genawaiter::stack::stack_yield_fn(u8)]
        async fn odds() {
            let mut n = 0_u8;
            loop {
                if n == 9 { break }
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

    fn stack_yield_match() {
        #[genawaiter::stack::stack_yield_fn(u8)]
        async fn odds() {
            for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
                match Some(n) {
                    Some(n) if n % 2 != 0 => {
                        println!("{}", n);
                        genawaiter::yield_!(n)
                    },
                    _ => {},
                }
            }
        }
        genawaiter::generator_mut!(gen, odds);
        let res = gen.into_iter().collect::<Vec<_>>();
        assert_eq!(vec![1, 3, 5, 7, 9], res)
    }

    pub fn main() {
        rc_yield_a_func_method_call();
        stack_yield_closure();
        sync_proc_macro_fn();
        stack_yield_fn();
        stack_yield_match();
    }
}
fn main() {
    #[cfg(feature = "proc_macro_example")]
    mac::main();
}
