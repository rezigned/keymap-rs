//! This crate provides a derive macro for generating `TryFrom<KeyMap>` and `TryFrom<Vec<KeyMap>>` implementations for enums.
//!
//! The `KeyMap` derive macro automatically implements the `TryFrom<KeyMap>` trait for enums,
//! allowing you to easily convert a `KeyMap` to an enum variant based on the specified key bindings.
use item::{parse_items, Item};
use keymap_parser::Key;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Ident};

mod item;

/// A derive macro that generates [`TryFrom<KeyMap>`] implementations for enums.
///
/// # Example
///
/// ```
/// use keymap::KeyMapConfig;
///
/// #[derive(Debug, PartialEq, keymap::KeyMap)]
/// enum Action {
///     /// Create a new item
///     #[key("c")]
///     Create,
///     /// Delete an item
///     #[key("x", "d")]
///     Delete,
/// }
///
/// let keymap = keymap::parse("c").unwrap();
/// let action = Action::try_from(keymap).unwrap();
///
/// assert_eq!(action, Action::Create);
/// assert_eq!(action.keymap_item().description, "Create a new item");
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
    let mut match_arms = Vec::new();

    for item in items {
        let ident = &item.variant.ident;
        let keys = &item
            .keys
            .iter()
            .map(|key| quote! { #key.to_string() })
            .collect::<Vec<_>>();
        let doc = &item.description;

        entries.push(quote! {
            (
                #name::#ident,
                ::keymap::Item::new(
                    vec![#(#keys),*],
                    #doc.to_string()
                )
            ),
        });

        match_arms.push(quote! {
            #name::#ident => ::keymap::Item::new(
                vec![#(#keys),*],
                #doc.to_string()
            ),
        });
    }

    quote! {
        impl ::keymap::KeyMapConfig<#name> for #name {
            fn keymap_config() -> Vec<(#name, ::keymap::Item)> {
                vec![#(#entries)*]
            }

            fn keymap_item(&self) -> ::keymap::Item {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}

/// Implements [`TryFrom<KeyMap>`] for enums.
fn impl_try_from(name: &Ident, items: &Vec<Item>) -> proc_macro2::TokenStream {
    let mut match_arms = vec![];
    let mut char_group_branches = vec![];

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
            .map(|key| key.split_whitespace())
            .map(|seq| quote! { [#(#seq),*] })
            .collect::<Vec<_>>();

        // Build match arms e.g.
        //
        // ["x"] | ["d", "d"] => Action::Delete
        if !keys.is_empty() {
            match_arms.push(quote! {
                #(#keys)|* => ::std::result::Result::Ok(#name::#ident),
            });
        }

        // Build char group expression e.g.
        //
        // #[key("@digit")]
        // Delete,
        //
        // CharGroup::Digit.matches(c)
        let groups = item
            .nodes
            .iter()
            .filter_map(|nodes| {
                let char_groups = nodes
                    .iter()
                    .filter(|node| matches!(node.key, Key::Group(_)))
                    .map(|node| match node.key {
                        Key::Group(group) => {
                            let name = format!("{group:?}");
                            let ident = Ident::new(&name, proc_macro2::Span::call_site());

                            quote! {
                                ::keymap_parser::node::CharGroup::#ident.matches(c)
                            }
                        }
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>();

                if char_groups.is_empty() {
                    None
                } else {
                    Some(quote! { #(#char_groups)&&* })
                }
            })
            .collect::<Vec<_>>();

        // Build if expression
        //
        // if a.matches(c) || b.matches(c) { Action::Delete }
        if !groups.is_empty() {
            char_group_branches.push(quote! {
                if #(#groups)||* { ::std::result::Result::Ok(#name::#ident) } else
            });
        }
    }

    quote! {
        use ::keymap::KeyMap;

        impl TryFrom<KeyMap> for #name {
            type Error = String;

            /// Convert a [`KeyMap`] into an enum.
            fn try_from(value: ::keymap::KeyMap) -> ::std::result::Result<Self, Self::Error> {
                #name::try_from(vec![value])
            }
        }

        impl TryFrom<Vec<KeyMap>> for #name {
            type Error = String;

            /// Convert a [`Vec<KeyMap>`] into an enum.
            fn try_from(value: Vec<::keymap::KeyMap>) -> ::std::result::Result<Self, Self::Error> {

                // TODO: Check nodes directly (no conversion to string).
                let keys = value.iter().map(ToString::to_string).collect::<Vec<_>>();

                match keys.iter().map(|v| v.as_str()).collect::<Vec<_>>().as_slice() {
                    #(#match_arms)*

                    // Match char group e.g. CharGroup::Digit, CharGroup::Alpha, etc.
                    //
                    // NOTE: It currenlty only supports single char groups.
                    [char] => {
                        let c = char.chars().next().unwrap();
                        #(#char_group_branches)* { ::std::result::Result::Err(format!("Unknown key [{char}]")) }
                    }
                    _ => ::std::result::Result::Err(format!("Unknown key [{}]", keys.join(", ")))
                }
            }
        }

    }
}
