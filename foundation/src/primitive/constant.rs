use crate::{AnyDevice, Device, DeviceContainer, DriveValue, Pin};
use device_derive::Device;
use std::cell::RefCell;
use std::rc::Rc;

/// A `Device` consisting of a single `Pin` which is always driving a constant value onto the
/// `Wire`.
#[derive(Device)]
pub struct Constant {
    /// The output `Pin` that is always driving the desired value.
    #[pin]
    output: Rc<RefCell<Pin>>,
}

impl Constant {
    /// Creates a new constant which drives a `DriveValue::Strong` value onto the `Wire`.
    pub fn new_strong(value: bool) -> Self {
        Constant {
            output: Pin::new(DriveValue::Strong(value)),
        }
    }

    /// Creates a new constant which drives a `DriveValue::Strong` value onto the `Wire`.
    pub fn new_weak(value: bool) -> Self {
        Constant {
            output: Pin::new(DriveValue::Weak(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogicValue;

    #[test]
    fn test_strong_true() {
        let strong_true = Constant::new_strong(true);
        assert_eq!(
            strong_true.get_output().borrow().get_drive(),
            DriveValue::Strong(true)
        );
        assert_eq!(
            strong_true.get_output().borrow().read(),
            LogicValue::Driven(true)
        );
    }

    #[test]
    fn test_strong_false() {
        let strong_false = Constant::new_strong(false);
        assert_eq!(
            strong_false.get_output().borrow().get_drive(),
            DriveValue::Strong(false)
        );
        assert_eq!(
            strong_false.get_output().borrow().read(),
            LogicValue::Driven(false)
        );
    }

    #[test]
    fn test_weak_true() {
        let weak_true = Constant::new_weak(true);
        assert_eq!(
            weak_true.get_output().borrow().get_drive(),
            DriveValue::Weak(true)
        );
        assert_eq!(
            weak_true.get_output().borrow().read(),
            LogicValue::Driven(true)
        );
    }

    #[test]
    fn test_weak_false() {
        let weak_false = Constant::new_weak(false);
        assert_eq!(
            weak_false.get_output().borrow().get_drive(),
            DriveValue::Weak(false)
        );
        assert_eq!(
            weak_false.get_output().borrow().read(),
            LogicValue::Driven(false)
        );
    }
}
