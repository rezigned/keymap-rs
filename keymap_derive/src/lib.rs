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

        // Find the index of a key group (like @any, @digit, etc.) in the parsed nodes.
        // For simplicity, we only check the first key mapped to this variant.
        // If the first key contains a group, the matched node at that index will be a `Char`.
        let mut char_idx: Option<usize> = None;
        if let Some(first_node_seq) = item.nodes.first() {
            for (idx, node) in first_node_seq.iter().enumerate() {
                if let ::keymap_parser::node::Key::Group(_) = node.key {
                    char_idx = Some(idx);
                }
            }
        }

        let extract_char = if let Some(idx) = char_idx {
            quote! {
                match keys.get(#idx).map(|n| &n.key) {
                    Some(::keymap_parser::node::Key::Char(c)) => *c,
                    _ => Default::default(),
                }
            }
        } else {
            quote! { Default::default() }
        };

        let variant_expr = match &item.variant.fields {
            Fields::Unit => quote! { #name::#ident },
            Fields::Unnamed(fields) => {
                let defaults = fields.unnamed.iter().map(|f| {
                    let ty_str = quote!(#f).to_string();
                    if ty_str == "char" {
                        extract_char.clone()
                    } else {
                        quote! { Default::default() }
                    }
                });
                quote! { #name::#ident(#(#defaults),*) }
            }
            Fields::Named(fields) => {
                let defaults = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    let ty_str = quote!(#f).to_string();
                    if ty_str.contains("char") {
                        quote! { #name: #extract_char }
                    } else {
                        quote! { #name: Default::default() }
                    }
                });
                quote! { #name::#ident { #(#defaults),* } }
            }
        };

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

        let variant_pat = match &item.variant.fields {
            Fields::Unit => quote! { #name::#ident },
            Fields::Unnamed(_) => quote! { #name::#ident(..) },
            Fields::Named(_) => quote! { #name::#ident { .. } },
        };

        let variant_name_str = ident.to_string();

        match_arms_serialize.push(quote! {
            #variant_pat => #variant_name_str,
        });

        match_arms_deserialize.push(quote! {
            #variant_name_str => Ok(#variant_expr_default),
        });

        match_arms_bind.push(quote! {
            #variant_pat => #variant_expr,
        });

        // keymap_item
        match_arms.push(quote! {
            #variant_pat => ::keymap::Item::new(
                vec![#(#keys),*],
                #doc.to_string()
            ),
        });

        // keymap_config
        if !item.ignore {
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
                }
            }

            fn bind(&self, keys: &[::keymap::KeyMap]) -> Self
            where
                Self: Clone,
            {
                match self {
                    #(#match_arms_bind)*
                }
            }
        }

        #serde_impls
    }
}
