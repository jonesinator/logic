use crate::Pin;
use std::any::Any;
use std::cell::{Ref, RefMut};
use std::collections::HashMap;

/// Helper enum that allows the pins/children hash maps in the `Device` to store single elements
/// in a way distinct from a vector of one entry.
pub enum DeviceContainer<T> {
    /// Stores a single instance of the device type.
    Single(T),

    /// Stores a multiple instances of the device type in a vector.
    Multiple(Vec<T>),
}

/// Represents a generic device (collection of `Pin`s and/or child `Device`s) in a digital logic
/// simulation.
pub trait Device {
    /// Gets a friendly type name of this device. This should be used for debugging only.
    fn type_name(&self) -> String;

    /// Gets all of the `Pin`s this `Device` owns.
    fn pins(&self) -> HashMap<String, DeviceContainer<Ref<Pin>>>;

    /// Gets all of the `Pin`s this `Device` owns, mutably.
    fn pins_mut(&mut self) -> HashMap<String, DeviceContainer<RefMut<Pin>>>;

    /// Gets all of the child `Device`s this `Device` owns.
    fn children(&self) -> HashMap<String, DeviceContainer<&dyn AnyDevice>>;

    /// Gets all of the child `Device`s this `Device` owns, mutably.
    fn children_mut(&mut self) -> HashMap<String, DeviceContainer<&mut dyn AnyDevice>>;
}

/// A composite trait for a `Device` which is also `Any`. This allows the concrete type to be
/// extracted from the `Device` if it is known.
pub trait AnyDevice: Device + Any {}

/// Automatically implement `AnyDevice` for any `T` which conforms to both `Device` and `Any`
/// traits.
impl<T> AnyDevice for T where T: Device + Any {}

// You can derive `Device` from an `enum`...but it won't do anything. This is a rather silly test
// that allows us to get code coverage on `device_derive`. This is especially silly since if we
// panicked on that path that used an `enum`, then trying to use it would just be a compilation
// error. That's clearly a better behavior overall, but gotta have that 100% green coverage...
// Additionally, we create a structure that has a `pins` but no `pin` because there's no example
// of that and that is also required for full branch coverage in `device_derive`.
#[cfg(test)]
mod tests {
    use super::*;
    use device_derive::Device;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, Device, PartialEq)]
    enum TestEnum {
        SingleValue,
    }

    #[derive(Device)]
    struct TestJustPins {
        #[pins]
        p: Vec<Rc<RefCell<Pin>>>,
    }

    #[test]
    fn test_enum_device() {
        assert_eq!(TestEnum::SingleValue, TestEnum::SingleValue);
    }

    #[test]
    fn test_just_pins() {
        let test_device = TestJustPins { p: vec![] };
        assert_eq!(test_device.get_p().len(), 0);
    }
}
