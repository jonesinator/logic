use crate::FullAdder;
use device_derive::Device;
use foundation::{AnyDevice, Constant, Device, DeviceContainer, Pin};
use std::cell::RefCell;
use std::iter::zip;
use std::rc::Rc;

/// A simple and slow device that can add two n-bit unsigned integers. It is comprised of a number
/// of full adders in a chain.
#[derive(Device)]
pub struct RippleCarryAdder {
    #[child]
    strong_false: Constant,
    #[children]
    adders: Vec<FullAdder>,
    #[pins]
    input_a: Vec<Rc<RefCell<Pin>>>,
    #[pins]
    input_b: Vec<Rc<RefCell<Pin>>>,
    #[pins]
    sum: Vec<Rc<RefCell<Pin>>>,
    #[pin]
    overflow: Rc<RefCell<Pin>>,
}

impl RippleCarryAdder {
    /// Creates a new `RippleCarryAdder` of the desired width.
    pub fn new(width: usize) -> Self {
        if width == 0 {
            panic!("RippleCarryAdder width must be non-zero.")
        }

        let strong_false = Constant::new_strong(false);
        let adders: Vec<FullAdder> = (0..width).map(|_| FullAdder::new()).collect();
        let input_a: Vec<Rc<RefCell<Pin>>> = adders.iter().map(|a| a.get_a().clone()).collect();
        let input_b: Vec<Rc<RefCell<Pin>>> = adders.iter().map(|a| a.get_b().clone()).collect();
        let sum: Vec<Rc<RefCell<Pin>>> = adders.iter().map(|a| a.get_sum().clone()).collect();
        let overflow = adders.last().unwrap().get_carry().clone();

        Pin::connect(strong_false.get_output(), adders[0].get_carry_in());
        zip(adders.iter(), adders[1..].iter()).for_each(|(a0, a1)| {
            Pin::connect(a0.get_carry(), a1.get_carry_in());
        });

        RippleCarryAdder {
            strong_false,
            adders,
            input_a,
            input_b,
            sum,
            overflow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{settle, DriveValue, LogicValue as LV, Pin, TestPin, DRIVE_VALUES};

    // This is a low-level tests that tests error and high impedance conditions as well. Because of
    // this, the truth table is quite large.
    #[test]
    fn test_ripple_carry_adder_2_bit() {
        let expected = |a0: &DriveValue, a1: &DriveValue, b0: &DriveValue, b1: &DriveValue| match (
            LV::from(*a1),
            LV::from(*a0),
            LV::from(*b1),
            LV::from(*b0),
        ) {
            /* Valid logic cases. */

            /* 0b00 + 0b00 = 0b00 */
            (LV::Driven(false), LV::Driven(false), LV::Driven(false), LV::Driven(false)) => {
                (LV::Driven(false), LV::Driven(false), LV::Driven(false))
            }

            /* 0b00 + 0b01 = 0b01 */
            (LV::Driven(false), LV::Driven(false), LV::Driven(false), LV::Driven(true)) => {
                (LV::Driven(false), LV::Driven(true), LV::Driven(false))
            }

            /* 0b01 + 0b00 = 0b01 */
            (LV::Driven(false), LV::Driven(true), LV::Driven(false), LV::Driven(false)) => {
                (LV::Driven(false), LV::Driven(true), LV::Driven(false))
            }

            /* 0b01 + 0b01 = 0b10 */
            (LV::Driven(false), LV::Driven(true), LV::Driven(false), LV::Driven(true)) => {
                (LV::Driven(true), LV::Driven(false), LV::Driven(false))
            }

            /* 0b00 + 0b10 = 0b10 */
            (LV::Driven(false), LV::Driven(false), LV::Driven(true), LV::Driven(false)) => {
                (LV::Driven(true), LV::Driven(false), LV::Driven(false))
            }

            /* 0b00 + 0b11 = 0b11 */
            (LV::Driven(false), LV::Driven(false), LV::Driven(true), LV::Driven(true)) => {
                (LV::Driven(true), LV::Driven(true), LV::Driven(false))
            }

            /* 0b01 + 0b10 = 0b11 */
            (LV::Driven(false), LV::Driven(true), LV::Driven(true), LV::Driven(false)) => {
                (LV::Driven(true), LV::Driven(true), LV::Driven(false))
            }

            /* 0b01 + 0b11 = 0b00 w/overflow */
            (LV::Driven(false), LV::Driven(true), LV::Driven(true), LV::Driven(true)) => {
                (LV::Driven(false), LV::Driven(false), LV::Driven(true))
            }

            /* 0b10 + 0b00 = 0b10 */
            (LV::Driven(true), LV::Driven(false), LV::Driven(false), LV::Driven(false)) => {
                (LV::Driven(true), LV::Driven(false), LV::Driven(false))
            }

            /* 0b10 + 0b01 = 0b11 */
            (LV::Driven(true), LV::Driven(false), LV::Driven(false), LV::Driven(true)) => {
                (LV::Driven(true), LV::Driven(true), LV::Driven(false))
            }

            /* 0b11 + 0b00 = 0b11 */
            (LV::Driven(true), LV::Driven(true), LV::Driven(false), LV::Driven(false)) => {
                (LV::Driven(true), LV::Driven(true), LV::Driven(false))
            }

            /* 0b11 + 0b01 = 0b00 w/overflow */
            (LV::Driven(true), LV::Driven(true), LV::Driven(false), LV::Driven(true)) => {
                (LV::Driven(false), LV::Driven(false), LV::Driven(true))
            }

            /* 0b10 + 0b10 = 0b00 w/overflow */
            (LV::Driven(true), LV::Driven(false), LV::Driven(true), LV::Driven(false)) => {
                (LV::Driven(false), LV::Driven(false), LV::Driven(true))
            }

            /* 0b10 + 0b11 = 0b00 w/overflow */
            (LV::Driven(true), LV::Driven(false), LV::Driven(true), LV::Driven(true)) => {
                (LV::Driven(false), LV::Driven(true), LV::Driven(true))
            }

            /* 0b11 + 0b10 = 0b00 w/overflow */
            (LV::Driven(true), LV::Driven(true), LV::Driven(true), LV::Driven(false)) => {
                (LV::Driven(false), LV::Driven(true), LV::Driven(true))
            }

            /* 0b11 + 0b11 = 0b00 w/overflow */
            (LV::Driven(true), LV::Driven(true), LV::Driven(true), LV::Driven(true)) => {
                (LV::Driven(true), LV::Driven(false), LV::Driven(true))
            }

            /* Error cases with some non-error outputs. */

            /* 0bn0 + 0bx0 = 0bx0 w/overflow=x */
            (
                LV::Driven(_) | LV::HighImpedance | LV::Error,
                LV::Driven(false),
                LV::HighImpedance | LV::Error,
                LV::Driven(false),
            ) => (LV::Error, LV::Driven(false), LV::Error),

            /* 0bn0 + 0bx1 = 0bx1 w/overflow=x */
            (
                LV::Driven(_) | LV::HighImpedance | LV::Error,
                LV::Driven(false),
                LV::HighImpedance | LV::Error,
                LV::Driven(true),
            ) => (LV::Error, LV::Driven(true), LV::Error),

            /* 0bn1 + 0bx0 = 0bx0 w/overflow=x */
            (
                LV::Driven(_) | LV::HighImpedance | LV::Error,
                LV::Driven(true),
                LV::HighImpedance | LV::Error,
                LV::Driven(false),
            ) => (LV::Error, LV::Driven(true), LV::Error),

            /* 0bn1 + 0bx1 = 0bx0 w/overflow=x */
            (
                LV::Driven(_) | LV::HighImpedance | LV::Error,
                LV::Driven(true),
                LV::HighImpedance | LV::Error,
                LV::Driven(true),
            ) => (LV::Error, LV::Driven(false), LV::Error),

            /* 0bx0 + 0bn0 = 0bx0 w/overflow=x */
            (
                LV::HighImpedance | LV::Error,
                LV::Driven(false),
                LV::Driven(_),
                LV::Driven(false),
            ) => (LV::Error, LV::Driven(false), LV::Error),

            /* 0bx0 + 0bn1 = 0bx1 w/overflow=x */
            (LV::HighImpedance | LV::Error, LV::Driven(false), LV::Driven(_), LV::Driven(true)) => {
                (LV::Error, LV::Driven(true), LV::Error)
            }

            /* 0bx1 + 0bn0 = 0bx0 w/overflow=x */
            (LV::HighImpedance | LV::Error, LV::Driven(true), LV::Driven(_), LV::Driven(false)) => {
                (LV::Error, LV::Driven(true), LV::Error)
            }

            /* 0bx1 + 0bx1 = 0bx0 w/overflow=x */
            (LV::HighImpedance | LV::Error, LV::Driven(true), LV::Driven(_), LV::Driven(true)) => {
                (LV::Error, LV::Driven(false), LV::Error)
            }

            _ => (LV::Error, LV::Error, LV::Error),
        };

        let mut adder = RippleCarryAdder::new(2);
        let mut test_pin_a0 = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_a1 = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b0 = TestPin::new(DriveValue::HighImpedance);
        let mut test_pin_b1 = TestPin::new(DriveValue::HighImpedance);

        Pin::connect(test_pin_a0.get_output(), &adder.get_input_a()[0]);
        Pin::connect(test_pin_a1.get_output(), &adder.get_input_a()[1]);
        Pin::connect(test_pin_b0.get_output(), &adder.get_input_b()[0]);
        Pin::connect(test_pin_b1.get_output(), &adder.get_input_b()[1]);

        for a0 in DRIVE_VALUES.iter() {
            for a1 in DRIVE_VALUES.iter() {
                for b0 in DRIVE_VALUES.iter() {
                    for b1 in DRIVE_VALUES.iter() {
                        test_pin_a0.set_drive(*a0);
                        test_pin_a1.set_drive(*a1);
                        test_pin_b0.set_drive(*b0);
                        test_pin_b1.set_drive(*b1);
                        settle(&mut adder);
                        let actual_s0 = adder.get_sum()[0].borrow().read();
                        let actual_s1 = adder.get_sum()[1].borrow().read();
                        let actual_overflow = adder.get_overflow().borrow().read();
                        let (expected_s1, expected_s0, expected_overflow) =
                            expected(a0, a1, b0, b1);
                        assert_eq!(actual_s0, expected_s0);
                        assert_eq!(actual_s1, expected_s1);
                        assert_eq!(actual_overflow, expected_overflow);
                    }
                }
            }
        }
    }

    // This is a high-level tests that tests that the logic in the adders is actually equivalent to
    // unsigned integer addition.
    #[test]
    fn test_ripple_carry_adder_logic() {
        test_ripple_carry_adder_n_bit(1);
        test_ripple_carry_adder_n_bit(2);
        test_ripple_carry_adder_n_bit(3);
        test_ripple_carry_adder_n_bit(4);
    }

    #[test]
    #[should_panic]
    fn test_bad_ripple_carry_adder() {
        RippleCarryAdder::new(0);
    }

    // Utility function that fully tests the truth table (not including error conditions) for an
    // n-bit ripple-carry adder.
    fn test_ripple_carry_adder_n_bit(width: usize) {
        let max_value = 2usize.pow(width as u32);
        let set_pins = |pins: &mut Vec<TestPin>, value: usize| {
            for (index, pin) in pins.iter_mut().enumerate() {
                pin.set_drive(DriveValue::Strong(
                    value / 2usize.pow(index as u32) % 2 == 1,
                ));
            }
        };
        let read_pins = |pins: &Vec<Rc<RefCell<Pin>>>| {
            let mut sum = 0;
            for (index, pin) in pins.iter().enumerate() {
                if pin.borrow().read() == LV::Driven(true) {
                    sum += 2usize.pow(index as u32);
                }
            }
            sum
        };

        let mut adder = RippleCarryAdder::new(width);

        let mut test_pins_a: Vec<TestPin> = (0..width)
            .map(|_| TestPin::new(DriveValue::HighImpedance))
            .collect();
        for (test_pin, input_pin) in zip(test_pins_a.iter(), adder.get_input_a().iter()) {
            Pin::connect(test_pin.get_output(), input_pin);
        }

        let mut test_pins_b: Vec<TestPin> = (0..width)
            .map(|_| TestPin::new(DriveValue::HighImpedance))
            .collect();
        for (test_pin, input_pin) in zip(test_pins_b.iter(), adder.get_input_b().iter()) {
            Pin::connect(test_pin.get_output(), input_pin);
        }

        for value_a in 0..max_value {
            for value_b in 0..max_value {
                set_pins(&mut test_pins_a, value_a);
                set_pins(&mut test_pins_b, value_b);
                settle(&mut adder);
                let actual_sum = read_pins(adder.get_sum());
                let actual_overflow = adder.get_overflow().borrow().read();
                assert_eq!(actual_sum, (value_a + value_b) % max_value);
                assert_eq!(actual_overflow, LV::Driven(value_a + value_b >= max_value));
            }
        }
    }
}
