use keymap_parser::parse_seq;
use syn::{punctuated::Punctuated, token::Comma, LitStr, Token, Variant};

/// An attribute path name #[key(...)]
const KEY_IDENT: &str = "key";

const DOC_IDENT: &str = "doc";

pub(crate) struct Item<'a> {
    pub variant: &'a Variant,
    pub keys: Vec<String>,
    pub description: String,
}

pub(crate) fn parse_items(variants: &Punctuated<Variant, Comma>) -> Result<Vec<Item>, syn::Error> {
    variants
        .iter()
        .map(|variant| {
            Ok(Item {
                variant,
                description: parse_doc(variant),
                keys: parse_keys(variant)?,
            })
        })
        .collect()
}

fn parse_doc(variant: &Variant) -> String {
    let mut doc = String::new();

    for attr in &variant.attrs {
        if !attr.path().is_ident(DOC_IDENT) {
            continue;
        }

        // Parse name-value pair e.g. #[doc = "..."]
        if let syn::Meta::NameValue(meta_name_value) = &attr.meta {
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
        // e.g. [["a"], ["g g"]]
        match attr.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_separated_nonempty) {
            Ok(lit_strs) => {
                for str in lit_strs {
                    // Directly use the string value, quote! will handle making it a literal
                    let val = str.value();

                    // Parse key sequence, and return early on error
                    let _ = parse_seq(&val).map_err(|err| {
                        syn::Error::new(str.span(), format!("Invalid key: \"{val}\". {err}"))
                    })?;

                    keys.push(val);
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    Ok(keys)
}
