use keymap_parser::{parse_seq, Node};
use syn::{punctuated::Punctuated, token::Comma, Attribute, LitStr, Token, Variant};

/// An attribute path name #[key(...)]
const KEY_IDENT: &str = "key";
const DOC_IDENT: &str = "doc";

pub(crate) struct Item<'a> {
    pub variant: &'a Variant,
    /// Raw string representations of the keys (e.g., ["ctrl-c", "@any"]).
    pub keys: Vec<String>,
    /// Fully parsed nodes for each key sequence. Used for inspecting
    /// key groups (like @any, @digit) during Key Group Capturing.
    pub nodes: Vec<Vec<Node>>,
    pub ignore: bool,
    pub description: String,
}

pub(crate) fn parse_items(
    variants: &Punctuated<Variant, Comma>,
) -> Result<Vec<Item<'_>>, syn::Error> {
    // NOTE: All variants are parsed, even those without the #[key(...)] attribute.
    // This allows the deserializer to override keys and descriptions for variants that don't define them explicitly.
    variants
        .iter()
        .map(|variant| {
            let ignore = parse_ignore(variant);
            let (keys, nodes) = parse_keys(variant, ignore)?;

            Ok(Item {
                variant,
                ignore,
                description: parse_doc(variant),
                keys,
                nodes,
            })
        })
        .collect()
}

fn parse_ignore(variant: &Variant) -> bool {
    variant.attrs.iter().any(|attr| {
        let mut ignore = false;
        if attr.path().is_ident(KEY_IDENT) {
            let _ = attr.parse_nested_meta(|meta| {
                ignore = meta.path.is_ident("ignore");
                Ok(())
            });
        }

        ignore
    })
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

fn parse_keys(variant: &Variant, ignore: bool) -> syn::Result<(Vec<String>, Vec<Vec<Node>>)> {
    let mut keys = Vec::new();
    let mut nodes = Vec::new();

    for attr in &variant.attrs {
        if !attr.path().is_ident(KEY_IDENT) || ignore {
            continue;
        }

        // Collect arguments
        //
        // e.g. [["a"], ["g g"]]
        for arg in parse_args(attr)? {
            let val = arg.value();
            let seq = parse_seq(&val)
                .map_err(|e| syn::Error::new(arg.span(), format!("Invalid key \"{val}\": {e}")))?;

            keys.push(val);
            nodes.push(seq);
        }
    }

    Ok((keys, nodes))
}
