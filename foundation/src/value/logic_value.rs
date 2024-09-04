use crate::DriveValue;

/// The simulated electrical states a `Wire` can resolve to, taking into account all `Pins`
/// connected together on the `Wire`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogicValue {
    /// The `Wire` is being driven high/true/1 or low/false/0.
    Driven(bool),

    /// The `Wire` is not being driven, i.e. disconnected.
    HighImpedance,

    /// The `Wire` is in an invalid state.
    Error,
}

impl From<DriveValue> for LogicValue {
    /// Converts a `DriveValue` to a `LogicValue`. Both `Strong` and `Weak` map to `Driven`.
    fn from(drive_value: DriveValue) -> Self {
        match drive_value {
            DriveValue::Strong(value) | DriveValue::Weak(value) => LogicValue::Driven(value),
            DriveValue::HighImpedance => LogicValue::HighImpedance,
            DriveValue::Error => LogicValue::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_value_from_drive_value() {
        assert_eq!(
            LogicValue::Driven(true),
            LogicValue::from(DriveValue::Strong(true))
        );
        assert_eq!(
            LogicValue::Driven(true),
            LogicValue::from(DriveValue::Weak(true))
        );
        assert_eq!(
            LogicValue::Driven(false),
            LogicValue::from(DriveValue::Strong(false))
        );
        assert_eq!(
            LogicValue::Driven(false),
            LogicValue::from(DriveValue::Weak(false))
        );
        assert_eq!(
            LogicValue::HighImpedance,
            LogicValue::from(DriveValue::HighImpedance)
        );
        assert_eq!(LogicValue::Error, LogicValue::from(DriveValue::Error));
    }
}
