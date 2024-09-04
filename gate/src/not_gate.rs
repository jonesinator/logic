use device_derive::Device;
use foundation::{AnyDevice, Constant, Device, DeviceContainer, Pin, Transistor};
use std::cell::RefCell;
use std::rc::Rc;

/// A gate made from transistors that performs the NOT function.
#[derive(Device)]
pub struct NotGate {
    #[child]
    constant_true: Constant,
    #[child]
    constant_false: Constant,
    #[child]
    nmos: Transistor,
    #[child]
    pmos: Transistor,
    #[pin]
    input: Rc<RefCell<Pin>>,
    #[pin]
    output: Rc<RefCell<Pin>>,
}

impl NotGate {
    /// Constructs a new NOT gate.
    pub fn new() -> Self {
        let constant_true = Constant::new_strong(true);
        let constant_false = Constant::new_strong(false);
        let nmos = Transistor::new_nmos();
        let pmos = Transistor::new_pmos();
        let input = nmos.get_gate().clone();
        let output = nmos.get_drain().clone();

        Pin::connect(nmos.get_gate(), pmos.get_gate());
        Pin::connect(nmos.get_drain(), pmos.get_drain());
        Pin::connect(constant_false.get_output(), nmos.get_source());
        Pin::connect(constant_true.get_output(), pmos.get_source());

        Self {
            constant_true,
            constant_false,
            nmos,
            pmos,
            input,
            output,
        }
    }
}

impl Default for NotGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{settle, DriveValue, LogicValue, TestPin, DRIVE_VALUES};

    #[test]
    fn test_not_gate() {
        let get_expected = |a: &DriveValue| match LogicValue::from(*a) {
            LogicValue::Driven(false) => LogicValue::Driven(true),
            LogicValue::Driven(true) => LogicValue::Driven(false),
            _ => LogicValue::Error,
        };

        let mut not_gate = NotGate::default();
        let mut test_pin = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin.get_output(), not_gate.get_input());

        for value in DRIVE_VALUES.iter() {
            test_pin.set_drive(*value);
            settle(&mut not_gate);
            let actual = not_gate.get_output().borrow().read();
            let expected = get_expected(value);
            assert_eq!(expected, actual);
        }
    }
}
