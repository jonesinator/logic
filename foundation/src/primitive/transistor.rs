use crate::{AnyDevice, Device, DeviceContainer, DriveValue, LogicValue, Pin};
use device_derive::Device;
use std::cell::RefCell;
use std::rc::Rc;

/// Represents either an NMOS or PMOS transistor in an eletronic circuit.
#[derive(Device)]
pub struct Transistor {
    /// The source pin of the transistor. Current can flow from source to drain when the gate is
    /// activated.
    #[pin]
    source: Rc<RefCell<Pin>>,

    /// The gate pin of the transistor. Controls whether or not current can flow from source to
    /// drain.
    #[pin]
    gate: Rc<RefCell<Pin>>,

    /// The drain pin of the transistor. Current can flow from source to drain when the gate is
    /// activated.
    #[pin]
    drain: Rc<RefCell<Pin>>,

    /// The activation logic level of the transistor. If `true`, then the transistor will allow
    /// conductivity when the gate pin is driven `true`, as is the case for NMOS transistors. If
    /// `false`, then the transistor will allow conductivity when the gate pin is driven `false`,
    /// as is the case for PMOS transistors.
    activation: bool,

    /// Used to delay gate errors by one tick.
    ///
    /// This is a somewhat unfortunate hack to get circuits with feedback working when transient
    /// errors are expected. Transient errors are expected due to the way CMOS transistors work in
    /// the simulation, for a short period we can expect a transient `LogicValue::Error` or
    /// `LogicValue::HighImpedance` output for a tick on some of the fundamental logic gates,
    /// because some transitors are wired in parallel and some in series, and thus have different
    /// propagation delays.
    ///
    /// On the first tick where we notice the gate pin is `LogicValue::HighImpedance` or
    /// `LogicValue::Error` then we will set this `error_hysteresis` flag, and instead of driving
    /// `LogicValue::Error` immediately, we'll keep driving whatever we're already driving. On the
    /// next tick, if the gate pin is still `LogicValue::HighImpedance` or `LogicValue::Error`,
    /// _then_ we will start outputing `LogicValue::Error`. This means error conditions must persist
    /// for at least two ticks before being reported. Errors that last only one tick should be
    /// invisible to the rest of the system, and this seems to be sufficient for everything to work.
    error_hysteresis: bool,
}

impl Transistor {
    /// Creates a new NMOS transistor, not connected to anything.
    pub fn new_nmos() -> Self {
        Self::new(true)
    }

    /// Creates a new PMOS transistor, not connected to anything.
    pub fn new_pmos() -> Self {
        Self::new(false)
    }

    /// Gets the activation level for the `Transistor`. Used to distinguish NMOS and PMOS.
    pub fn get_activation(&self) -> bool {
        self.activation
    }

    /// Updates the drive of the drain `Pin` based on the states of the gate and source `Pin`s.
    /// Returns `true` if the `Transistor`'s drain drive value changes, or if this is the first tick
    /// where the gate is high impedance / error and error hysteresis is being applied. Returns
    /// `false` otherwise.
    pub(crate) fn tick(&mut self) -> bool {
        let current = LogicValue::from(self.drain.borrow().get_drive());
        let next = match self.gate.borrow().read() {
            LogicValue::Driven(drive) => {
                self.error_hysteresis = false;
                if drive == self.activation {
                    self.source.borrow().read()
                } else {
                    LogicValue::HighImpedance
                }
            }
            _ => {
                if !self.error_hysteresis {
                    self.error_hysteresis = true;
                    return true;
                } else {
                    LogicValue::Error
                }
            }
        };

        self.drain.borrow_mut().set_drive(next.into());
        current != next
    }

    // Private generic function for creating transistors.
    fn new(activation: bool) -> Self {
        Self {
            source: Pin::new(DriveValue::HighImpedance),
            gate: Pin::new(DriveValue::HighImpedance),
            drain: Pin::new(DriveValue::HighImpedance),
            activation,
            error_hysteresis: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DRIVE_VALUES;

    #[test]
    fn test_nmos() {
        // We need two truth tables since error values are delayed by one tick.
        let get_expected_1 = |gate: &DriveValue, source: &DriveValue| match (gate, source) {
            (DriveValue::Strong(true), DriveValue::Strong(true)) => DriveValue::Strong(true),
            (DriveValue::Strong(true), DriveValue::Strong(false)) => DriveValue::Strong(false),
            (DriveValue::Strong(true), DriveValue::Weak(true)) => DriveValue::Strong(true),
            (DriveValue::Strong(true), DriveValue::Weak(false)) => DriveValue::Strong(false),
            (DriveValue::Strong(true), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Error) => DriveValue::Error,
            (DriveValue::Strong(false), DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Strong(true)) => DriveValue::Strong(true),
            (DriveValue::Weak(true), DriveValue::Strong(false)) => DriveValue::Strong(false),
            (DriveValue::Weak(true), DriveValue::Weak(true)) => DriveValue::Strong(true),
            (DriveValue::Weak(true), DriveValue::Weak(false)) => DriveValue::Strong(false),
            (DriveValue::Weak(true), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Error) => DriveValue::Error,
            (DriveValue::Weak(false), DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Error) => DriveValue::HighImpedance,
        };

        let get_expected_2 = |gate: &DriveValue, source: &DriveValue| match (gate, source) {
            (DriveValue::Strong(true), DriveValue::Strong(true)) => DriveValue::Strong(true),
            (DriveValue::Strong(true), DriveValue::Strong(false)) => DriveValue::Strong(false),
            (DriveValue::Strong(true), DriveValue::Weak(true)) => DriveValue::Strong(true),
            (DriveValue::Strong(true), DriveValue::Weak(false)) => DriveValue::Strong(false),
            (DriveValue::Strong(true), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Error) => DriveValue::Error,
            (DriveValue::Strong(false), DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Strong(true)) => DriveValue::Strong(true),
            (DriveValue::Weak(true), DriveValue::Strong(false)) => DriveValue::Strong(false),
            (DriveValue::Weak(true), DriveValue::Weak(true)) => DriveValue::Strong(true),
            (DriveValue::Weak(true), DriveValue::Weak(false)) => DriveValue::Strong(false),
            (DriveValue::Weak(true), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Error) => DriveValue::Error,
            (DriveValue::Weak(false), DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Strong(true)) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Strong(false)) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Weak(true)) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Weak(false)) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::HighImpedance) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Error) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Strong(true)) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Strong(false)) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Weak(true)) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Weak(false)) => DriveValue::Error,
            (DriveValue::Error, DriveValue::HighImpedance) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Error) => DriveValue::Error,
        };

        for gate in DRIVE_VALUES.iter() {
            for source in DRIVE_VALUES.iter() {
                let mut nmos = Transistor::new_nmos();

                let actual_1 = tick_transistor(&mut nmos, gate, source);
                let expected_1 = get_expected_1(gate, source);
                assert_eq!(actual_1, expected_1);

                let actual_2 = tick_transistor(&mut nmos, gate, source);
                let expected_2 = get_expected_2(gate, source);
                assert_eq!(actual_2, expected_2);

                assert!(nmos.get_activation());
            }
        }
    }

    #[test]
    fn test_pmos() {
        // We need two truth tables since error values are delayed by one tick.
        let get_expected_1 = |gate: &DriveValue, source: &DriveValue| match (gate, source) {
            (DriveValue::Strong(true), DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Strong(true)) => DriveValue::Strong(true),
            (DriveValue::Strong(false), DriveValue::Strong(false)) => DriveValue::Strong(false),
            (DriveValue::Strong(false), DriveValue::Weak(true)) => DriveValue::Strong(true),
            (DriveValue::Strong(false), DriveValue::Weak(false)) => DriveValue::Strong(false),
            (DriveValue::Strong(false), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Error) => DriveValue::Error,
            (DriveValue::Weak(true), DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Strong(true)) => DriveValue::Strong(true),
            (DriveValue::Weak(false), DriveValue::Strong(false)) => DriveValue::Strong(false),
            (DriveValue::Weak(false), DriveValue::Weak(true)) => DriveValue::Strong(true),
            (DriveValue::Weak(false), DriveValue::Weak(false)) => DriveValue::Strong(false),
            (DriveValue::Weak(false), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Error) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::HighImpedance, DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Error, DriveValue::Error) => DriveValue::HighImpedance,
        };

        let get_expected_2 = |gate: &DriveValue, source: &DriveValue| match (gate, source) {
            (DriveValue::Strong(true), DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Strong(true), DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Strong(true)) => DriveValue::Strong(true),
            (DriveValue::Strong(false), DriveValue::Strong(false)) => DriveValue::Strong(false),
            (DriveValue::Strong(false), DriveValue::Weak(true)) => DriveValue::Strong(true),
            (DriveValue::Strong(false), DriveValue::Weak(false)) => DriveValue::Strong(false),
            (DriveValue::Strong(false), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Strong(false), DriveValue::Error) => DriveValue::Error,
            (DriveValue::Weak(true), DriveValue::Strong(true)) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Strong(false)) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Weak(true)) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Weak(false)) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Weak(true), DriveValue::Error) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Strong(true)) => DriveValue::Strong(true),
            (DriveValue::Weak(false), DriveValue::Strong(false)) => DriveValue::Strong(false),
            (DriveValue::Weak(false), DriveValue::Weak(true)) => DriveValue::Strong(true),
            (DriveValue::Weak(false), DriveValue::Weak(false)) => DriveValue::Strong(false),
            (DriveValue::Weak(false), DriveValue::HighImpedance) => DriveValue::HighImpedance,
            (DriveValue::Weak(false), DriveValue::Error) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Strong(true)) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Strong(false)) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Weak(true)) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Weak(false)) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::HighImpedance) => DriveValue::Error,
            (DriveValue::HighImpedance, DriveValue::Error) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Strong(true)) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Strong(false)) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Weak(true)) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Weak(false)) => DriveValue::Error,
            (DriveValue::Error, DriveValue::HighImpedance) => DriveValue::Error,
            (DriveValue::Error, DriveValue::Error) => DriveValue::Error,
        };

        for gate in DRIVE_VALUES.iter() {
            for source in DRIVE_VALUES.iter() {
                let mut pmos = Transistor::new_pmos();

                let actual_1 = tick_transistor(&mut pmos, gate, source);
                let expected_1 = get_expected_1(gate, source);
                assert_eq!(actual_1, expected_1);

                let actual_2 = tick_transistor(&mut pmos, gate, source);
                let expected_2 = get_expected_2(gate, source);
                assert_eq!(actual_2, expected_2);

                assert!(!pmos.get_activation());
            }
        }
    }

    fn tick_transistor(
        transistor: &mut Transistor,
        gate: &DriveValue,
        source: &DriveValue,
    ) -> DriveValue {
        transistor.get_gate().borrow_mut().set_drive(*gate);
        transistor.get_source().borrow_mut().set_drive(*source);

        // Perform the ticks necessary to compute the output of the transistor.
        transistor.get_gate().borrow_mut().tick();
        transistor.get_source().borrow_mut().tick();
        transistor.tick();
        transistor.get_drain().borrow_mut().tick();

        transistor.get_drain().borrow().get_drive()
    }
}
