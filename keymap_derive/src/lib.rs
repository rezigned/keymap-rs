//! This crate provides a derive macro for generating `TryFrom<KeyMap>` and `TryFrom<Vec<KeyMap>>` implementations for enums.
//!
//! The `KeyMap` derive macro automatically implements the `TryFrom<KeyMap>` trait for enums,
//! allowing you to easily convert a `KeyMap` to an enum variant based on the specified key bindings.
use item::{parse_items, Item};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, Ident};

mod item;

/// A derive macro that generates [`TryFrom<KeyMap>`] implementations for enums.
///
/// # Example
///
/// ```ignore
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
/// let config = Action::keymap_config();
/// let action = config.get_by_keymap(&keymap).unwrap();
///
/// assert_eq!(action, &Action::Create);
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

            quote! {
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

        let variant = match &item.variant.fields {
            Fields::Unit => quote! { #name::#ident },
            Fields::Unnamed(_) => quote! { #name::#ident(..) },
            Fields::Named(_) => quote! { #name::#ident { .. } },
        };

        // keymap_item
        match_arms.push(quote! {
            #variant => ::keymap::Item::new(
                vec![#(#keys),*],
                #doc.to_string()
            ),
        });

        // keymap_config
        if !item.ignore {
            entries.push(quote! {
                (
                    #variant,
                    ::keymap::Item::new(
                        vec![#(#keys),*],
                        #doc.to_string()
                    )
                ),
            });
        }
    }

    quote! {
        impl ::keymap::KeyMapConfig<#name> for #name {
            fn keymap_config() -> ::keymap::Config<#name> {
                ::keymap::Config::new(vec![#(#entries)*])
            }

            fn keymap_item(&self) -> ::keymap::Item {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}
