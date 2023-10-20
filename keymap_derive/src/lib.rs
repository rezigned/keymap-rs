use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::ParseBuffer,
    punctuated::Punctuated,
    token::{self, Comma}, Attribute, DeriveInput, Ident, LitStr, Token, Variant,
};

const KEY_IDENT: &'static str = "key";

/// Generates the `TryFrom<KeyMap>` implementation.
#[proc_macro_derive(KeyMap, attributes(key))]
pub fn keymap(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    match ast.data {
        syn::Data::Enum(ref data) => {
            impl_from_keymap(&ast.ident, &data.variants).into()
        }
        _ => panic!("#[derive(KeyMap)] is only defined for enums"),
    }
}

/// Implements a conversion from [`keymap::KeyMap`] for the target [`enum`].
fn impl_from_keymap(enum_name: &Ident, variants: &Punctuated<Variant, Comma>) -> proc_macro2::TokenStream {
    let patterns = Patterns::from(variants);
    let match_keys = patterns.match_keys;
    let match_variants = patterns.match_variants;
    let key_val_pairs = patterns.key_val_pairs;

    quote! {
        use std::collections::HashMap;
        use keymap::{KeyMap, KeyValPair};

        impl TryFrom<KeyMap> for #enum_name {
            type Error = String;

            fn try_from(value: keymap::KeyMap) -> Result<Self, Self::Error> {
                let key = format!("{}", value);

                match key.as_str() {
                    #(#match_keys)*
                    _ => Err(format!("Unhandle key {key}")),
                }
            }
        }

        impl KeyValPair<Self> for #enum_name {
            fn keymaps() -> HashMap<Vec<&'static str>, Self> {
                HashMap::from([
                    #(#key_val_pairs)*
                ])
            }
        }

        impl #enum_name {
            /// Returns a list of keys from [`Variant`].
            pub fn keymap_keys(self) -> Vec<&'static str> {
                match self {
                    #(#match_variants)*
                }
            }
        }
    }
}

/// Represents the match arms patterns generated from [`Variant`]
#[derive(Default)]
struct Patterns {
    // A collection of `"key" => Action` match arms
    match_keys: Vec<proc_macro2::TokenStream>,

    // A collection of `Action => vec!["key"]` match arms
    match_variants: Vec<proc_macro2::TokenStream>,

    // A collection of `Action => vec!["key"]` match arms
    key_val_pairs: Vec<proc_macro2::TokenStream>,
}

impl From<&Punctuated<Variant, Comma>> for Patterns {
    /// Generates match arms from key attributes.
    fn from(variants: &Punctuated<Variant, Comma>) -> Self {
        let mut match_variants = vec![];
        let mut match_keys = vec![];
        let mut key_val_pairs = vec![];

        variants
            .iter()
            .for_each(|variant| {
                let variant_name = &variant.ident;

                variant
                    .attrs
                    .iter()
                    .filter(|v| v.path().is_ident(KEY_IDENT))
                    .for_each(|attr| {
                        let keys = parse_keys(&attr);

                        // Generates `Action => vec!["key"]` match arms
                        match_variants.push(quote! {
                            Self::#variant_name => vec![#(#keys),*],
                        }.into());

                        key_val_pairs.push(quote! {
                            (vec![#(#keys),*], Self::#variant_name),
                        });

                        // Generates `"key" => Action` match arms
                        match_keys.push(quote! {
                            #(#keys)|* => Ok(Self::#variant_name),
                        }.into());
                    });
            });

        Self { match_keys, match_variants, key_val_pairs }
    }
}

/// Parser for comma-separated string literals
const KEY_ITEM_PARSER: for<'a> fn(
    &'a ParseBuffer<'a>,
) -> Result<Punctuated<LitStr, token::Comma>, syn::Error> =
    Punctuated::<LitStr, Token![,]>::parse_separated_nonempty;

// Parses comma-separated string literals in attribute
//
// For example this would return "a", "b", ... in #[key("a", "b", ...)]
fn parse_keys(attr: &Attribute) -> Vec<LitStr> {
    if let syn::Meta::List(list) = &attr.meta {
        if let Ok(items) = list.parse_args_with(KEY_ITEM_PARSER) {
            return Vec::from_iter(items.into_iter());
        }
    }

    vec![]
}
