use crate::NotGate;
use device_derive::Device;
use foundation::{AnyDevice, Constant, Device, DeviceContainer, Pin, Transistor};
use std::cell::RefCell;
use std::rc::Rc;

/// A gate made from transistors that performs the XOR function.
#[derive(Device)]
pub struct XorGate {
    #[child]
    strong_true: Constant,
    #[child]
    strong_false: Constant,
    #[child]
    a_not_gate: NotGate,
    #[child]
    a_nmos: Transistor,
    #[child]
    a_inverted_nmos: Transistor,
    #[child]
    a_pmos: Transistor,
    #[child]
    a_inverted_pmos: Transistor,
    #[child]
    b_not_gate: NotGate,
    #[child]
    b_nmos_1: Transistor,
    #[child]
    b_nmos_2: Transistor,
    #[child]
    b_pmos: Transistor,
    #[child]
    b_inverted_pmos: Transistor,
    #[pin]
    a_input: Rc<RefCell<Pin>>,
    #[pin]
    b_input: Rc<RefCell<Pin>>,
    #[pin]
    output: Rc<RefCell<Pin>>,
}

impl XorGate {
    /// Construct a new 2-input XOR gate.
    pub fn new() -> Self {
        let strong_true = Constant::new_strong(true);
        let strong_false = Constant::new_strong(false);
        let a_not_gate = NotGate::new();
        let a_nmos = Transistor::new_nmos();
        let a_inverted_nmos = Transistor::new_nmos();
        let a_pmos = Transistor::new_pmos();
        let a_inverted_pmos = Transistor::new_pmos();
        let a_input = a_not_gate.get_input().clone();
        let b_not_gate = NotGate::new();
        let b_nmos_1 = Transistor::new_nmos();
        let b_nmos_2 = Transistor::new_nmos();
        let b_pmos = Transistor::new_pmos();
        let b_inverted_pmos = Transistor::new_pmos();
        let b_input = b_not_gate.get_input().clone();
        let output = a_nmos.get_drain().clone();

        Pin::connect(strong_true.get_output(), a_pmos.get_source());
        Pin::connect(a_pmos.get_drain(), b_inverted_pmos.get_source());
        Pin::connect(b_inverted_pmos.get_drain(), a_nmos.get_drain());
        Pin::connect(a_nmos.get_source(), b_nmos_1.get_drain());
        Pin::connect(b_nmos_1.get_source(), strong_false.get_output());

        Pin::connect(a_pmos.get_gate(), a_not_gate.get_output());
        Pin::connect(b_inverted_pmos.get_gate(), &b_input);
        Pin::connect(a_nmos.get_gate(), &a_input);
        Pin::connect(b_nmos_1.get_gate(), &b_input);

        Pin::connect(strong_true.get_output(), a_inverted_pmos.get_source());
        Pin::connect(a_inverted_pmos.get_drain(), b_pmos.get_source());
        Pin::connect(b_pmos.get_drain(), a_inverted_nmos.get_drain());
        Pin::connect(a_inverted_nmos.get_source(), b_nmos_2.get_drain());
        Pin::connect(b_nmos_2.get_source(), strong_false.get_output());

        Pin::connect(a_inverted_pmos.get_gate(), &a_input);
        Pin::connect(b_pmos.get_gate(), b_not_gate.get_output());
        Pin::connect(a_inverted_nmos.get_gate(), a_not_gate.get_output());
        Pin::connect(b_nmos_2.get_gate(), b_not_gate.get_output());

        Pin::connect(a_nmos.get_drain(), a_inverted_nmos.get_drain());

        Self {
            strong_true,
            strong_false,
            a_not_gate,
            a_nmos,
            a_inverted_nmos,
            a_pmos,
            a_inverted_pmos,
            a_input,
            b_not_gate,
            b_nmos_1,
            b_nmos_2,
            b_pmos,
            b_inverted_pmos,
            b_input,
            output,
        }
    }
}

impl Default for XorGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{settle, DriveValue, LogicValue, TestPin, DRIVE_VALUES};

    #[test]
    fn test_xor_gate() {
        let get_expected =
            |a: &DriveValue, b: &DriveValue| match (LogicValue::from(*a), LogicValue::from(*b)) {
                (LogicValue::Driven(false), LogicValue::Driven(false)) => LogicValue::Driven(false),
                (LogicValue::Driven(false), LogicValue::Driven(true)) => LogicValue::Driven(true),
                (LogicValue::Driven(true), LogicValue::Driven(false)) => LogicValue::Driven(true),
                (LogicValue::Driven(true), LogicValue::Driven(true)) => LogicValue::Driven(false),
                _ => LogicValue::Error,
            };

        let mut xor_gate = XorGate::default();
        let mut test_pin_a = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin_a.get_output(), xor_gate.get_a_input());
        Pin::connect(test_pin_b.get_output(), xor_gate.get_b_input());

        for value_a in DRIVE_VALUES.iter() {
            for value_b in DRIVE_VALUES.iter() {
                test_pin_a.set_drive(*value_a);
                test_pin_b.set_drive(*value_b);
                settle(&mut xor_gate);
                let actual = xor_gate.get_output().borrow().read();
                let expected = get_expected(value_a, value_b);
                assert_eq!(expected, actual);
            }
        }
    }
}
