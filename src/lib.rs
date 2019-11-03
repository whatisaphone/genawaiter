#![feature(async_await, async_closure)]
#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![cfg_attr(feature = "strict", deny(warnings))]

pub use state::GeneratorState;

mod generator;
mod iterator;
pub mod safe_rc;
mod stack;
mod state;
mod waker;
