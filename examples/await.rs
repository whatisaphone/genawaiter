#![feature(async_closure)]
#![feature(proc_macro_hygiene)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use genawaiter::{
    generator_mut,
    stack::{yielder_cls, yielder_fn, Co, Gen, Shelf},
    yield_,
};

#[yielder_fn(u8)]
async fn odds() {
    for n in (1..).step_by(2).take_while(|&n| n < 10) {
        genawaiter::yield_! {n}
        // this is still `()` but for completeness you can do it.
        let x = genawaiter::yield_!(n);
        genawaiter::yield_!(n)
    }
}

#[yielder_fn(u8)]
async fn odds2() {
    yield_!(10)
}

#[yielder_fn(u8)]
async fn odds3() {
    for n in (1..).step_by(2).take_while(|&n| n < 10) {
        if true {
            yield_!(n)
        }
    }
}

fn main() {
    // let closure = yielder_cls!(u8 in async move || {
    //     yield_!(5);
    // });

    // let hello = yielder_cls!(&str in async move || {
    //     yield_!("");
    // });

    let mut shelf = Shelf::new();
    let gen = unsafe {
        Gen::new(&mut shelf, yielder_cls!(
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
