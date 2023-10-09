use crate::utils::consume_comma;
use crate::simple_serde::SerdeSettings;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use std::fmt::Debug;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Path, Token};

mod keywords {
    use syn::custom_keyword;
    custom_keyword!(default);
    custom_keyword!(default_in_cow);
    custom_keyword!(impl_common_traits);
    custom_keyword!(serde);
    custom_keyword!(impl_strum);
}

/// This attribute is used to generate an enum of keys for a struct.
/// #[enum_of_keys(KeyEnumName)]
#[derive(Debug)]
pub struct EnumOfKeysAttribute {
    /// Required - The name of the KeyEnum
    /// #[enum_of_keys(KeyEnumName)]
    pub name: Path,
    /// Store the default variant in a Cow
    /// #[enum_of_keys(KeyEnumName, default_in_cow)]
    pub store_default_in_cow: bool,
    /// Implement common traits for the enum of keys such as
    /// Add derive(Clone, Copy, Debug, PartialEq, Eq) to the enum of keys
    pub impl_common_traits: bool,
    /// Implements serde::Serialize and serde::Deserialize for the enum of keys
    /// Requires std::fmt::Display and std::str::FromStr to be implemented for the enum of keys
    /// You can use the strum crate to implement these traits easily
    pub serde: Option<SerdeSettings>,
    /// Add derive(strum::EnumIter, strum::EnumString, strum::Display, strum::EnumIs, strum::AsRefStr)
    /// to the enum of keys
    pub impl_strum: bool,
}

impl Parse for EnumOfKeysAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let mut default_in_cow = false;
        let mut impl_common_traits = false;
        let mut serde = None;
        let mut impl_strum = false;
        consume_comma!(input);

        while !input.is_empty() {
            let lookahead1 = input.lookahead1();
            if lookahead1.peek(keywords::default_in_cow) {
                input.parse::<keywords::default_in_cow>()?;
                default_in_cow = true;
            } else if lookahead1.peek(keywords::impl_common_traits) {
                input.parse::<keywords::impl_common_traits>()?;
                impl_common_traits = true;
            } else if lookahead1.peek(keywords::serde) {
                input.parse::<keywords::serde>()?;
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    serde = Some(SerdeSettings::default());
                    continue;
                } else {
                    let content;
                    parenthesized!(content in input);
                    serde = Some(content.parse::<SerdeSettings>()?);
                }
            } else if lookahead1.peek(keywords::impl_strum) {
                input.parse::<keywords::impl_strum>()?;
                impl_strum = true;
            } else {
                return Err(lookahead1.error());
            }
            consume_comma!(input);
        }
        if impl_strum {
            if let Some(serde_settings) = serde.as_mut() {
                serde_settings.has_as_ref_str = true;
            }
        }
        Ok(EnumOfKeysAttribute {
            name,
            store_default_in_cow: default_in_cow,
            impl_common_traits,
            serde,
            impl_strum,
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
