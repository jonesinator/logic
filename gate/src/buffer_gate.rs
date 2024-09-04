use crate::NotGate;
use device_derive::Device;
use foundation::{AnyDevice, Device, DeviceContainer, Pin};
use std::cell::RefCell;
use std::rc::Rc;

/// A gate made from transistors that performs the identity function. Made from two NOT gates.
#[derive(Device)]
pub struct BufferGate {
    #[children]
    not_gate: Vec<NotGate>,
    #[pin]
    input: Rc<RefCell<Pin>>,
    #[pin]
    output: Rc<RefCell<Pin>>,
}

impl BufferGate {
    /// Construct a new buffer gate.
    pub fn new() -> Self {
        let not_gate = vec![NotGate::new(), NotGate::new()];
        let input = not_gate[0].get_input().clone();
        let output = not_gate[1].get_output().clone();

        Pin::connect(not_gate[0].get_output(), not_gate[1].get_input());

        Self {
            not_gate,
            input,
            output,
        }
    }
}

impl Default for BufferGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{settle, DriveValue, LogicValue, TestPin, DRIVE_VALUES};

    #[test]
    fn test_buffer_gate() {
        let get_expected = |a: &DriveValue| match LogicValue::from(*a) {
            LogicValue::Driven(false) => LogicValue::Driven(false),
            LogicValue::Driven(true) => LogicValue::Driven(true),
            _ => LogicValue::Error,
        };

        let mut buffer_gate = BufferGate::default();
        let mut test_pin = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin.get_output(), buffer_gate.get_input());

        for value in DRIVE_VALUES.iter() {
            test_pin.set_drive(*value);
            settle(&mut buffer_gate);
            let actual = buffer_gate.get_output().borrow().read();
            let expected = get_expected(value);
            assert_eq!(expected, actual);
        }
    }
}
