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
    for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
        genawaiter::yield_!{n};
        genawaiter::yield_!{n};
    }
}

fn main() {
    // #[yielder_cls(u8)]
    // let hello = async move |co: Co<'static, u8>| {
    //     let mut n = 1_u8;
    //     while n < 10 {
    //         yield_!(n);
    //         n += 2;
    //     }
    // };

    // let mut shelf = Shelf::new();
    // #[yielder_cls(u8)]
    // let gen = unsafe {
    //     Gen::new(&mut shelf, yielder_cls! {
    //         let mut n = 1_u8;
    //         while n < 10 {
    //             // or without semi-colon
    //             yield_!(n)
    //             n += 2;
    //         }
    //     })
    // };
    // let gen = Gen::new(odds);

    // generator_mut!(test, hello);

    generator_mut!(gen, odds);

    for x in gen {
        println!("{}", x);
    }
}
