//! This crate provides a derive macro for generating `TryFrom<KeyMap>` and `TryFrom<Vec<KeyMap>>` implementations for enums.
//!
//! The `KeyMap` derive macro automatically implements the `TryFrom<KeyMap>` trait for enums,
//! allowing you to easily convert a `KeyMap` to an enum variant based on the specified key bindings.
use keymap_parser::{parse_seq, Node};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Comma, DataEnum, DeriveInput, Ident, LitStr, Token, Variant,
};

/// An attribute path name #[key(...)]
const KEY_IDENT: &str = "key";

const DOC_IDENT: &str = "doc";

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

    // let config_impl = impl_keymap_config(&ast.ident, &variants);
    let items = parse_items(&variants);
    let config = impl_keymap_config(&ast.ident, &items);
    let try_from = impl_try_from_keymap(&ast.ident, &items);

    quote! {
        #try_from
        #config
    }
    .into()
}

struct Item<'a> {
    variant: &'a Variant,
    keys: Result<Vec<String>, syn::Error>,
    description: String,
}

fn parse_items(variants: &Punctuated<Variant, Comma>) -> Vec<Item> {
    variants
        .iter()
        .map(|variant| Item {
            variant,
            description: parse_doc(variant),
            keys: parse_keys(variant),
        })
        .collect::<Vec<_>>()
}

fn impl_keymap_config(name: &Ident, items: &Vec<Item>) -> proc_macro2::TokenStream {
    let mut map_entries = Vec::new();

    for item in items {
        let Ok(keys) = &item.keys else {
            return item.keys.clone().unwrap_err().to_compile_error();
        };

        let variant_ident = &item.variant.ident;
        let doc = &item.description;

        map_entries.push(quote! {
            (
                #name::#variant_ident,
                (
                    vec![#(#keys),*].iter().map(|s| s.to_string()).collect::<Vec<String>>(),
                    #doc.to_string()
                )
            ),
        });
    }

    quote! {
        impl #name {
            pub fn keymap_config() -> std::collections::HashMap<#name, (Vec<String>, String)> {
                std::collections::HashMap::from([
                    #(#map_entries)*
                ])
            }
        }
    }
}

/// Implements [`TryFrom<KeyMap>`] for enums.
fn impl_try_from_keymap(name: &Ident, items: &Vec<Item>) -> proc_macro2::TokenStream {
    let mut match_arms = vec![];

    for item in items {
        let Ok(keys) = &item.keys else {
            return item.keys.clone().unwrap_err().to_compile_error();
        };

        let variant_ident = &item.variant.ident;

        // Split string into a list of keys e.g.
        //
        // "a b" => [quote! { a }, quote! { b }]
        let seq = keys
            .iter()
            .map(|key| key.split(' '))
            .map(|seq| quote! { [#(#seq),*] });

        // Build sequence of keys e.g.
        //
        // [quote! { a }, quote! { b }] => ["a", "b"]
        match_arms.push(quote! {
            #(#seq)|* => Ok(#name::#variant_ident),
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
    // match build_match_arms(name, variants) {
    //     Ok(match_arms) => quote! {
    //         use keymap::KeyMap;
    //
    //         impl TryFrom<KeyMap> for #name {
    //             type Error = String;
    //
    //             /// Convert a [`KeyMap`] into an enum.
    //             fn try_from(value: keymap::KeyMap) -> Result<Self, Self::Error> {
    //                 #name::try_from(vec![value])
    //             }
    //         }
    //
    //         impl TryFrom<Vec<KeyMap>> for #name {
    //             type Error = String;
    //
    //             /// Convert a [`Vec<KeyMap>`] into an enum.
    //             fn try_from(value: Vec<keymap::KeyMap>) -> Result<Self, Self::Error> {
    //                 let keys = value.iter().map(ToString::to_string).collect::<Vec<_>>();
    //
    //                 match keys.iter().map(|v| v.as_str()).collect::<Vec<_>>().as_slice() {
    //                     #(#match_arms)*
    //                     _ => {
    //                         Err(format!("Unknown key [{}]", keys.join(", ")))
    //                     }
    //                 }
    //             }
    //         }
    //     },
    //     Err(err) => err.to_compile_error(),
    // }
}

/// Builds match arms from key attributes.
///
/// # Example
///
/// ```ignore
/// #[derive(KeyMap)]
/// enum Action {
///     #[key("c")]
///     Create,
///     #[key("x", "d")]
///     Delete,
/// }
/// ```
///
/// The above code will generate the following match arms:
///
/// ```ignore
/// ["c"] => Ok(Action::Create),
/// ["x"] | ["d"] => Ok(Action::Delete),
/// ```
fn build_match_arms(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut arms = Vec::new();

    for variant in variants {
        let variant_ident = &variant.ident;
        for attr in &variant.attrs {
            if !attr.path().is_ident(KEY_IDENT) {
                continue;
            }

            // Parse #[key("a", "g g")] directly into list of LitStr
            //          |  |________|
            //        path   (args)
            //
            // e.g. [["a"], ["g", "g"]]
            match attr.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_separated_nonempty) {
                Ok(lit_strs) => {
                    let keys = lit_strs
                        .iter()
                        .map(|str| {
                            let val = str.value();

                            // Parse key sequence, and return early on error
                            let parsed = parse_seq(&val).map_err(|err| {
                                syn::Error::new(
                                    str.span(),
                                    format!("Invalid key: \"{val}\". {err}"),
                                )
                            })?;

                            // Split string into a list of keys e.g.
                            //
                            // "a b" => [quote! { a }, quote! { b }]
                            let seq = parsed
                                .iter()
                                .map(ToString::to_string)
                                .map(|key| quote! { #key })
                                .collect::<Vec<_>>();

                            // Build sequence of keys e.g.
                            //
                            // [quote! { a }, quote! { b }] => ["a", "b"]
                            Ok(quote! { [#(#seq),*] })
                        })
                        .collect::<Result<Vec<_>, syn::Error>>()?;

                    // Combine keys into a single match arm e.g.
                    //
                    // ["a"] | ["g", "g"] => Ok(Action::Delete),
                    arms.push(quote! {
                        #(#keys)|* => Ok(#name::#variant_ident),
                    });
                }
                Err(err) => {
                    arms.push(err.to_compile_error());
                }
            }
        }
    }

    Ok(arms)
}

fn parse_doc(variant: &Variant) -> String {
    let mut doc = String::new();

    for attr in &variant.attrs {
        if !attr.path().is_ident(DOC_IDENT) {
            continue;
        }

        match &attr.meta {
            syn::Meta::NameValue(meta_name_value) => {
                if let syn::Expr::Lit(expr_lit) = &meta_name_value.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let doc_line = lit_str.value().trim().to_string();
                        if !doc.is_empty() {
                            doc.push('\n');
                        }
                        doc.push_str(&doc_line);
                    }
                }
            }
            // Not a NameValue, or other meta type, ignore for doc comments
            _ => {}
        }
    }

    doc
}

fn parse_keys(variant: &Variant) -> Result<Vec<String>, syn::Error> {
    let mut keys = Vec::new();

    for attr in &variant.attrs {
        if !attr.path().is_ident(KEY_IDENT) {
            continue;
        }

        // Parse #[key("a", "g g")] directly into list of LitStr
        //          |  |________|
        //        path   (args)
        //
        // e.g. [["a"], ["g", "g"]]
        match attr.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_separated_nonempty) {
            Ok(lit_strs) => {
                for str in lit_strs {
                    // Directly use the string value, quote! will handle making it a literal
                    let val = str.value();

                    // Parse key sequence, and return early on error
                    let _ = parse_seq(&val).map_err(|err| {
                        syn::Error::new(str.span(), format!("Invalid key: \"{val}\". {err}"))
                    })?;

                    keys.push(str.value());

                    // Split string into a list of keys e.g.
                    //
                    // "a b" => [quote! { a }, quote! { b }]
                    // let seq = parsed
                    //     .iter()
                    //     .map(ToString::to_string)
                    //     .map(|key| quote! { #key })
                    //     .collect::<Vec<_>>();

                    // Build sequence of keys e.g.
                    //
                    // [quote! { a }, quote! { b }] => ["a", "b"]
                    // keys.push(quote! { [#(#seq),*] })
                }
            }
            Err(err) => {
                // Handle error or ignore if not critical for this specific function
                return Err(err);
            }
        }
    }

    Ok(keys)
}
