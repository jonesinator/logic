use crate::{pin::wire::Wire, DriveValue, LogicValue};
use std::cell::RefCell;
use std::rc::Rc;

/// Represents a physical pin on an electronic device.
pub struct Pin {
    /// The current value that this individual `Pin` is driving.
    current_drive: DriveValue,

    /// The value that this `Pin` should start driving on the next tick, if any.
    ///
    /// Use `set_drive` to set. Note, `set_drive` can only be called once per `tick`.
    next_drive: Option<DriveValue>,

    /// The `Wire` that this `Pin` is connected to, representing connections to other `Pin`s.
    ///
    /// It would make sense for this to be an `Option`, and only instantiate a `Wire` when two or
    /// more `Pin`s are connected together; however, we've opted to make `Pin`s _always_ be
    /// associatged with a `Wire` so we don't have to check for the `Wire`'s existence in many
    /// places, and we only need a single method to connect `Wire`s.
    ///
    /// The outside world is not really aware of the existence of `Wire`s, they're managed
    /// implicitly by `Pin`s.
    wire: Rc<RefCell<Wire>>,
}

impl Pin {
    /// Connects two `Pin`s together via a shared `Wire`.
    pub fn connect(pin_1: &Rc<RefCell<Pin>>, pin_2: &Rc<RefCell<Pin>>) {
        let wire_2 = &pin_2.borrow().wire.clone();
        Wire::connect(&pin_1.borrow().wire, wire_2);
    }

    /// Gets all of the `Pin`s connected to this `Pin` (including this `Pin`).
    pub fn get_connected_pins(&self) -> Vec<Rc<RefCell<Pin>>> {
        self.wire.borrow().get_all_pins()
    }

    /// Gets the current `DriveValue` that is being driven by this `Pin`.
    pub fn get_drive(&self) -> DriveValue {
        self.current_drive
    }

    /// Reads the current value of the `Wire` to which this `Pin` is connected.
    pub fn read(&self) -> LogicValue {
        self.wire.borrow().read()
    }

    /// Creates a new `Pin` in the given initial state, connected to nothing.
    pub(crate) fn new(initial_value: DriveValue) -> Rc<RefCell<Pin>> {
        let pin = Rc::new(RefCell::new(Pin {
            current_drive: initial_value,
            next_drive: None,
            wire: Wire::new(),
        }));
        pin.borrow().wire.borrow_mut().set_initial_pin(&pin);
        pin
    }

    /// Sets the next drive value for the `Pin`. This can only be called once per `tick`.
    pub(crate) fn set_drive(&mut self, to: DriveValue) {
        match self.next_drive {
            Some(_) => panic!("Drive set multiple times in a single tick."),
            None => self.next_drive = Some(to),
        }
    }

    /// Updates the `Wire` for this `Pin`. This is used only by `Wire::connect`, to merge `Wire`s.
    pub(crate) fn set_wire(&mut self, wire: Rc<RefCell<Wire>>) {
        self.wire = wire
    }

    /// Updates the `Pin`'s current state with the next state, if one was set earlier.
    /// Returns `true` if the `Pin`'s drive changed. Returns `false` otherwise.
    pub(crate) fn tick(&mut self) -> bool {
        if let Some(next_drive) = self.next_drive {
            let changed = self.current_drive != next_drive;
            if changed {
                self.wire
                    .borrow_mut()
                    .update_pin(self.current_drive, next_drive);
                self.current_drive = next_drive;
            }
            self.next_drive = None;
            changed
        } else {
            false
        }
    }
}
