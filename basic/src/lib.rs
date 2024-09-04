//! Implements several simple digital logic devices using logic gates.
#![deny(missing_docs)]

mod adder;
mod flip_flop;

pub use adder::{FullAdder, HalfAdder, RippleCarryAdder};
pub use flip_flop::SrLatch;
