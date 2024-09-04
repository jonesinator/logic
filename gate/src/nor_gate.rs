use device_derive::Device;
use foundation::{AnyDevice, Constant, Device, DeviceContainer, Pin, Transistor};
use std::cell::RefCell;
use std::iter::zip;
use std::rc::Rc;

/// A gate made from transistors that performs the NOR function.
#[derive(Device)]
pub struct NorGate {
    #[child]
    strong_true: Constant,
    #[child]
    strong_false: Constant,
    #[children]
    nmos: Vec<Transistor>,
    #[children]
    pmos: Vec<Transistor>,
    #[pins]
    input: Vec<Rc<RefCell<Pin>>>,
    #[pin]
    output: Rc<RefCell<Pin>>,
}

impl NorGate {
    /// Construct a new NOR gate with the given number of inputs (which must be greater than 2).
    pub fn new(num_inputs: usize) -> Self {
        // Check the arguments.
        if num_inputs < 2 {
            panic!("NOR gate must have two or more inputs.");
        }

        // Create the children and pins.
        let strong_true = Constant::new_strong(true);
        let strong_false = Constant::new_strong(false);
        let nmos: Vec<Transistor> = (0..num_inputs).map(|_| Transistor::new_nmos()).collect();
        let pmos: Vec<Transistor> = (0..num_inputs).map(|_| Transistor::new_pmos()).collect();
        let input: Vec<Rc<RefCell<Pin>>> = nmos.iter().map(|n| n.get_gate().clone()).collect();
        let output = nmos[num_inputs - 1].get_drain().clone();

        // The first pmos source is connected high.
        Pin::connect(strong_true.get_output(), pmos[0].get_source());

        // All of the nmos sources are connected low.
        nmos.iter()
            .for_each(|n| Pin::connect(strong_false.get_output(), n.get_source()));

        // The remaining pmos are connected in a chain.
        zip(pmos.iter(), pmos[1..].iter())
            .for_each(|(p0, p1)| Pin::connect(p0.get_drain(), p1.get_source()));

        // All of the nmos drains are connected together (by connecting them all to drain 0).
        nmos[1..]
            .iter()
            .for_each(|n| Pin::connect(n.get_drain(), nmos[0].get_drain()));

        // The nmos drains are connected to the final pmos drain.
        Pin::connect(nmos[0].get_drain(), pmos[num_inputs - 1].get_drain());

        // All of the nmos and pmos gates are connected together.
        zip(nmos.iter(), pmos.iter()).for_each(|(n, p)| Pin::connect(n.get_gate(), p.get_gate()));

        Self {
            strong_true,
            strong_false,
            nmos,
            pmos,
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
    fn test_nor_gate_2_input() {
        let get_expected =
            |a: &DriveValue, b: &DriveValue| match (LogicValue::from(*a), LogicValue::from(*b)) {
                (LogicValue::Driven(false), LogicValue::Driven(false)) => LogicValue::Driven(true),
                (LogicValue::Driven(false), LogicValue::Driven(true)) => LogicValue::Driven(false),
                (LogicValue::Driven(true), LogicValue::Driven(false)) => LogicValue::Driven(false),
                (LogicValue::Driven(true), LogicValue::Driven(true)) => LogicValue::Driven(false),
                _ => LogicValue::Error,
            };

        let mut nor_gate = NorGate::new(2);
        let mut test_pin_a = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin_a.get_output(), &nor_gate.get_input()[0]);
        Pin::connect(test_pin_b.get_output(), &nor_gate.get_input()[1]);

        for value_a in DRIVE_VALUES.iter() {
            for value_b in DRIVE_VALUES.iter() {
                test_pin_a.set_drive(*value_a);
                test_pin_b.set_drive(*value_b);
                settle(&mut nor_gate);
                let actual = nor_gate.get_output().borrow().read();
                let expected = get_expected(value_a, value_b);
                assert_eq!(expected, actual);
            }
        }
    }

    #[test]
    fn test_nor_gate_3_input() {
        let get_expected = |a: &DriveValue, b: &DriveValue, c: &DriveValue| match (
            LogicValue::from(*a),
            LogicValue::from(*b),
            LogicValue::from(*c),
        ) {
            (LogicValue::Driven(false), LogicValue::Driven(false), LogicValue::Driven(false)) => {
                LogicValue::Driven(true)
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
                LogicValue::Driven(false)
            }
            _ => LogicValue::Error,
        };

        let mut nor_gate = NorGate::new(3);
        let mut test_pin_a = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_c = TestPin::new(DriveValue::HighImpedance);
        Pin::connect(test_pin_a.get_output(), &nor_gate.get_input()[0]);
        Pin::connect(test_pin_b.get_output(), &nor_gate.get_input()[1]);
        Pin::connect(test_pin_c.get_output(), &nor_gate.get_input()[2]);

        for value_a in DRIVE_VALUES.iter() {
            for value_b in DRIVE_VALUES.iter() {
                for value_c in DRIVE_VALUES.iter() {
                    test_pin_a.set_drive(*value_a);
                    test_pin_b.set_drive(*value_b);
                    test_pin_c.set_drive(*value_c);
                    settle(&mut nor_gate);
                    let actual = nor_gate.get_output().borrow().read();
                    let expected = get_expected(value_a, value_b, value_c);
                    assert_eq!(expected, actual);
                }
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_bad_nor_gate() {
        NorGate::new(1);
    }
}
