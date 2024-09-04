mod pin_impl;
mod wire;

pub use pin_impl::Pin;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DriveValue, LogicValue};
    use std::rc::Rc;
    use wire::Wire;

    // Ensure the initial value actually gets set.
    #[test]
    fn test_pin_initial_value() {
        let pin_1 = Pin::new(DriveValue::HighImpedance);
        assert_eq!(pin_1.borrow().get_drive(), DriveValue::HighImpedance);
        assert_eq!(pin_1.borrow().read(), LogicValue::HighImpedance);

        let pin_2 = Pin::new(DriveValue::Strong(true));
        assert_eq!(pin_2.borrow().get_drive(), DriveValue::Strong(true));
        assert_eq!(pin_2.borrow().read(), LogicValue::Driven(true));
    }

    // Ensure that the `Pin` and `Wire` get updated when needed.
    #[test]
    fn test_pin_read_update() {
        let pin = Pin::new(DriveValue::HighImpedance);

        // Set the drive, but the change should not be immediately visible.
        pin.borrow_mut().set_drive(DriveValue::Strong(true));
        assert_eq!(pin.borrow().get_drive(), DriveValue::HighImpedance);
        assert_eq!(pin.borrow().read(), LogicValue::HighImpedance);

        // Tick to make the change visible.
        pin.borrow_mut().tick();
        assert_eq!(pin.borrow().get_drive(), DriveValue::Strong(true));
        assert_eq!(pin.borrow().read(), LogicValue::Driven(true));

        // Ticking again doesn't change anything.
        pin.borrow_mut().tick();
        assert_eq!(pin.borrow().get_drive(), DriveValue::Strong(true));
        assert_eq!(pin.borrow().read(), LogicValue::Driven(true));

        // Set the drive to what it already is. This is fine, and won't do anything, but does take a
        // different branch.
        pin.borrow_mut().set_drive(DriveValue::Strong(true));
        pin.borrow_mut().tick();
        assert_eq!(pin.borrow().get_drive(), DriveValue::Strong(true));
        assert_eq!(pin.borrow().read(), LogicValue::Driven(true));
    }

    // Ensure a `Pin` can be connected to itself (which does nothing).
    #[test]
    fn test_connect_self() {
        let pin = Pin::new(DriveValue::HighImpedance);
        Pin::connect(&pin, &pin);
    }

    // Ensure multiple `Pin`s can be connected together.
    #[test]
    fn test_connect_pins() {
        // Connect to pins and ensure both pins return the same set of all pins.
        let pin_1 = Pin::new(DriveValue::HighImpedance);
        let pin_2 = Pin::new(DriveValue::HighImpedance);
        Pin::connect(&pin_1, &pin_2);

        let all_pins = pin_1.borrow().get_connected_pins();
        assert_eq!(all_pins.len(), 2);
        assert!(Rc::ptr_eq(&pin_1, &all_pins[0]));
        assert!(Rc::ptr_eq(&pin_2, &all_pins[1]));

        let all_pins = pin_2.borrow().get_connected_pins();
        assert_eq!(all_pins.len(), 2);
        assert!(Rc::ptr_eq(&pin_1, &all_pins[0]));
        assert!(Rc::ptr_eq(&pin_2, &all_pins[1]));

        // Add a third pin, and ensure all three pins return the same set of all pins.
        let pin_3 = Pin::new(DriveValue::HighImpedance);
        Pin::connect(&pin_2, &pin_3);

        let all_pins = pin_1.borrow().get_connected_pins();
        assert_eq!(all_pins.len(), 3);
        assert!(Rc::ptr_eq(&pin_1, &all_pins[0]));
        assert!(Rc::ptr_eq(&pin_2, &all_pins[1]));
        assert!(Rc::ptr_eq(&pin_3, &all_pins[2]));

        let all_pins = pin_2.borrow().get_connected_pins();
        assert_eq!(all_pins.len(), 3);
        assert!(Rc::ptr_eq(&pin_1, &all_pins[0]));
        assert!(Rc::ptr_eq(&pin_2, &all_pins[1]));
        assert!(Rc::ptr_eq(&pin_3, &all_pins[2]));

        let all_pins = pin_3.borrow().get_connected_pins();
        assert_eq!(all_pins.len(), 3);
        assert!(Rc::ptr_eq(&pin_1, &all_pins[0]));
        assert!(Rc::ptr_eq(&pin_2, &all_pins[1]));
        assert!(Rc::ptr_eq(&pin_3, &all_pins[2]));
    }

    #[test]
    #[should_panic]
    fn test_pin_set_drive_multiple() {
        let pin = Pin::new(DriveValue::HighImpedance);
        pin.borrow_mut().set_drive(DriveValue::Strong(true));
        pin.borrow_mut().set_drive(DriveValue::Strong(true));
    }

    #[test]
    #[should_panic]
    fn test_wire_set_initial_pin_multiple() {
        let pin = Pin::new(DriveValue::HighImpedance);
        let wire = Wire::new();
        wire.borrow_mut().set_initial_pin(&pin);
        wire.borrow_mut().set_initial_pin(&pin);
    }
}
