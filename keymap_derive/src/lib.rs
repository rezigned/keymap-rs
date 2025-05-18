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
/// ```rust
/// use keymap::KeyMap;
/// use keymap_derive::KeyMap;
///
/// #[derive(KeyMap)]
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

fn impl_try_from_keymap(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
) -> proc_macro2::TokenStream {
    let match_arms = build_match_arms(name, variants);

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

/// Builds match arms from key attributes.
///
/// # Example
///
/// ```rust
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
/// ```rust
/// ["c"] => Ok(Action::Create),
/// ["x"] | ["d"] => Ok(Action::Delete),
/// ```
fn build_match_arms(
    name: &Ident,
    variants: &Punctuated<Variant, Comma>,
) -> Vec<proc_macro2::TokenStream> {
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
                    let keys = lit_strs.iter().map(|str| {
                        let val = str.value();

                        // Split string into a list of keys e.g.
                        //
                        // "a b" => [quote! { a }, quote! { b }]
                        let seq = val.split_whitespace().map(|key| quote! { #key });

                        // Build sequence of keys e.g.
                        //
                        // [quote! { a }, quote! { b }] => ["a", "b"]
                        quote! { [#(#seq),*] }
                    });

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

    arms
}
