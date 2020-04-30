//! This is an example of a recursive generator.
//!
//! Note that a naive recursive generator would result in a recursive state machine (and a compiler
//! error about an infinitely sized type). Thus, we need to introduce a layer of indirection (a
//! `Box`). Each level of recursion will result in another allocation.
//!
//! Read this page for details:
//!
//! <https://rust-lang.github.io/async-book/07_workarounds/05_recursion.html>

#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use genawaiter::sync::{Gen, GenBoxed};

fn main() {
    for n in countdown(10) {
        println!("{}", n);
    }
}

fn countdown(start: i32) -> GenBoxed<i32> {
    Gen::new_boxed(|mut co| {
        async move {
            if start == 0 {
                return;
            }

            co.yield_(start).await;

            for n in countdown(start - 1) {
                co.yield_(n).await;
            }
        }
    })
}
