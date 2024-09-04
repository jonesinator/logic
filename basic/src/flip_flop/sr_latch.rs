use device_derive::Device;
use foundation::{AnyDevice, Device, DeviceContainer, Pin};
use gate::NorGate;
use std::cell::RefCell;
use std::rc::Rc;

/// The fundamental structure for storing information in digital logic. An SR latch can "remember"
/// whether it has been set or reset.
#[derive(Device)]
pub struct SrLatch {
    #[child]
    nor_gate_1: NorGate,

    #[child]
    nor_gate_2: NorGate,

    #[pin]
    set: Rc<RefCell<Pin>>,

    #[pin]
    reset: Rc<RefCell<Pin>>,

    #[pin]
    output: Rc<RefCell<Pin>>,

    #[pin]
    output_inverted: Rc<RefCell<Pin>>,
}

impl SrLatch {
    /// Creates a new SR Latch.
    pub fn new() -> Self {
        let nor_gate_1 = NorGate::new(2);
        let nor_gate_2 = NorGate::new(2);
        let reset = nor_gate_1.get_input()[0].clone();
        let set = nor_gate_2.get_input()[1].clone();
        let output = nor_gate_1.get_output().clone();
        let output_inverted = nor_gate_2.get_output().clone();

        Pin::connect(nor_gate_1.get_output(), &nor_gate_2.get_input()[0]);
        Pin::connect(nor_gate_2.get_output(), &nor_gate_1.get_input()[1]);

        Self {
            nor_gate_1,
            nor_gate_2,
            set,
            reset,
            output,
            output_inverted,
        }
    }
}

impl Default for SrLatch {
    fn default() -> Self {
        SrLatch::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{settle, Constant, DriveValue, LogicValue, TestPin};

    #[test]
    fn test_sr_latch() {
        let weak_true = Constant::new_weak(true);
        let weak_false = Constant::new_weak(false);
        let mut latch = SrLatch::default();
        let mut test_pin_set = TestPin::new(DriveValue::Strong(false));
        let mut test_pin_reset = TestPin::new(DriveValue::Strong(false));

        let check = |latch: &SrLatch, value: bool| {
            assert_eq!(
                latch.get_output().borrow().read(),
                LogicValue::Driven(value)
            );
            assert_eq!(
                latch.get_output_inverted().borrow().read(),
                LogicValue::Driven(!value)
            );
        };

        Pin::connect(weak_false.get_output(), latch.get_output());
        Pin::connect(test_pin_set.get_output(), latch.get_set());
        Pin::connect(weak_true.get_output(), latch.get_output_inverted());
        Pin::connect(test_pin_reset.get_output(), latch.get_reset());

        settle(&mut latch);
        check(&latch, false);

        (0..3).for_each(|_| {
            test_pin_set.set_drive(DriveValue::Strong(true));
            settle(&mut latch);
            check(&latch, true);

            test_pin_set.set_drive(DriveValue::Strong(false));
            settle(&mut latch);
            check(&latch, true);

            test_pin_reset.set_drive(DriveValue::Strong(true));
            settle(&mut latch);
            check(&latch, false);

            test_pin_reset.set_drive(DriveValue::Strong(false));
            settle(&mut latch);
            check(&latch, false);
        });
    }
}
