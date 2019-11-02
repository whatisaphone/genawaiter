#![feature(async_await, async_closure)]
#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![cfg_attr(feature = "strict", deny(warnings))]

pub use crate::{
    engine::Co,
    safe_rc::Generator as SafeRcGenerator,
    state::GeneratorState,
};

mod engine;
mod iterator;
mod safe_rc;
mod state;
mod waker;
