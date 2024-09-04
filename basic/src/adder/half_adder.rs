use device_derive::Device;
use foundation::{AnyDevice, Device, DeviceContainer, Pin};
use gate::{AndGate, XorGate};
use std::cell::RefCell;
use std::rc::Rc;

/// A half adder circuit. Adds two one-bit numbers and outputs their sum. Additionaly has a carry
/// output
#[derive(Device)]
pub struct HalfAdder {
    #[child]
    and_gate: AndGate,
    #[child]
    xor_gate: XorGate,
    #[pin]
    a: Rc<RefCell<Pin>>,
    #[pin]
    b: Rc<RefCell<Pin>>,
    #[pin]
    sum: Rc<RefCell<Pin>>,
    #[pin]
    carry: Rc<RefCell<Pin>>,
}

impl HalfAdder {
    /// Creates a new half adder.
    pub fn new() -> Self {
        let and_gate = AndGate::new(2);
        let xor_gate = XorGate::default();
        let a = and_gate.get_input()[0].clone();
        let b = and_gate.get_input()[1].clone();
        let sum = xor_gate.get_output().clone();
        let carry = and_gate.get_output().clone();

        Pin::connect(&and_gate.get_input()[0], xor_gate.get_a_input());
        Pin::connect(&and_gate.get_input()[1], xor_gate.get_b_input());

        Self {
            and_gate,
            xor_gate,
            a,
            b,
            sum,
            carry,
        }
    }
}

impl Default for HalfAdder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_half_adder() {
        use super::*;
        use foundation::{settle, DriveValue, LogicValue, TestPin, DRIVE_VALUES};

        let get_expected =
            |a: &DriveValue, b: &DriveValue| match (LogicValue::from(*a), LogicValue::from(*b)) {
                (LogicValue::Driven(false), LogicValue::Driven(false)) => {
                    (LogicValue::Driven(false), LogicValue::Driven(false))
                }
                (LogicValue::Driven(false), LogicValue::Driven(true)) => {
                    (LogicValue::Driven(true), LogicValue::Driven(false))
                }
                (LogicValue::Driven(true), LogicValue::Driven(false)) => {
                    (LogicValue::Driven(true), LogicValue::Driven(false))
                }
                (LogicValue::Driven(true), LogicValue::Driven(true)) => {
                    (LogicValue::Driven(false), LogicValue::Driven(true))
                }
                _ => (LogicValue::Error, LogicValue::Error),
            };

        let mut half_adder = HalfAdder::default();
        let mut test_pin_a = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin_a.get_output(), half_adder.get_a());
        Pin::connect(test_pin_b.get_output(), half_adder.get_b());

        for value_a in DRIVE_VALUES.iter() {
            for value_b in DRIVE_VALUES.iter() {
                test_pin_a.set_drive(*value_a);
                test_pin_b.set_drive(*value_b);
                settle(&mut half_adder);
                let actual_sum = half_adder.get_sum().borrow().read();
                let actual_carry = half_adder.get_carry().borrow().read();
                let (expected_sum, expected_carry) = get_expected(value_a, value_b);
                assert_eq!(expected_sum, actual_sum);
                assert_eq!(expected_carry, actual_carry);
            }
        }
    }
}
