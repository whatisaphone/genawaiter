//! This is a full project with the example from the readme.
//!
//! It also serves as an integration test for the proc macro.

#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use genawaiter::{sync::gen, yield_};

fn main() {
    let odd_numbers_less_than_ten = gen!({
        let mut n = 1;
        while n < 10 {
            yield_!(n); // Suspend a function at any point with a value.
            n += 2;
        }
    });

    // Generators can be used as ordinary iterators.
    for num in odd_numbers_less_than_ten {
        println!("{}", num);
    }
}
