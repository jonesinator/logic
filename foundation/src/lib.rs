//! A barebones digital logic simulation and ciruit analysis framework. CMOS transistors are used
//! as the base device upon which everything else is built.
//!
//! # Overview
//!
//! This crate contains the full simulation framework, and the code is kept rather minimal to keep
//! the logic as clear as possible. The other crates build upon this crate, but all they can really
//! do is provide a function that arranges transistors in a certain way, but they cannot play any
//! active role in the simulation. That is to say, there's no magic, it's really just transistors
//! connected together following a fairly simple set of rules.
//!
//! # Concepts
//!
//! The following is a short guide to the concepts in the framework.
//!
//! ## Value
//!
//! This simulation exists at the edge between electrical components and "pure" digital logic. This
//! is best represented by the distinction between [`DriveValue`] and [`LogicValue`], which are the
//! first concepts you should take a look at. They're both very simple `enum`s, and have a lot in
//! common.
//!
//! A [`DriveValue`] is an electrical state, and can distinguish between [`DriveValue::Strong`] and
//! [`DriveValue::Weak`] drives, i.e. either directly connected to a voltage source, or connected
//! "weakly" through a resistor. A `LogicValue` is a logical state, representing the aggregate state
//! of a collection of [`DriveValue`]s all connected to the same electrical conductor.
//!
//! A [`LogicValue`] is only `Driven` `true` or `false`, and does not distinguish between `Strong`
//! and `Weak` drives like a [`DriveValue`]. The rules for converting a set of [`DriveValue`] to a
//! single [`LogicValue`] are encoded in the `DriveValueAccumulator` (A private struct). The rules
//! attempt to model the physics of a wire in a very simple way, and can be derived for yourself
//! using a power supply, multimeter, and piece of wire. The `Error` states are not real physical
//! states, but represent cases where the physical state on the wire is indeterminate. In a sense,
//! the simulation is more "strict" than the physical system, because the physical system resolves
//! to some state, it just can't be predicted which.
//!
//! ## Pin
//!
//! These abstract [`DriveValue`] and [`LogicValue`] concepts can now be attached to more concrete
//! entities, the [`Pin`] and the `Wire` respectively (the `Wire` `struct` is not exposed publicly,
//! but still be discussed here.)
//!
//! A [`Pin`] represents an actual "pin" on an electronics component like a transistor, i.e. a metal
//! conductor that connects to a particular piece of the component's internals. On a breadboard,
//! PCB, or integrated circuit, different pins of different devices are connected together using
//! metal wires. Similarly, [`Pin::connect`] is used to connect [`Pin`]s, and internally a `Wire`
//! structure is used to track this connection.
//!
//! Each [`Pin`] has its own [`DriveValue`] that it is driving onto the `Wire`. The `Wire` has a
//! `DriveValueAccumulator` which combines all of the [`DriveValue`]s of the [`Pin`]s into a
//! single [`LogicValue`] that the `Wire` resolves to. Since `Wire` is hidden, the [`Pin::read`]
//! function is used to get the [`LogicValue`] of the underlying `Wire`.
//!
//! Note that [`Pin`]s cannot be created outside of this crate, and there are no functions other
//! than [`Pin::connect`] that mutate pins. So, for the most part, your interact with [`Pin`]s will
//! be merely connecting them and passing them along through the hierarchy. Because [`Pin`]s must
//! be mutable to connect them, and because `Wire`s need to keep a list of `Weak` pointers to their
//! constituent `[Pin]`s (in order to update them when connections are made), we must use
//! `Rc<RefCell<Pin>>` everywhere rather than nicer references, unfortunately.
//!
//! ## Device
//!
//! Now we come to the abstract concept of a [`Device`], which represents any electronic component
//! in the system. Fundamentally a [`Device`] is two collections:
//!
//! 1. A collection of [`Pin`]s representing the external interface of the [`Device`].
//! 2. A collection of [`Device`]s representing the subdevices of the [`Device`].
//!
//! This is incredibly generic, and can represent basically any device in an electronic system.
//! Since this definition is recursive, i.e. a [`Device`] can contain other [`Device`]s, then we
//! will need some initial "primitive" devices which consist only of [`Pin`]s, to provide a way to
//! terminate this recursion, since [`Pin`]s cannot be created outside of this crate. That is
//! exactly what the `primitive` module does, documented below.
//!
//! Note, the [`Device`] trait is cumbersome to implement by hand. A derive procedural macro is
//! provided at [`device_derive::Device`] which does most of the hard work for you.
//!
//! The [`Device`] trait makes it possible to write generic algorithms that can work with any
//! [`Device`]. The intent is that concrete types are used when constructing specific circuits,
//! which is more clear and readable, but generic algorithms can use this [`Device`] interface to
//! work with arbitrary devices. The hope is that this can be used, for example, to:
//!
//! 1. Synthesize the circuit to a Fritzing diagram, so it can be constructed on a breadboard.
//! 2. Synthesize the circuit to a KiCad PCB that can be printed, ordered, populated, and tested.
//! 3. Synthesize the circuit to an FPGA.
//! 4. Count the number of transistors in a circuit.
//! 5. Other wild stuff!
//!
//! A [`DeviceContainer`] `enum` exists, that allows a [`Device`] to store one of or multiple of a
//! [`Pin`] or [`Device`]. This is semantically and syntatically much nicer than using a vector of
//! a single item to represent a single item, which seems to be the primary alternative.
//!
//! An [`AnyDevice`] trait is included which combines the [`Device`] and `Any` traits. It is
//! automatically implemented for any [`Device`] which implements `Any`, which should be
//! basically everything. This allows us to get the concrete type back in our generic algorithms, if
//! needed.
//!
//! ## Primitives
//!
//! There are three "primitive" [`Device`]s, i.e. [`Device`]s consisting only of [`Pin`]s.
//!
//! ### Constant
//!
//! A [`Constant`] is a very simple [`Device`] with a single [`Pin`] which is always driving some
//! [`DriveValue::Strong`] or [`DriveValue::Weak`] onto the `Wire`. [`DriveValue::Strong`] values
//! are interpreted as a direct connection to a voltage source/sink (VCC/GND), while
//! [`DriveValue::Weak`] values are interpreted as a connection to a voltage source/sink (VCC/GND)
//! through a "pull-up" or "pull-down" resistor.
//!
//! ### Transistor
//!
//! A [`Transistor`] is the fundamental unit of digital logic in this simulation. These devices are
//! immensely important, and immensely complicated, but our model of them here is rather simple.
//! There are two kinds of CMOS transitor, NMOS and PMOS, and they can be created with
//! [`Transistor::new_nmos`] and [`Transistor::new_pmos`] respectively. They differ in which
//! [`LogicValue`] causes them to active. Both of them consist simply of three [`Pin`]s
//!
//! 1. `gate` -- This is the [`Pin`] that "makes the decision".
//! 2. `source` -- This is the [`Pin`] that is connected to the source of the current we wish to
//!    connect to.
//! 3. `drain` -- This is the [`Pin`] that will be connected to `source` if the `gate` is in the
//!    activation state, but will be [`DriveValue::HighImpedance`] otherwise.
//!
//! The rules in the code attempt to model that of real transitors, and you should be able to build
//! the truth table for each transistor for yourself using a power source, a transistor, and a
//! multimeter.
//!
//! ### TestPin
//!
//! A [`TestPin`] is a very simple [`Device`] with a single [`Pin`] which is similar to a
//! [`Constant`], but its value can be manipulated over time. In theory, this is primarily for use
//! in tests, but it could synthesize to a header if you want the [`TestPin`] to remain in a
//! physical design.
//!
//! ## Simulation
//!
//! The `simulation` module provides the [`print()`], [`settle`], and [`tick`] functions, all
//! accepting a [`Device`]. The [`tick`] function moves forward one time step. The [`settle`]
//! function moves forward until the circuit stops changing. The [`print()`] function is for
//! debugging, and prints a very detailed representation of the [`Device`].
//!
//! # Usage
//!
//! In general, you will use this crate by creating your own `struct`s implementing the
//! [`Device`] trait. You will do this by using the `device_derive::Device` procedural macro on your
//! `struct` and annotating certain fields.
//!
//! Below is an example of a nonsense [`Device`], but it does demonstrate the basic structure of all
//! [`Device`]s, especailly the steps within the `new` function.
//!
//! ```
//! use device_derive::Device;
//! use foundation::{AnyDevice, Device, DeviceContainer, Pin, Transistor};
//! use std::cell::RefCell;
//! use std::rc::Rc;
//!
//! #[derive(Device)]
//! struct MyDevice {
//!   #[child]
//!   nmos: Transistor,
//!
//!   #[children]
//!   pmos: Vec<Transistor>,
//!
//!   #[pin]
//!   input: Rc<RefCell<Pin>>,
//!
//!   #[pins]
//!   output: Vec<Rc<RefCell<Pin>>>,
//! }
//!
//! impl MyDevice {
//!     fn new(width: usize) -> Self {
//!         // Create children.
//!         let nmos = Transistor::new_nmos();
//!         let pmos: Vec<Transistor> = (0..width).map(|_| Transistor::new_nmos()).collect();
//!
//!         // Clone pins.
//!         let input = nmos.get_gate().clone();
//!         let output: Vec<Rc<RefCell<Pin>>> = pmos.iter().map(|t| t.get_drain().clone()).collect();
//!
//!         // Connect pins.
//!         Pin::connect(nmos.get_source(), pmos[0].get_gate());
//!         Pin::connect(nmos.get_drain(), pmos[1].get_gate());
//!
//!         // Return completed struct.
//!         Self {
//!             nmos,
//!             pmos,
//!             input,
//!             output,
//!         }
//!     }
//! }
//! ```
//!
//! After you've defined your [`Device`] you can simulate it using the [`tick`] and [`settle`]
//! functions, as well as using the [`print()`] function to get a detailed representation of your
//! [`Device`].
//!
//! # Handwavey Stuff About the Future
//!
//! It's my hope that starting from this crate (and [`device_derive`]) an entire RISC-V computer
//! can be implemented by building up digital logic abstractions. Along the way, tools will be
//! created so these circuits can be tested on breadboards, PCBs, and FPGAs in isolation, before
//! being combined into a whole computer.
//!
//! A stretch goal is having the final computer be built from 7400 series logic, but each chip in
//! the computer is in a DIP socket that can be swapped out with an FPGA implementation of the same
//! chip, or even a PCB or or breadboard of the same. That chip could contain simpler 7400 series
//! logic, so the PCB would contain several other chips, each of which can be swapped out in the
//! same way, until we get down to the transistor level. This way, we don't have to build an entire
//! computer out of transistors, but we can still physically demonstrate that each part can be
//! \[recursively\] swapped out with something made out of just transistors. Further, this computer
//! could use SPI Flash memory for its stored programs (as many existing CPUs do). Then we could
//! create a smaller, simpler SPI Flash programmer from a hexadecimal keypad, to create the initial
//! programs using machine code without needing an existing computer.
//!
//! Once the computer is completed it can be used to make a keyboard, display, and terminal
//! interface which reads from the keyboard and updates the display while also writing the input to
//! another SPI Flash card. This would be the first source code editor program. The first assembler
//! program, written using the hex keypad, could be created using machine code. It can read from the
//! source SPI Flash, and write the assembled program to a third SPI Flash which could then be run
//! on any other instance of this computer.
//!
//! Given this configuration, a SPI flash filesystem and higher level language could be created, and
//! using those tools, enough of a POSIX system could be built, such that it could host the
//! <https://bootstrappable.org/> procedure to get a working GNU/Linux system essentially
//! bootstrapped from nothing on a computer built from scratch.
//!
//! While the end goal is to produce a GNU/Linux from nothing, it is very interesting that this
//! provides numerous branching-off points to try other system architectures, not just at the
//! software level, but at the low-level hardware level.
//!
//! This is probably a mind boggling amount of work, and likely a lot of bad assumptions have been
//! made, but you have to start somewhere, and starting with a transistor-level simulation of
//! digital electronics seems like a pretty good place.

#![feature(stmt_expr_attributes)]
#![feature(strict_overflow_ops)]
#![feature(trait_upcasting)]
#![deny(missing_docs)]

// Modules.
mod device;
mod pin;
mod primitive;
mod simulation;
mod value;

// Re-exports.
pub use device::{AnyDevice, Device, DeviceContainer};
pub use pin::Pin;
pub use primitive::{Constant, TestPin, Transistor};
pub use simulation::{print, settle, tick};
pub use value::{DriveValue, LogicValue, DRIVE_VALUES};
