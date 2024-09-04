//! Implement logic gates from transistors.
//!
//! Outside of this crate, raw transistors are basically not used, and everything is built from
//! these logic gates.
#![deny(missing_docs)]

mod and_gate;
mod buffer_gate;
mod nand_gate;
mod nor_gate;
mod not_gate;
mod or_gate;
mod tri_state_buffer_gate;
mod xnor_gate;
mod xor_gate;

pub use and_gate::AndGate;
pub use buffer_gate::BufferGate;
pub use nand_gate::NandGate;
pub use nor_gate::NorGate;
pub use not_gate::NotGate;
pub use or_gate::OrGate;
pub use tri_state_buffer_gate::TriStateBufferGate;
pub use xnor_gate::XnorGate;
pub use xor_gate::XorGate;
