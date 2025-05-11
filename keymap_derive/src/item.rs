use keymap_parser::{parse_seq, Node};
use syn::{punctuated::Punctuated, token::Comma, Attribute, LitStr, Token, Variant};

/// An attribute path name #[key(...)]
const KEY_IDENT: &str = "key";

const DOC_IDENT: &str = "doc";

pub(crate) struct Item<'a> {
    pub variant: &'a Variant,
    pub keys: Vec<String>,
    pub nodes: Vec<Vec<Node>>,
    pub description: String,
}

pub(crate) fn parse_items(variants: &Punctuated<Variant, Comma>) -> Result<Vec<Item>, syn::Error> {
    variants
        .iter()
        .filter(|variant| {
            // Only keep variants with #[key(...)]
            variant
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident(KEY_IDENT))
        })
        .map(|variant| {
            Ok(Item {
                variant,
                description: parse_doc(variant),
                keys: parse_keys(variant)?,
                nodes: parse_nodes(variant)?,
            })
        })
        .collect()
}

fn parse_doc(variant: &Variant) -> String {
    variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident(DOC_IDENT))
        .filter_map(|attr| match &attr.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) => Some(lit_str.value().trim().to_string()),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse attribute arguments.
///
/// Example:
///
/// #[key("a", "g g")]
///    |  |________|
///  path   (args)
fn parse_args(attr: &Attribute) -> syn::Result<Punctuated<LitStr, Token![,]>> {
    attr.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_separated_nonempty)
}

fn parse_keys(variant: &Variant) -> syn::Result<Vec<String>> {
    let mut keys = Vec::new();

    for attr in &variant.attrs {
        if !attr.path().is_ident(KEY_IDENT) {
            continue;
        }

        // Collect arguments
        //
        // e.g. [["a"], ["g g"]]
        for arg in parse_args(attr)? {
            let val = arg.value();
            parse_seq(&val)
                .map_err(|e| syn::Error::new(arg.span(), format!("Invalid key \"{val}\": {e}")))?;

            keys.push(val);
        }
    }

    Ok(keys)
}

fn parse_nodes(variant: &Variant) -> syn::Result<Vec<Vec<Node>>> {
    let mut nodes = Vec::new();

    for attr in &variant.attrs {
        if !attr.path().is_ident(KEY_IDENT) {
            continue;
        }

        // Collect arguments
        //
        // e.g. [["a"], ["g g"]]
        for arg in parse_args(attr)? {
            let val = arg.value();
            let keys = parse_seq(&val)
                .map_err(|e| syn::Error::new(arg.span(), format!("Invalid key \"{val}\": {e}")))?;

            nodes.push(keys);
        }
    }

    Ok(nodes)
}
