//! A procedural macro for implementing the `foundation::Device` trait.
//!
//! This greatly eases implementing the `foundation::Device` trait by using annotations on struct
//! members to generate the `pins`, `children`, and `children_mut` functions of the trait, and using
//! the struct name as the `String` returned by `type_name`.
//!
//! See the example in the `foundation` crate.

extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields};

/// Implements the `Device` trait for a given `struct`. You must label the struct members with
/// one of the following attributes:
///
/// - `pin` - The member should be `Rc<RefCell<Pin>>`.
/// - `pins` - The member should be `Vec<Rc<RefCell<Pin>>>`.
/// - `child` - The member should implement the `Device` trait.
/// - `children` - The member should be a vector of structs that implement the `Device` trait.
///
/// Each `pin` and `pins` field will also get a `get_field` function implemented for it. This
/// allows all `Device` fields to be private
///
/// Nothing is done to fields without attributes.
#[proc_macro_derive(Device, attributes(pin, pins, child, children))]
pub fn derive_device(input_token_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_token_stream as DeriveInput);
    let struct_identifier = &input.ident;
    if let Data::Struct(DataStruct { fields, .. }) = &input.data {
        let pin_fields = get_fields_with_attribute(fields, "pin");
        let pins_fields = get_fields_with_attribute(fields, "pins");
        let child_fields = get_fields_with_attribute(fields, "child");
        let children_fields = get_fields_with_attribute(fields, "children");

        let pin_getters = make_pin_getters(&pin_fields, &pins_fields);
        let pins_implementation = make_pins_implementation(&pin_fields, &pins_fields);
        let children_implementation = make_children_implementation(&child_fields, &children_fields);

        quote! {
            impl #struct_identifier {
                #pin_getters
            }

            #[automatically_derived]
            impl Device for #struct_identifier {
                #pins_implementation
                #children_implementation

                fn type_name(&self) -> String {
                    stringify!(#struct_identifier).to_string()
                }
            }
        }
    } else {
        quote! {}
    }
    .into()
}

/// Gets a vector of fields that have an attribute with the given name.
fn get_fields_with_attribute<'a>(fields: &'a Fields, ident: &str) -> Vec<&'a Field> {
    fields
        .iter()
        .filter(|field| field.attrs.iter().any(|attr| attr.path().is_ident(ident)))
        .collect()
}

/// Creates the implementation of the `pins` and `pins_mut` functions of the `Device` trait.
fn make_pins_implementation(pin_fields: &[&Field], pins_fields: &[&Field]) -> TokenStream2 {
    if !pin_fields.is_empty() || !pins_fields.is_empty() {
        // Need two copies of each since we need two passes, one for the immutable function
        // and one for the mutable function.
        let pin_names = pin_fields.iter().map(|field| &field.ident);
        let pin_names2 = pin_names.clone();
        let pins_names = pins_fields.iter().map(|field| &field.ident);
        let pins_names2 = pins_names.clone();

        quote! {
            fn pins(
                &self
            ) -> std::collections::HashMap<String, DeviceContainer<std::cell::Ref<Pin>>> {
                std::collections::HashMap::from([
                    #((
                        stringify!(#pin_names).to_string(),
                        DeviceContainer::Single(self.#pin_names.borrow())
                    ),)*
                    #((
                        stringify!(#pins_names).to_string(),
                        DeviceContainer::Multiple(
                            self.#pins_names.iter().map(|pin| pin.borrow()).collect()
                        )
                    ),)*
                ])
            }

            fn pins_mut(
                &mut self
            ) -> std::collections::HashMap<String, DeviceContainer<std::cell::RefMut<Pin>>> {
                std::collections::HashMap::from([
                    #((
                        stringify!(#pin_names2).to_string(),
                        DeviceContainer::Single(self.#pin_names2.borrow_mut())
                    ),)*
                    #((
                        stringify!(#pins_names2).to_string(),
                        DeviceContainer::Multiple(
                            self.#pins_names2.iter().map(|pin| pin.borrow_mut()).collect()
                        )
                    ),)*
                ])
            }
        }
    } else {
        quote! {
            fn pins(
                &self
            ) -> std::collections::HashMap<String, DeviceContainer<std::cell::Ref<Pin>>> {
                std::collections::HashMap::new()
            }

            fn pins_mut(
                &mut self
            ) -> std::collections::HashMap<String, DeviceContainer<std::cell::RefMut<Pin>>> {
                std::collections::HashMap::new()
            }
        }
    }
}

/// Creates the implementations of the `get_${pin}` getter functions of the `Device` trait.
fn make_pin_getters(pin_fields: &[&Field], pins_fields: &[&Field]) -> TokenStream2 {
    let pin_names = pin_fields.iter().map(|field| field.ident.as_ref().unwrap());
    let pin_getter_names = pin_fields
        .iter()
        .map(|field| format_ident!("get_{}", field.ident.as_ref().unwrap()));
    let pins_names = pins_fields.iter().map(|field| &field.ident);
    let pins_getter_names = pins_fields
        .iter()
        .map(|field| format_ident!("get_{}", field.ident.as_ref().unwrap()));

    quote! {
        #(
            /// Get the pin from the device. (automatically generated function)
            pub fn #pin_getter_names(&self) -> &std::rc::Rc<std::cell::RefCell<Pin>> {
                &self.#pin_names
            }
        )*
        #(
            /// Get the pins from the device. (automatically generated function)
            pub fn #pins_getter_names(&self) -> &Vec<std::rc::Rc<std::cell::RefCell<Pin>>> {
                &self.#pins_names
            }
        )*
    }
}

/// Creates the implementation of the `children` and `children_mut` functions of the `Device` trait.
fn make_children_implementation(
    child_fields: &[&Field],
    children_fields: &[&Field],
) -> TokenStream2 {
    if !child_fields.is_empty() || !children_fields.is_empty() {
        // Need two copies of each since we need two passes, one for the immutable function
        // and one for the mutable function.
        let child_names_1 = child_fields.iter().map(|field| &field.ident);
        let child_names_2 = child_names_1.clone();
        let children_names_1 = children_fields.iter().map(|field| &field.ident);
        let children_names_2 = children_names_1.clone();

        quote! {
            fn children(
                &self
            ) -> std::collections::HashMap<String, DeviceContainer<&dyn AnyDevice>> {
                std::collections::HashMap::from([
                    #((
                        stringify!(#child_names_1).to_string(),
                        DeviceContainer::Single(&self.#child_names_1 as &dyn AnyDevice)
                    ),)*
                    #((
                        stringify!(#children_names_1).to_string(),
                        DeviceContainer::Multiple(
                            self.#children_names_1.iter().map(
                                |child| child as &dyn AnyDevice
                            ).collect()
                        )
                    ),)*
                ])
            }

            fn children_mut(
                &mut self
            ) -> std::collections::HashMap<String, DeviceContainer<&mut dyn AnyDevice>> {
                std::collections::HashMap::from([
                    #((
                        stringify!(#child_names_2).to_string(),
                        DeviceContainer::Single(
                            &mut self.#child_names_2 as &mut dyn AnyDevice
                        )
                    ),)*
                    #((
                        stringify!(#children_names_2).to_string(),
                        DeviceContainer::Multiple(
                            self.#children_names_2.iter_mut().map(
                                |child| child as &mut dyn AnyDevice
                            ).collect()
                        )
                    ),)*
                ])
            }
        }
    } else {
        quote! {
            fn children(
                &self
            ) -> std::collections::HashMap<String, DeviceContainer<&dyn AnyDevice>> {
                std::collections::HashMap::new()
            }

            fn children_mut(
                &mut self
            ) -> std::collections::HashMap<String, DeviceContainer<&mut dyn AnyDevice>> {
                std::collections::HashMap::new()
            }
        }
    }
}
