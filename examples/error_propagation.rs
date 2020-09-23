//! This example demonstrates error propagation with generators.
//!
//! See [fallible-iterator's docs][fitd] for discussion about the same problem,
//! but in the context of iterators.
//!
//! [fitd]: https://docs.rs/fallible-iterator/0.2.1/fallible_iterator/

#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use std::error::Error;

#[cfg(not(feature = "proc_macro"))]
fn main() {
    println!("Feature `proc_macro` is required for this example.");
}

#[cfg(feature = "proc_macro")]
fn main() -> Result<(), Box<dyn Error>> {
    use genawaiter::{sync::gen, yield_};

    fn main() -> Result<(), Box<dyn Error>> {
        // Create a generator which yields values of type `Result<String, _>`
        let counter = gen!({
            for num in 0..10 {
                // Perform some fallible operation, and yield the result (or the error)
                yield_!(process(num));
            }
        });

        for result in counter {
            // Check each item for errors, and bail early if we hit one
            let result = result?;
            println!("{}", result);
        }
        Ok(())
    }

    fn process(num: u8) -> Result<String, Box<dyn Error>> {
        // Pretend this function has a failure condition
        if num > 5 {
            return Err(<_>::from("enhance your small"));
        }

        // If there's no error, do some work and return a value
        Ok(format!(":{}:", num))
    }

    main()
}
