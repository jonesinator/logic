use crate::LogicValue;

/// The simulated electrical states a `Pin` can "drive" onto a `Wire`. Unlike `LogicValue` the
/// `DriveValue` differentiates between `Strong` and `Weak` drives.
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DriveValue {
    /// The `Pin` is strongly driving high/true/1 or low/false/0.
    Strong(bool),

    /// The `Pin` is weakly driving high/true/1 or low/false/0, i.e. through a pull-up or pull-down
    /// resistor.
    Weak(bool),

    /// The `Pin` is not being driven.
    HighImpedance,

    /// The `Pin` is driving an invalid state.
    Error,
}

impl From<LogicValue> for DriveValue {
    /// Converts a `LogicValue` to a `DriveValue`. All `LogicValue`s are `Strong`ly driven.
    fn from(logic_value: LogicValue) -> Self {
        match logic_value {
            LogicValue::Driven(value) => DriveValue::Strong(value),
            LogicValue::HighImpedance => DriveValue::HighImpedance,
            LogicValue::Error => DriveValue::Error,
        }
    }
}

/// A list of all possible `DriveValue`s. Used primarily in tests.
pub const DRIVE_VALUES: &[DriveValue] = &[
    DriveValue::Strong(true),
    DriveValue::Strong(false),
    DriveValue::Weak(true),
    DriveValue::Weak(false),
    DriveValue::HighImpedance,
    DriveValue::Error,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drive_value_from_logic_value() {
        assert_eq!(
            DriveValue::Strong(true),
            DriveValue::from(LogicValue::Driven(true))
        );
        assert_eq!(
            DriveValue::Strong(false),
            DriveValue::from(LogicValue::Driven(false))
        );
        assert_eq!(
            DriveValue::HighImpedance,
            DriveValue::from(LogicValue::HighImpedance)
        );
        assert_eq!(DriveValue::Error, DriveValue::from(LogicValue::Error));
    }
}
