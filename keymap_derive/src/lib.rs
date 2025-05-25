//! This crate provides a derive macro for generating `TryFrom<KeyMap>` and `TryFrom<Vec<KeyMap>>` implementations for enums.
//!
//! The `KeyMap` derive macro automatically implements the `TryFrom<KeyMap>` trait for enums,
//! allowing you to easily convert a `KeyMap` to an enum variant based on the specified key bindings.
use item::{parse_items, Item};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Ident};

mod item;

/// A derive macro that generates [`TryFrom<KeyMap>`] implementations for enums.
///
/// # Example
///
/// ```ignore
/// use keymap::parse;
/// use keymap_derive::KeyMap;
///
/// #[derive(PartialEq, KeyMap)]
/// enum Action {
///     #[key("c")]
///     Create,
///     #[key("x", "d")]
///     Delete,
/// }
///
/// let keymap = keymap::parse("c").unwrap();
/// let action = Action::try_from(keymap).unwrap();
///
/// assert_eq!(action, Action::Create);
/// ```
///
/// # Attributes
///
/// The `keymap_derive` crate supports the following attributes:
///
/// - `#[key("key")]`: Specifies a key to match.
///
#[proc_macro_derive(KeyMap, attributes(key))]
pub fn keymap(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let syn::Data::Enum(DataEnum { variants, .. }) = ast.data else {
        return syn::Error::new_spanned(
            ast.ident,
            "#[derive(KeyMap)] can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    match parse_items(&variants) {
        Ok(items) => {
            let config = impl_keymap_config(&ast.ident, &items);
            let try_from = impl_try_from(&ast.ident, &items);

            quote! {
                #try_from
                #config
            }
            .into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_keymap_config(name: &Ident, items: &Vec<Item>) -> proc_macro2::TokenStream {
    let mut entries = Vec::new();

    for item in items {
        let ident = &item.variant.ident;
        let keys = &item.keys;
        let doc = &item.description;

        entries.push(quote! {
            (
                #name::#ident,
                keymap::Item::new(
                    vec![#(#keys),*].iter().map(ToString::to_string).collect::<Vec<String>>(),
                    #doc.to_string()
                )
            ),
        });
    }

    quote! {
        impl #name {
            pub fn keymap_config() -> Vec<(#name, keymap::Item)> {
                vec![#(#entries)*]
            }
        }
    }
}

/// Implements [`TryFrom<KeyMap>`] for enums.
fn impl_try_from(name: &Ident, items: &Vec<Item>) -> proc_macro2::TokenStream {
    let mut match_arms = vec![];

    // Builds match arms from key attributes.
    //
    // # Example
    //
    // ```ignore
    // #[derive(KeyMap)]
    // enum Action {
    //     #[key("c")]
    //     Create,
    //     #[key("x", "d d")]
    //     Delete,
    // }
    // ```
    //
    // The above code will generate the following match arms:
    //
    // ```ignore
    // ["c"] => Ok(Action::Create),
    // ["x"] | ["d", "d"] => Ok(Action::Delete),
    // ```
    for item in items {
        let ident = &item.variant.ident;

        // Split string into a list of keys e.g.
        //
        // "d d" => ["d", "d"]
        let keys = item
            .keys
            .iter()
            .map(|key| key.split(' '))
            .map(|seq| quote! { [#(#seq),*] });

        // Build match arms e.g.
        //
        // ["x"] | ["d", "d"] => Action::Delete
        match_arms.push(quote! {
            #(#keys)|* => Ok(#name::#ident),
        });
    }

    quote! {
        use keymap::KeyMap;

        impl TryFrom<KeyMap> for #name {
            type Error = String;

            /// Convert a [`KeyMap`] into an enum.
            fn try_from(value: keymap::KeyMap) -> Result<Self, Self::Error> {
                #name::try_from(vec![value])
            }
        }

        impl TryFrom<Vec<KeyMap>> for #name {
            type Error = String;

            /// Convert a [`Vec<KeyMap>`] into an enum.
            fn try_from(value: Vec<keymap::KeyMap>) -> Result<Self, Self::Error> {
                let keys = value.iter().map(ToString::to_string).collect::<Vec<_>>();

                match keys.iter().map(|v| v.as_str()).collect::<Vec<_>>().as_slice() {
                    #(#match_arms)*
                    _ => {
                        Err(format!("Unknown key [{}]", keys.join(", ")))
                    }
                }
            }
        }

    }
}
