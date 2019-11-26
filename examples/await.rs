#![feature(async_closure)]
#![feature(proc_macro_hygiene)]
#![feature(generators)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use genawaiter::{
    generator_mut,
    stack::{Co, Gen, Shelf},
};

use gen_proc_macro::{yielder_cls, yielder_fn};

#[yielder_fn(u8)]
async fn odds() {
    for n in (1_u8..).step_by(2).take_while(|&n| n < 10) {
        yield n;
    }
}

fn main() {
    #[yielder_cls(u8)]
    let hello = async move |co: Co<'static, u8>| {
        let mut n = 1_u8;
        while n < 10 {
            yield n;
            n += 2;
        }
    };

    // let hello = async move |co: Co<'_, (), String>| {
    //     let x = co.yield_(()).await;
    //     println!("{}", x)
    // };
    // generator_mut!(gen, hello);
    // gen.resume_with("hello".to_string());

    let mut shelf = Shelf::new();
    #[yielder_cls(u8)]
    let gen = unsafe {
        Gen::new(&mut shelf, async move || {
            let mut n = 1_u8;
            while n < 10 {
                yield n;
                n += 2;
            }
        })
    };
    // let gen = Gen::new(odds);

    // generator_mut!(test, hello);

    //generator_mut!(gen, odds);

    for x in gen {
        println!("{}", x);
    }
    assert!(true);
    assert_eq!(0, 0)
}
