use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::fmt::Debug;
use syn::parse::{Parse, ParseStream};
use syn::{Path, Token};

mod keywords {
    use syn::custom_keyword;
    custom_keyword!(default);
    custom_keyword!(default_in_cow);
}

/// This attribute is used to generate an enum of keys for a struct.
/// #[enum_of_keys(KeyEnumName, derive(...))]
#[derive(Debug)]
pub struct EnumOfKeysAttribute {
    pub name: Path,
    pub store_default_in_cow: bool,
}

impl Parse for EnumOfKeysAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let mut default_in_cow = false;
        if input.parse::<Token![,]>().is_ok() {
            let lookahead1 = input.lookahead1();
            if lookahead1.peek(keywords::default_in_cow) {
                input.parse::<keywords::default_in_cow>()?;
                default_in_cow = true;
            } else {
                return Err(lookahead1.error());
            }
        }

        Ok(EnumOfKeysAttribute {
            name,
            store_default_in_cow: default_in_cow,
        })
    }
}

/// A inner attribute is an attribute that is inside a bracket.
///
/// # Example
/// ```rust, ignore
/// #[enum_attr(strum(serialize_all = "UPPERCASE"))]
/// #[enum_attr(derive(Clone, Copy, Debug, PartialEq, Eq))]
/// #[enum_attr(strum(default))]
/// ```
#[derive(Debug)]
pub struct InnerAttribute {
    pub meta: syn::Meta,
}

impl Parse for InnerAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta = input.parse::<syn::Meta>()?;
        Ok(InnerAttribute { meta })
    }
}
impl ToTokens for InnerAttribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let meta = self.meta.clone().into_token_stream();
        let value = quote! {
            #[#meta]
        };
        tokens.append_all(value);
    }
}

#[derive(Debug)]
pub struct DefaultValue {
    pub key_name: Ident,
    pub unwrap_variant: Option<TokenStream>,
}
#[derive(Debug)]
pub struct VariantAttribute {
    pub default: Option<DefaultValue>,
}

impl Parse for VariantAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut default: Option<DefaultValue> = None;
        // Loop through tokens seperated by ,

        while !input.is_empty() {
            let peak = input.lookahead1();
            if peak.peek(keywords::default) {
                input.parse::<keywords::default>()?;
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    let key_name = input.parse::<syn::Ident>()?;
                    default = Some(DefaultValue {
                        key_name,
                        unwrap_variant: None,
                    });
                } else {
                    default = Some(DefaultValue {
                        key_name: format_ident!("value"),
                        unwrap_variant: None,
                    });
                }
                input.parse::<Option<Token![,]>>()?;
            } else {
                return Err(peak.error());
            }
        }

        Ok(VariantAttribute { default })
    }
}
