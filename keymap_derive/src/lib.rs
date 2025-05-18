//! This crate provides a derive macro for generating `TryFrom<KeyMap>` and `TryFrom<Vec<KeyMap>>` implementations for enums.
//!
//! The `KeyMap` derive macro automatically implements the `TryFrom<KeyMap>` trait for enums,
//! allowing you to easily convert a `KeyMap` to an enum variant based on the specified key bindings.
use keymap_parser::parse_seq;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, token::Comma, DataEnum, DeriveInput, Ident, LitStr, Token, Variant,
};

/// An attribute path name #[key(...)]
const KEY_IDENT: &str = "key";

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

    impl_try_from_keymap(&ast.ident, &variants).into()
}

/// Implements [`TryFrom<KeyMap>`] for enums.
fn impl_try_from_keymap(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
) -> proc_macro2::TokenStream {
    match build_match_arms(name, variants) {
        Ok(match_arms) => quote! {
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
        },
        Err(err) => err.to_compile_error(),
    }
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
