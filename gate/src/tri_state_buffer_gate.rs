use crate::NotGate;
use device_derive::Device;
use foundation::{AnyDevice, Constant, Device, DeviceContainer, Pin, Transistor};
use std::cell::RefCell;
use std::rc::Rc;

/// A gate made from transistors that allows a value to be optionally passed through or
/// disconnected, based on the status of an enable pin.
#[derive(Device)]
pub struct TriStateBufferGate {
    #[child]
    strong_true: Constant,
    #[child]
    strong_false: Constant,
    #[child]
    enable_not_gate: NotGate,
    #[child]
    enable_nmos: Transistor,
    #[child]
    enable_pmos: Transistor,
    #[child]
    input_not_gate: NotGate,
    #[child]
    input_nmos: Transistor,
    #[child]
    input_pmos: Transistor,
    #[pin]
    enable: Rc<RefCell<Pin>>,
    #[pin]
    input: Rc<RefCell<Pin>>,
    #[pin]
    output: Rc<RefCell<Pin>>,
}

impl TriStateBufferGate {
    /// Construct a new tri-state buffer gate.
    pub fn new() -> Self {
        let strong_true = Constant::new_strong(true);
        let strong_false = Constant::new_strong(false);
        let enable_not_gate = NotGate::new();
        let enable_nmos = Transistor::new_nmos();
        let enable_pmos = Transistor::new_pmos();
        let input_not_gate = NotGate::new();
        let input_nmos = Transistor::new_nmos();
        let input_pmos = Transistor::new_pmos();
        let enable = enable_not_gate.get_input().clone();
        let input = input_not_gate.get_input().clone();
        let output = input_nmos.get_drain().clone();

        Pin::connect(strong_true.get_output(), enable_pmos.get_source());
        Pin::connect(strong_false.get_output(), enable_nmos.get_source());
        Pin::connect(enable_pmos.get_drain(), input_pmos.get_source());
        Pin::connect(enable_nmos.get_drain(), input_nmos.get_source());
        Pin::connect(input_nmos.get_drain(), input_pmos.get_drain());
        Pin::connect(input_not_gate.get_output(), input_nmos.get_gate());
        Pin::connect(input_not_gate.get_output(), input_pmos.get_gate());
        Pin::connect(enable_not_gate.get_input(), enable_pmos.get_gate());
        Pin::connect(enable_not_gate.get_output(), enable_nmos.get_gate());

        TriStateBufferGate {
            strong_true,
            strong_false,
            enable_not_gate,
            enable_nmos,
            enable_pmos,
            input_not_gate,
            input_nmos,
            input_pmos,
            enable,
            input,
            output,
        }
    }
}

impl Default for TriStateBufferGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{settle, DriveValue, LogicValue, TestPin, DRIVE_VALUES};

    #[test]
    fn test_tri_state_buffer_gate() {
        let get_expected = |enable: &DriveValue, input: &DriveValue| match (
            LogicValue::from(*enable),
            LogicValue::from(*input),
        ) {
            (LogicValue::Driven(false), LogicValue::Driven(false)) => LogicValue::Driven(false),
            (LogicValue::Driven(false), LogicValue::Driven(true)) => LogicValue::Driven(true),
            (LogicValue::Driven(true), LogicValue::Driven(false)) => LogicValue::HighImpedance,
            (LogicValue::Driven(true), LogicValue::Driven(true)) => LogicValue::HighImpedance,
            _ => LogicValue::Error,
        };

        let mut buffer_gate = TriStateBufferGate::default();
        let mut test_pin_enable = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_input = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin_enable.get_output(), buffer_gate.get_enable());
        Pin::connect(test_pin_input.get_output(), buffer_gate.get_input());

        for value_enable in DRIVE_VALUES.iter() {
            for value_input in DRIVE_VALUES.iter() {
                test_pin_enable.set_drive(*value_enable);
                test_pin_input.set_drive(*value_input);
                settle(&mut buffer_gate);
                let actual = buffer_gate.get_output().borrow().read();
                let expected = get_expected(value_enable, value_input);
                assert_eq!(expected, actual);
            }
        }
    }
}
