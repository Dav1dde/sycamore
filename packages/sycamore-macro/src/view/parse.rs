//! Parse syntax for `view!` macro.
//!

use std::fmt;

use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Result, Token};

use super::ir::*;

impl Parse for ElementTag {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident::peek_any) {
            // Builtin element.
            let ident: Ident = input.call(Ident::parse_any)?;
            Ok(Self::Builtin(ident))
        } else if input.peek(LitStr) {
            // Custom element.
            let name: LitStr = input.parse()?;
            Ok(Self::Custom(name.value()))
        } else {
            Err(input.error("expected an element"))
        }
    }
}

impl Parse for AttributeType {
    fn parse(input: ParseStream) -> Result<Self> {
        pub struct AttributeName {
            tag: Ident,
            extended: Vec<(Token![-], Ident)>,
        }

        impl Parse for AttributeName {
            fn parse(input: ParseStream) -> Result<Self> {
                let tag = input.call(Ident::parse_any)?;
                let mut extended = Vec::new();
                while input.peek(Token![-]) {
                    extended.push((input.parse()?, input.parse()?));
                }

                Ok(Self { tag, extended })
            }
        }

        impl fmt::Display for AttributeName {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let AttributeName { tag, extended } = self;

                write!(f, "{}", tag)?;
                for (_, ident) in extended {
                    write!(f, "-{}", ident)?;
                }

                Ok(())
            }
        }

        let ident: AttributeName = input.parse()?;
        let name = ident.to_string();

        if name == "ref" {
            Ok(Self::Ref)
        } else if name == "dangerously_set_inner_html" {
            Ok(Self::DangerouslySetInnerHtml)
        } else if input.peek(Token![:]) {
            let _colon: Token![:] = input.parse()?;
            match name.as_str() {
                "on" => {
                    let event = input.call(Ident::parse_any)?;
                    Ok(Self::Event {
                        event: event.to_string(),
                    })
                }
                "bind" => {
                    let prop = input.call(Ident::parse_any)?;
                    Ok(Self::Bind {
                        prop: prop.to_string(),
                    })
                }
                _ => Err(syn::Error::new_spanned(
                    ident.tag,
                    format!("unknown directive `{}`", name),
                )),
            }
        } else if is_bool_attr(&name) {
            Ok(Self::Bool { name })
        } else {
            Ok(Self::Str { name })
        }
    }
}
