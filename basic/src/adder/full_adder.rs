use crate::HalfAdder;
use device_derive::Device;
use foundation::{AnyDevice, Device, DeviceContainer, Pin};
use gate::OrGate;
use std::cell::RefCell;
use std::rc::Rc;

/// A full adder circuit, made from two half adder circuits and an or gate. Basically does what a
/// half adder does, but has a `carry_in` input that allows it to add in the presence of a carry
/// bit.
#[derive(Device)]
pub struct FullAdder {
    #[child]
    input_half_adder: HalfAdder,
    #[child]
    carry_half_adder: HalfAdder,
    #[child]
    or_gate: OrGate,
    #[pin]
    a: Rc<RefCell<Pin>>,
    #[pin]
    b: Rc<RefCell<Pin>>,
    #[pin]
    carry_in: Rc<RefCell<Pin>>,
    #[pin]
    sum: Rc<RefCell<Pin>>,
    #[pin]
    carry: Rc<RefCell<Pin>>,
}

impl FullAdder {
    /// Creates a new full adder.
    pub fn new() -> Self {
        let input_half_adder = HalfAdder::new();
        let carry_half_adder = HalfAdder::new();
        let or_gate = OrGate::new(2);
        let a = input_half_adder.get_a().clone();
        let b = input_half_adder.get_b().clone();
        let carry_in = carry_half_adder.get_a().clone();
        let sum = carry_half_adder.get_sum().clone();
        let carry = or_gate.get_output().clone();

        Pin::connect(input_half_adder.get_sum(), carry_half_adder.get_b());
        Pin::connect(carry_half_adder.get_carry(), &or_gate.get_input()[0]);
        Pin::connect(input_half_adder.get_carry(), &or_gate.get_input()[1]);

        Self {
            input_half_adder,
            carry_half_adder,
            or_gate,
            a,
            b,
            carry_in,
            sum,
            carry,
        }
    }
}

impl Default for FullAdder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_full_adder() {
        use super::*;
        use foundation::{settle, DriveValue, LogicValue, TestPin, DRIVE_VALUES};

        let get_expected = |a: &DriveValue, b: &DriveValue, carry_in: &DriveValue| match (
            LogicValue::from(*a),
            LogicValue::from(*b),
            LogicValue::from(*carry_in),
        ) {
            (LogicValue::Driven(false), LogicValue::Driven(false), LogicValue::Driven(false)) => {
                (LogicValue::Driven(false), LogicValue::Driven(false))
            }
            (LogicValue::Driven(false), LogicValue::Driven(false), LogicValue::Driven(true)) => {
                (LogicValue::Driven(true), LogicValue::Driven(false))
            }
            (LogicValue::Driven(false), LogicValue::Driven(true), LogicValue::Driven(false)) => {
                (LogicValue::Driven(true), LogicValue::Driven(false))
            }
            (LogicValue::Driven(false), LogicValue::Driven(true), LogicValue::Driven(true)) => {
                (LogicValue::Driven(false), LogicValue::Driven(true))
            }
            (LogicValue::Driven(true), LogicValue::Driven(false), LogicValue::Driven(false)) => {
                (LogicValue::Driven(true), LogicValue::Driven(false))
            }
            (LogicValue::Driven(true), LogicValue::Driven(false), LogicValue::Driven(true)) => {
                (LogicValue::Driven(false), LogicValue::Driven(true))
            }
            (LogicValue::Driven(true), LogicValue::Driven(true), LogicValue::Driven(false)) => {
                (LogicValue::Driven(false), LogicValue::Driven(true))
            }
            (LogicValue::Driven(true), LogicValue::Driven(true), LogicValue::Driven(true)) => {
                (LogicValue::Driven(true), LogicValue::Driven(true))
            }
            _ => (LogicValue::Error, LogicValue::Error),
        };

        let mut full_adder = FullAdder::default();
        let mut test_pin_a = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_carry_in = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin_a.get_output(), full_adder.get_a());
        Pin::connect(test_pin_b.get_output(), full_adder.get_b());
        Pin::connect(test_pin_carry_in.get_output(), full_adder.get_carry_in());

        for value_a in DRIVE_VALUES.iter() {
            for value_b in DRIVE_VALUES.iter() {
                for value_carry_in in DRIVE_VALUES.iter() {
                    test_pin_a.set_drive(*value_a);
                    test_pin_b.set_drive(*value_b);
                    test_pin_carry_in.set_drive(*value_carry_in);
                    settle(&mut full_adder);
                    let actual_sum = full_adder.get_sum().borrow().read();
                    let actual_carry = full_adder.get_carry().borrow().read();
                    let (expected_sum, expected_carry) =
                        get_expected(value_a, value_b, value_carry_in);
                    assert_eq!(expected_sum, actual_sum);
                    assert_eq!(expected_carry, actual_carry);
                }
            }
        }
    }
}
