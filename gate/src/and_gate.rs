use crate::{NandGate, NotGate};
use device_derive::Device;
use foundation::{AnyDevice, Device, DeviceContainer, Pin};
use std::cell::RefCell;
use std::rc::Rc;

/// A composite gate that performs the AND function. Made from a NAND gate and a NOT gate.
#[derive(Device)]
pub struct AndGate {
    #[child]
    nand_gate: NandGate,
    #[child]
    not_gate: NotGate,
    #[pins]
    input: Vec<Rc<RefCell<Pin>>>,
    #[pin]
    output: Rc<RefCell<Pin>>,
}

impl AndGate {
    /// Construct a new AND gate with the given number of inputs (which must be greater than 2).
    pub fn new(num_inputs: usize) -> Self {
        let nand_gate = NandGate::new(num_inputs);
        let not_gate = NotGate::new();
        let input = nand_gate.get_input().to_vec();
        let output = not_gate.get_output().clone();

        Pin::connect(nand_gate.get_output(), not_gate.get_input());

        Self {
            nand_gate,
            not_gate,
            input,
            output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{settle, DriveValue, LogicValue, TestPin, DRIVE_VALUES};

    #[test]
    fn test_and_gate_2_input() {
        let get_expected =
            |a: &DriveValue, b: &DriveValue| match (LogicValue::from(*a), LogicValue::from(*b)) {
                (LogicValue::Driven(false), LogicValue::Driven(false)) => LogicValue::Driven(false),
                (LogicValue::Driven(false), LogicValue::Driven(true)) => LogicValue::Driven(false),
                (LogicValue::Driven(true), LogicValue::Driven(false)) => LogicValue::Driven(false),
                (LogicValue::Driven(true), LogicValue::Driven(true)) => LogicValue::Driven(true),
                _ => LogicValue::Error,
            };

        let mut and_gate = AndGate::new(2);
        let mut test_pin_a = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin_a.get_output(), &and_gate.get_input()[0]);
        Pin::connect(test_pin_b.get_output(), &and_gate.get_input()[1]);

        for value_a in DRIVE_VALUES.iter() {
            for value_b in DRIVE_VALUES.iter() {
                test_pin_a.set_drive(*value_a);
                test_pin_b.set_drive(*value_b);
                settle(&mut and_gate);
                let actual = and_gate.get_output().borrow().read();
                let expected = get_expected(value_a, value_b);
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn test_and_gate_3_input() {
        let get_expected = |a: &DriveValue, b: &DriveValue, c: &DriveValue| match (
            LogicValue::from(*a),
            LogicValue::from(*b),
            LogicValue::from(*c),
        ) {
            (LogicValue::Driven(false), LogicValue::Driven(false), LogicValue::Driven(false)) => {
                LogicValue::Driven(false)
            }
            (LogicValue::Driven(false), LogicValue::Driven(false), LogicValue::Driven(true)) => {
                LogicValue::Driven(false)
            }
            (LogicValue::Driven(false), LogicValue::Driven(true), LogicValue::Driven(false)) => {
                LogicValue::Driven(false)
            }
            (LogicValue::Driven(false), LogicValue::Driven(true), LogicValue::Driven(true)) => {
                LogicValue::Driven(false)
            }
            (LogicValue::Driven(true), LogicValue::Driven(false), LogicValue::Driven(false)) => {
                LogicValue::Driven(false)
            }
            (LogicValue::Driven(true), LogicValue::Driven(false), LogicValue::Driven(true)) => {
                LogicValue::Driven(false)
            }
            (LogicValue::Driven(true), LogicValue::Driven(true), LogicValue::Driven(false)) => {
                LogicValue::Driven(false)
            }
            (LogicValue::Driven(true), LogicValue::Driven(true), LogicValue::Driven(true)) => {
                LogicValue::Driven(true)
            }
            _ => LogicValue::Error,
        };

        let mut and_gate = AndGate::new(3);
        let mut test_pin_a = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_c = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin_a.get_output(), &and_gate.get_input()[0]);
        Pin::connect(test_pin_b.get_output(), &and_gate.get_input()[1]);
        Pin::connect(test_pin_c.get_output(), &and_gate.get_input()[2]);

        for value_a in DRIVE_VALUES.iter() {
            for value_b in DRIVE_VALUES.iter() {
                for value_c in DRIVE_VALUES.iter() {
                    test_pin_a.set_drive(*value_a);
                    test_pin_b.set_drive(*value_b);
                    test_pin_c.set_drive(*value_c);
                    settle(&mut and_gate);
                    let actual = and_gate.get_output().borrow().read();
                    let expected = get_expected(value_a, value_b, value_c);
                    assert_eq!(expected, actual);
                }
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_bad_and_gate() {
        AndGate::new(1);
    }
}
