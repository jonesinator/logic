use crate::{AnyDevice, DeviceContainer, Pin, Transistor};
use std::{any::Any, cell::Ref, ops::Deref};

/// Prints a detailed recursive representation of a device to the console. Generates valid YAML in a
/// dumb way.
pub fn print(device: &dyn AnyDevice, level: usize, is_array_member: bool) {
    let name = device.type_name();
    if is_array_member {
        println!("{:indent$}- type: {}", "", name, indent = level - 2);
    } else {
        println!("{:indent$}type: {}", "", device.type_name(), indent = level);
    }

    if !device.pins().is_empty() {
        println!("{:indent$}pins:", "", indent = level);
    }
    for (name, pins) in device.pins().iter() {
        let show_pin = |pin: &Ref<Pin>| {
            println!(
                "{:indent$}drive: {:?}",
                "",
                pin.get_drive(),
                indent = level + 4,
            );
            println!("{:indent$}read: {:?}", "", pin.read(), indent = level + 4);
            println!("{:indent$}id: {:p}", "", pin.deref(), indent = level + 4);
            println!("{:indent$}connected:", "", indent = level + 4);
            for connected_pin in pin.get_connected_pins().iter() {
                println!(
                    "{:indent$}- {:p}",
                    "",
                    connected_pin.borrow().deref(),
                    indent = level + 6
                );
            }
        };

        println!("{:indent$}{}:", "", name, indent = level + 2);
        match pins {
            DeviceContainer::Single(pin) => show_pin(pin),
            DeviceContainer::Multiple(pins) => pins.iter().for_each(|pin| show_pin(pin)),
        }
    }

    if !device.children().is_empty() {
        println!("{:indent$}children:", "", indent = level);
    }
    for (name, children) in device.children().iter() {
        println!("{:indent$}{}:", "", name, indent = level + 2);
        match children {
            DeviceContainer::Single(child) => {
                print(*child, level + 4, false);
            }
            DeviceContainer::Multiple(children) => {
                for child in children.iter() {
                    print(*child, level + 6, true);
                }
            }
        }
    }
}

/// Moves the simulated `Transistors` and their associated `Pin`s and `Wire`s forward in time until
/// there is a time step where nothing changes.
///
/// Returns the number of ticks it took to achieve being settled.
pub fn settle(device: &mut dyn AnyDevice) -> usize {
    let mut ticks: usize = 0;
    while tick(device) {
        ticks += 1;
    }
    ticks
}

/// Moves all simulated `Transistors` and their associated `Pin`s and `Wire`s forward one time
/// step.
///
/// A tick is split into two phases. First, we tick all of the `Transistors`, which use all of the
/// _current_ `Pin` and `Wire` states to perform their updates. The `Transistors` will set the next
/// state on the `Pin`s, but that won't take effect until the `Pin` is ticked.
///
/// Returns `true` if anything changed during the tick. Returns `false` otherwise.
pub fn tick(device: &mut dyn AnyDevice) -> bool {
    let mut changed = false;
    changed |= tick_transistors(device);
    changed |= tick_pins(device);
    changed
}

/// Recursively goes through the `Device` hierarchy and calls `tick` on all `Transistor`
/// `Pin`s.
fn tick_pins(device: &mut dyn AnyDevice) -> bool {
    let mut changed = false;

    if let Some(transistor) = (device as &mut dyn Any).downcast_mut::<Transistor>() {
        changed |= transistor.get_drain().borrow_mut().tick();
        changed |= transistor.get_gate().borrow_mut().tick();
        changed |= transistor.get_source().borrow_mut().tick();
    }

    for (_, children) in device.children_mut().iter_mut() {
        match children {
            DeviceContainer::Single(child) => changed |= tick_pins(*child),
            DeviceContainer::Multiple(children) => children
                .iter_mut()
                .for_each(|child| changed |= tick_pins(*child)),
        }
    }

    changed
}

/// Recursively goes through the `Device` hierarchy and calls `tick` on all `Transistor`s.
fn tick_transistors(device: &mut dyn AnyDevice) -> bool {
    let mut changed = false;

    if let Some(transistor) = (device as &mut dyn Any).downcast_mut::<Transistor>() {
        changed |= transistor.tick();
    }

    for (_, children) in device.children_mut().iter_mut() {
        match children {
            DeviceContainer::Single(child) => changed |= tick_transistors(*child),
            DeviceContainer::Multiple(children) => children
                .iter_mut()
                .for_each(|child| changed |= tick_transistors(*child)),
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AnyDevice, Constant, Device, DeviceContainer, DriveValue, LogicValue, Pin, Transistor,
    };
    use device_derive::Device;
    use std::cell::RefCell;
    use std::rc::Rc;

    // Used to show an empty device (i.e. a device with no pins or children) can nevertheless be
    // simulated.
    #[derive(Device)]
    struct EmptyDevice {}

    // A simple device that contains fields for pins and children with both single and multiple
    // entries. This allows us to exercise most of the `device_derive::Device` procedural macro.
    //
    // Operationally this creates a not gate, but instead of tying the inputs together, the nmos
    // and pmos inputs are separate (in order to test a vector of pins).
    #[derive(Device)]
    struct SimpleDevice {
        #[child]
        strong_true: Constant,

        #[child]
        strong_false: Constant,

        #[children]
        nmos: Vec<Transistor>,

        #[child]
        pmos: Transistor,

        #[pins]
        input: Vec<Rc<RefCell<Pin>>>,

        #[pin]
        output: Rc<RefCell<Pin>>,
    }

    impl SimpleDevice {
        fn new() -> Self {
            let strong_true = Constant::new_strong(true);
            let strong_false = Constant::new_strong(false);
            let nmos = vec![Transistor::new_nmos()];
            let pmos = Transistor::new_pmos();
            let input = vec![nmos[0].get_gate().clone(), pmos.get_gate().clone()];
            let output = pmos.get_drain().clone();

            Pin::connect(strong_false.get_output(), nmos[0].get_source());
            Pin::connect(strong_true.get_output(), pmos.get_source());
            Pin::connect(nmos[0].get_drain(), pmos.get_drain());

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

    #[test]
    fn empty_device() {
        let mut empty_device = EmptyDevice {};
        tick(&mut empty_device);
        tick(&mut empty_device);
        print(&empty_device, 2, true);
    }

    #[test]
    fn simple_device() {
        let mut simple_device = SimpleDevice::new();
        simple_device.get_input()[0]
            .borrow_mut()
            .set_drive(DriveValue::Strong(true));
        simple_device.get_input()[1]
            .borrow_mut()
            .set_drive(DriveValue::Strong(true));
        assert_eq!(settle(&mut simple_device), 2);
        assert_eq!(
            LogicValue::Driven(false),
            simple_device.get_output().borrow().read()
        );
        print(&simple_device, 0, false);
    }
}
