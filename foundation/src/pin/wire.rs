use crate::{value::DriveValueAccumulator, DriveValue, LogicValue, Pin};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Represents a physical wire in an electronic device.
pub(crate) struct Wire {
    /// The current `LogicValue` to which the `Wire` is driven. This is the aggregate of all of the
    /// `DriveValue`s from all of the `Pin`s on the `Wire`.
    value: LogicValue,

    /// The collection of `Pin`s connected to the `Wire`.
    ///
    /// Since `Pin`s have an owning pointer to their `Wire`, the `Wire` must have a `Weak` pointer
    /// back to its `Pin`s to avoid reference cyles that would lead to memory leaks.
    pins: Vec<Weak<RefCell<Pin>>>,

    /// Used to keep track of the current `LogicValue` of the `Wire` without having to iterate
    /// through the `Pin`s in order to determine it.
    drive_value_accumulator: DriveValueAccumulator,
}

impl Wire {
    /// Creates a new `Wire` with no `Pin`s. You must call `set_initial_pin` after this.
    pub fn new() -> Rc<RefCell<Wire>> {
        Rc::new(RefCell::new(Wire {
            value: LogicValue::HighImpedance,
            pins: vec![],
            drive_value_accumulator: DriveValueAccumulator::new(),
        }))
    }

    /// Adds the first `Pin` to the `Wire`. This isn't done in `new` since we must borrow the
    /// `Pin`, but `new` is called while the `Pin` itself is still being created. This should
    /// _always_ be called after `new` and is considered part of the initialization of `Wire`.
    pub fn set_initial_pin(&mut self, pin: &Rc<RefCell<Pin>>) {
        if !self.pins.is_empty() {
            panic!("Wire is already associated with a Pin.");
        }

        self.pins.push(Rc::downgrade(pin));
        self.value = self
            .drive_value_accumulator
            .update(DriveValue::HighImpedance, pin.borrow().get_drive());
    }

    /// Reads the current `LogicValue` to which the `Wire` is driven.
    pub fn read(&self) -> LogicValue {
        self.value
    }

    /// Updates the current value of the `Wire` in the face of any changes made to the `Pin`s.
    pub fn update_pin(&mut self, before: DriveValue, after: DriveValue) {
        self.value = self.drive_value_accumulator.update(before, after)
    }

    /// Gets a `Vec` filled with pointers to all of the `Pin`s connected to this `Wire`.
    pub fn get_all_pins(&self) -> Vec<Rc<RefCell<Pin>>> {
        self.pins
            .iter()
            .map(|pin| pin.upgrade().expect("dropped pin"))
            .collect()
    }

    /// Connects two `Wire`s together. This essentially merges `wire_2` into `wire_1`, and in the
    /// end we're left with only one `Wire` which all of the `Pin`s point at.
    pub fn connect(wire_1: &Rc<RefCell<Wire>>, wire_2: &Rc<RefCell<Wire>>) {
        // If they're the same wire, then they're already "connected".
        if Rc::ptr_eq(wire_1, wire_2) {
            return;
        }

        // Make all of the Pins in wire 2 point to wire 1 as their wire. After this, there should
        // be no references to wire 2 (other than what we have right now) so it will be dropped
        // shortly.
        for weak_pin in wire_2.borrow_mut().pins.iter() {
            weak_pin
                .upgrade()
                .expect("dropped pin")
                .borrow_mut()
                .set_wire(wire_1.clone());
        }

        // Transfer all of the pins in wire 2's vector to wire 1's vector, and add their drive
        // accumulators together to get the new overall drive value for the wire.
        let mut mut_wire_1 = wire_1.borrow_mut();
        mut_wire_1.pins.append(&mut wire_2.borrow_mut().pins);
        mut_wire_1.value = mut_wire_1
            .drive_value_accumulator
            .add(&wire_2.borrow().drive_value_accumulator);
    }
}
