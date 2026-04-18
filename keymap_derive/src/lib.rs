//! This crate provides a derive macro for generating `TryFrom<KeyMap>` and `TryFrom<Vec<KeyMap>>` implementations for enums.
//!
//! The `KeyMap` derive macro automatically implements the `TryFrom<KeyMap>` trait for enums,
//! allowing you to easily convert a `KeyMap` to an enum variant based on the specified key bindings.
use item::{parse_items, Item};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, Ident};

mod item;

/// A derive macro that generates keymap configuration logic from enums.
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
///     /// Captured character matched by any key group macro (like `@any` or `@digit`)!
///     #[key("@any")]
///     Jump(char),
/// }
///
/// let keymap = keymap::parse("a").unwrap();
/// let config = Action::keymap_config();
///
/// // Standard lookup returning a reference to the default `Jump('\0')` variant
/// let action = config.get_by_keymap(&keymap).unwrap();
///
/// // Or use Key Group Capturing to extract the matched character from @any, @digit, etc.!
/// let bound_action = config.get_bound_by_keymap(&keymap).unwrap();
/// assert_eq!(bound_action, Action::Jump('a'));
/// ```
///
/// **Note:** `keymap_derive` automatically generates specialized `serde::Serialize`
/// and `serde::Deserialize` implementations for the target `enum` allowing seamless
/// string-mapped configurations without users needing to configure `#[serde(untagged)]`
/// defaults for enum variants containing payloads.
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
    let mut match_arms_serialize = Vec::new();
    let mut match_arms_deserialize = Vec::new();
    let mut match_arms_bind = Vec::new();

    for item in items {
        let ident = &item.variant.ident;
        let keys = &item
            .keys
            .iter()
            .map(|key| quote! { #key.to_string() })
            .collect::<Vec<_>>();
        let doc = &item.description;

        // `char_idx` is the position of a key group node (e.g. `@any`, `@digit`) within
        // the first key sequence of this variant. It is `None` when no group is present.
        //
        // Example: `#[key("@digit")]` → sequence is `[@digit]` → char_idx = Some(0)
        //          `#[key("d")]`      → sequence is `[d]`      → char_idx = None
        //
        // Only the first key sequence is inspected because all keys for a given variant
        // must share the same group position (they map to the same field type).
        // At runtime, `char_idx` tells `extract_via_trait` which node to pass to
        // `KeyGroupValue::from_keymap_node` when binding the matched character/digit.
        let mut char_idx: Option<usize> = None;
        if let Some(first_node_seq) = item.nodes.first() {
            for (idx, node) in first_node_seq.iter().enumerate() {
                if let keymap_parser::node::Key::Group(_) = node.key {
                    char_idx = Some(idx);
                }
            }
        }

        // Generates an expression for extracting a value at the key group index using the
        // `KeyGroupValue` trait. This works for any type that implements the trait, including
        // type aliases, because the trait bound is resolved at monomorphisation time rather
        // than by inspecting the token string of the type.
        let extract_via_trait = |ty: &syn::Type| -> proc_macro2::TokenStream {
            if let Some(idx) = char_idx {
                quote! {
                    match keys.get(#idx) {
                        Some(node) => <#ty as ::keymap::KeyGroupValue>::from_keymap_node(node),
                        None => Default::default(),
                    }
                }
            } else {
                quote! { Default::default() }
            }
        };

        let variant_expr = match &item.variant.fields {
            Fields::Unit => quote! { #name::#ident },
            Fields::Unnamed(fields) => {
                let defaults = fields.unnamed.iter().map(|f| {
                    if char_idx.is_some() {
                        extract_via_trait(&f.ty)
                    } else {
                        quote! { Default::default() }
                    }
                });
                quote! { #name::#ident(#(#defaults),*) }
            }
            Fields::Named(fields) => {
                let defaults = fields.named.iter().map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    if char_idx.is_some() {
                        let expr = extract_via_trait(&f.ty);
                        quote! { #field_name: #expr }
                    } else {
                        quote! { #field_name: Default::default() }
                    }
                });
                quote! { #name::#ident { #(#defaults),* } }
            }
        };

        let variant_pat = match &item.variant.fields {
            Fields::Unit => quote! { #name::#ident },
            Fields::Unnamed(_) => quote! { #name::#ident(..) },
            Fields::Named(_) => quote! { #name::#ident { .. } },
        };

        let variant_name_str = ident.to_string();

        match_arms_serialize.push(quote! {
            #variant_pat => #variant_name_str,
        });

        if !item.ignore {
            match_arms_bind.push(quote! {
                #variant_pat => #variant_expr,
            });

            let variant_expr_default = match &item.variant.fields {
                Fields::Unit => quote! { #name::#ident },
                Fields::Unnamed(fields) => {
                    let defaults = fields.unnamed.iter().map(|_| quote! { Default::default() });
                    quote! { #name::#ident(#(#defaults),*) }
                }
                Fields::Named(fields) => {
                    let defaults = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote! { #name: Default::default() }
                    });
                    quote! { #name::#ident { #(#defaults),* } }
                }
            };

            match_arms_deserialize.push(quote! {
                #variant_name_str => Ok(#variant_expr_default),
            });
            match_arms.push(quote! {
                #variant_pat => ::keymap::Item::new(
                    vec![#(#keys),*],
                    #doc.to_string()
                ),
            });

            entries.push(quote! {
                (
                    #variant_expr_default,
                    ::keymap::Item::new(
                        vec![#(#keys),*],
                        #doc.to_string()
                    )
                ),
            });
        }
    }

    let serde_impls = quote! {
        impl ::serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                let variant_name = match self {
                    #(#match_arms_serialize)*
                };
                serializer.serialize_str(variant_name)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct EnumVisitor;
                impl<'de> ::serde::de::Visitor<'de> for EnumVisitor {
                    type Value = #name;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        formatter.write_str("a valid variant name for #name")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        match value {
                            #(#match_arms_deserialize)*
                            _ => Err(E::unknown_variant(value, &[])),
                        }
                    }
                }
                deserializer.deserialize_str(EnumVisitor)
            }
        }
    };

    quote! {
        impl ::keymap::KeyMapConfig<#name> for #name {
            fn keymap_config() -> ::keymap::Config<#name> {
                ::keymap::Config::new(vec![#(#entries)*])
            }

            fn keymap_item(&self) -> ::keymap::Item {
                match self {
                    #(#match_arms)*
                    _ => ::core::unreachable!("ignored variant has no keymap"),
                }
            }

            fn bind(&self, keys: &[::keymap::KeyMap]) -> Self
            where
                Self: Clone,
            {
                match self {
                    #(#match_arms_bind)*
                    _ => ::core::unreachable!("ignored variant cannot be bound"),
                }
            }
        }

        #serde_impls
    }
}
