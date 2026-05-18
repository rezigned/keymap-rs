use keymap_parser::{parse_seq, Node};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, Token, Variant};

/// An attribute path name #[key(...)]
const KEY_IDENT: &str = "key";
const DOC_IDENT: &str = "doc";

pub(crate) struct Item<'a> {
    pub variant: &'a Variant,
    /// Raw string representations of the keys (e.g., ["ctrl-c", "@any", "g g"]).
    pub keys: Vec<String>,
    /// Fully parsed nodes for each key sequence. Used for inspecting
    /// key groups (like @any, @digit) during Key Group Capturing.
    pub nodes: Vec<Vec<Node>>,
    pub ignore: bool,
    pub description: String,
    pub symbol: Option<String>,
    pub help: Option<String>,
}

/// Helper struct representing the arguments parsed from a `#[key(...)]` attribute.
///
/// It supports a hybrid syntax:
/// 1. Positional string literals (e.g. `"ctrl-b"`, `"space"`), which represent the keys to bind.
/// 2. The `ignore` boolean flag (e.g. `#[key(ignore)]`).
/// 3. Named name-value fields:
///    - `symbol = "..."` (e.g. `symbol = "^B"`) defining a custom quick visual symbol for display.
///    - `help = "..."` (e.g. `help = "jump"`) defining a short help text description for the binding.
///
/// Example:
///
/// #[key("ctrl-b", symbol = "^B", help = "jump")]
///    |  |______|  |__________|   |___________|
///  path   keys       symbol           help
struct KeyAttrArgs {
    keys: Vec<String>,
    ignore: bool,
    symbol: Option<String>,
    help: Option<String>,
}

impl syn::parse::Parse for KeyAttrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut keys = Vec::new();
        let mut ignore = false;
        let mut symbol = None;
        let mut help = None;

        while !input.is_empty() {
            if input.peek(syn::LitStr) {
                // Parse positional key bindings like "ctrl-b"
                let lit: syn::LitStr = input.parse()?;
                keys.push(lit.value());
            } else if input.peek(syn::Ident) {
                let ident: syn::Ident = input.parse()?;
                if ident == "ignore" {
                    // Parse the single 'ignore' flag
                    ignore = true;
                } else if ident == "symbol" {
                    // Parse 'symbol = "..."'
                    let _: Token![=] = input.parse()?;
                    let lit: syn::LitStr = input.parse()?;
                    symbol = Some(lit.value());
                } else if ident == "help" {
                    // Parse 'help = "..."'
                    let _: Token![=] = input.parse()?;
                    let lit: syn::LitStr = input.parse()?;
                    help = Some(lit.value());
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("Unknown key attribute argument: {}", ident),
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "Expected string literal or identifier",
                ));
            }

            // Consume optional comma separator if there are remaining arguments
            if !input.is_empty() {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(KeyAttrArgs {
            keys,
            ignore,
            symbol,
            help,
        })
    }
}

pub(crate) fn parse_items(
    variants: &Punctuated<Variant, Comma>,
) -> Result<Vec<Item<'_>>, syn::Error> {
    variants
        .iter()
        .map(|variant| {
            let mut keys = Vec::new();
            let mut nodes = Vec::new();
            let mut ignore = false;
            let mut symbol = None;
            let mut help = None;

            for attr in &variant.attrs {
                if attr.path().is_ident(KEY_IDENT) {
                    let args: KeyAttrArgs = attr.parse_args()?;
                    if args.ignore {
                        ignore = true;
                    }
                    for key in args.keys {
                        let seq = parse_seq(&key).map_err(|e| {
                            syn::Error::new(attr.span(), format!("Invalid key \"{key}\": {e}"))
                        })?;
                        keys.push(key);
                        nodes.push(seq);
                    }
                    if args.symbol.is_some() {
                        symbol = args.symbol;
                    }
                    if args.help.is_some() {
                        help = args.help;
                    }
                }
            }

            Ok(Item {
                variant,
                ignore,
                description: parse_doc(variant),
                keys,
                nodes,
                symbol,
                help,
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
