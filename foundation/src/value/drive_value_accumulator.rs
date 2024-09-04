use crate::{DriveValue, LogicValue};

/// Tracks multiple `Pin`s on a `Wire`, taking into account all `DriveValue`s the `Pin`s drive onto
/// the `Wire`, and determining the ultimate resulting `DriveValue` of the `Wire`.
///
/// The rules for determining the `LogicValue` of a `Wire` from the `DriveValue`s of the `Pin`s on
/// the `Wire` are:
///
///   1. If any `Pin` is `DriveValue::Error`, then the `Wire` is `LogicValue::Error`. That is to
///      say, errors propagate.
///   2. If there are both `DriveValue::Strong(true)` and `DriveValue::Strong(false)`, i.e. the wire
///      is connected strongly both high and low (shorted), then the `Wire` is `LogicValue::Error`.
///   3. If there are both `DriveValue::Weak(true)` and `DriveValue::Weak(false)`, i.e. the wire is
///      connected weakly both high and low (shorted), then the `Wire` is `LogicValue::Error`.
///   4. `Strong` `DriveValue`s take precedence over `Weak` `DriveValue`s. The previous rules
///      filtered out cases where both `Strong` `true` and `false` are set or both `Weak` `true` and
///      `false` are set, so if a `Strong` value is set then that will be the value of the `Wire`.
///      If no `Strong` value is set, but a `Weak` value is set, then that will be the value of the
///      `Wire`. If neither a `Strong` nor a `Weak` value is set, then the next rule will apply.
///   5. If all pins are `DriveValue::HighImpedance` then the `Wire` is `LogicValue::HighImpedance`.
///
/// This structure is used as an alternative to iterating through all of the `Pin`s on the `Wire`
/// whenever a `Pin` state changes to determine the `Wire`'s `LogicState`. That would work, but is
/// generally more computationally expensive than just keeping track of the counts like we do here.
/// It's also useful because it separates the conceptual logic from the memory management of the
/// `Pin`s and `Wire`s.
pub(crate) struct DriveValueAccumulator {
    /// The number of pins in the `DriveValue::Strong(true)` state on the `Wire`.
    strong_true: usize,

    /// The number of pins in the `DriveValue::Strong(false)` state on the `Wire`.
    strong_false: usize,

    /// The number of pins in the `DriveValue::Weak(true)` state on the `Wire`.
    weak_true: usize,

    /// The number of pins in the `DriveValue::Weak(false)` state on the `Wire`.
    weak_false: usize,

    /// The number of pins in the `DriveValue::Error` state on the `Wire`.
    error: usize,
}

impl DriveValueAccumulator {
    /// Creates a new, initially empty `DriveValueAccumulator`.
    pub fn new() -> Self {
        Self {
            strong_true: 0,
            strong_false: 0,
            weak_true: 0,
            weak_false: 0,
            error: 0,
        }
    }

    /// Adds all of the counts in another `DriveValueAccumulator` to this one, essentially merging
    /// the counts. Used when connecting two `Wire`s.
    pub fn add(&mut self, other: &Self) -> LogicValue {
        self.strong_true = self.strong_true.strict_add(other.strong_true);
        self.strong_false = self.strong_false.strict_add(other.strong_false);
        self.weak_true = self.weak_true.strict_add(other.weak_true);
        self.weak_false = self.weak_false.strict_add(other.weak_false);
        self.error = self.error.strict_add(other.error);
        self.get_value()
    }

    /// Changes a `Pin`'s drive from `before` to `after`. All pins are initially assumed to be
    /// `DriveValue::HighImpedance`.
    pub fn update(&mut self, before: DriveValue, after: DriveValue) -> LogicValue {
        match before {
            DriveValue::Strong(true) => self.strong_true = self.strong_true.strict_sub(1),
            DriveValue::Strong(false) => self.strong_false = self.strong_false.strict_sub(1),
            DriveValue::Weak(true) => self.weak_true = self.weak_true.strict_sub(1),
            DriveValue::Weak(false) => self.weak_false = self.weak_false.strict_sub(1),
            DriveValue::Error => self.error = self.error.strict_sub(1),
            DriveValue::HighImpedance => (),
        }

        match after {
            DriveValue::Strong(true) => self.strong_true = self.strong_true.strict_add(1),
            DriveValue::Strong(false) => self.strong_false = self.strong_false.strict_add(1),
            DriveValue::Weak(true) => self.weak_true = self.weak_true.strict_add(1),
            DriveValue::Weak(false) => self.weak_false = self.weak_false.strict_add(1),
            DriveValue::Error => self.error = self.error.strict_add(1),
            DriveValue::HighImpedance => (),
        }

        self.get_value()
    }

    /// Uses the counts of all of the `DriveValue`s on the `Wire` to determine the final
    /// `LogicValue` of the `Wire`.
    fn get_value(&self) -> LogicValue {
        let strong_short = self.strong_true != 0 && self.strong_false != 0;
        let weak_short = self.weak_true != 0 && self.weak_false != 0;
        if self.error != 0 || strong_short || weak_short {
            LogicValue::Error
        } else if self.strong_true != 0 {
            LogicValue::Driven(true)
        } else if self.strong_false != 0 {
            LogicValue::Driven(false)
        } else if self.weak_true != 0 {
            LogicValue::Driven(true)
        } else if self.weak_false != 0 {
            LogicValue::Driven(false)
        } else {
            LogicValue::HighImpedance
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_value_accumulator_update() {
        let mut accumulator = DriveValueAccumulator::new();
        assert_eq!(
            accumulator.update(DriveValue::HighImpedance, DriveValue::HighImpedance),
            LogicValue::HighImpedance
        );
        assert_eq!(
            accumulator.update(DriveValue::HighImpedance, DriveValue::Strong(true)),
            LogicValue::Driven(true)
        );
        assert_eq!(
            accumulator.update(DriveValue::Strong(true), DriveValue::Strong(false)),
            LogicValue::Driven(false)
        );
        assert_eq!(
            accumulator.update(DriveValue::Strong(false), DriveValue::Weak(true)),
            LogicValue::Driven(true)
        );
        assert_eq!(
            accumulator.update(DriveValue::Weak(true), DriveValue::Weak(false)),
            LogicValue::Driven(false)
        );
        assert_eq!(
            accumulator.update(DriveValue::Weak(false), DriveValue::Error),
            LogicValue::Error
        );
        assert_eq!(
            accumulator.update(DriveValue::Error, DriveValue::HighImpedance),
            LogicValue::HighImpedance
        );
    }

    #[test]
    fn test_drive_value_accumulator_add() {
        let mut accumulator_1 = DriveValueAccumulator::new();
        let mut accumulator_2 = DriveValueAccumulator::new();
        accumulator_1.update(DriveValue::HighImpedance, DriveValue::Weak(true));
        accumulator_2.update(DriveValue::HighImpedance, DriveValue::Strong(false));
        assert_eq!(accumulator_1.add(&accumulator_2), LogicValue::Driven(false));
    }

    #[test]
    fn test_drive_value_accumulator_strong_short() {
        let mut accumulator = DriveValueAccumulator::new();
        accumulator.update(DriveValue::HighImpedance, DriveValue::Strong(true));
        assert_eq!(
            accumulator.update(DriveValue::HighImpedance, DriveValue::Strong(false)),
            LogicValue::Error
        );
    }

    #[test]
    fn test_drive_value_accumulator_weak_short() {
        let mut accumulator = DriveValueAccumulator::new();
        accumulator.update(DriveValue::HighImpedance, DriveValue::Weak(true));
        assert_eq!(
            accumulator.update(DriveValue::HighImpedance, DriveValue::Weak(false)),
            LogicValue::Error
        );
    }

    #[test]
    #[should_panic]
    fn test_illegal_use() {
        let mut accumulator = DriveValueAccumulator::new();
        accumulator.update(DriveValue::Error, DriveValue::HighImpedance);
    }
}
