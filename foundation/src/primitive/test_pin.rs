use crate::{AnyDevice, Device, DeviceContainer, DriveValue, Pin};
use device_derive::Device;
use std::cell::RefCell;
use std::rc::Rc;

/// A `Device` consisting of a single `Pin` which can be controlled for testing purposes.
#[derive(Device)]
pub struct TestPin {
    /// The pin whose drive can be controlled for testing purposes.
    #[pin]
    output: Rc<RefCell<Pin>>,
}

impl TestPin {
    /// Creates a new constant which drives a `DriveValue::Strong` value onto the `Wire`.
    pub fn new(initial_drive: DriveValue) -> Self {
        TestPin {
            output: Pin::new(initial_drive),
        }
    }

    /// Sets the drive of the test pin.
    pub fn set_drive(&mut self, new_drive: DriveValue) {
        let mut output = self.output.borrow_mut();
        output.set_drive(new_drive);
        output.tick();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogicValue;

    #[test]
    fn test_pin_test() {
        let mut test_pin = TestPin::new(DriveValue::HighImpedance);
        assert_eq!(
            test_pin.get_output().borrow().get_drive(),
            DriveValue::HighImpedance
        );
        assert_eq!(
            test_pin.get_output().borrow().read(),
            LogicValue::HighImpedance
        );

        test_pin.set_drive(DriveValue::Strong(true));
        assert_eq!(
            test_pin.get_output().borrow().get_drive(),
            DriveValue::Strong(true)
        );
        assert_eq!(
            test_pin.get_output().borrow().read(),
            LogicValue::Driven(true)
        );
    }
}
