use crate::utils::{consume_comma, into_enum};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::Parse;

use syn::{bracketed, Fields, LitStr, Result, Token, Variant};
use syn::{DeriveInput, Error};

mod keywords {
    use syn::custom_keyword;
    custom_keyword!(equals);
    custom_keyword!(contains);
    custom_keyword!(include_variant);
    custom_keyword!(deref_str);
    custom_keyword!(partial_eq);
    custom_keyword!(to_lowercase);
}
#[derive(Debug)]
pub struct TypeAttribute {
    pub include_variant: bool,
    pub partial_eq: bool,
    pub to_lowercase: bool,
}
impl Default for TypeAttribute {
    fn default() -> Self {
        Self {
            include_variant: true,
            partial_eq: true,
            to_lowercase: false,
        }
    }
}
impl Parse for TypeAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut type_attribute = Self::default();
        while !input.is_empty() {
            let peek = input.lookahead1();
            if peek.peek(keywords::include_variant) {
                input.parse::<keywords::include_variant>()?;
                if input.parse::<Token![=]>().is_ok() {
                    type_attribute.include_variant = input.parse::<syn::LitBool>()?.value();
                } else {
                    type_attribute.include_variant = true;
                }
            } else if peek.peek(keywords::to_lowercase) {
                input.parse::<keywords::to_lowercase>()?;
                if input.parse::<Token![=]>().is_ok() {
                    type_attribute.to_lowercase = input.parse::<syn::LitBool>()?.value();
                } else {
                    type_attribute.to_lowercase = true;
                }
            } else if peek.peek(keywords::partial_eq) {
                input.parse::<keywords::partial_eq>()?;
                if input.parse::<Token![=]>().is_ok() {
                    type_attribute.partial_eq = input.parse::<syn::LitBool>()?.value();
                } else {
                    type_attribute.partial_eq = true;
                }
            } else {
                return Err(peek.error());
            }
            let _ = input.parse::<Token![,]>();
        }
        Ok(type_attribute)
    }
}
/// This attribute is used to generate an compare to str
///
/// ```ignore
/// #[compare_str(equals["uuid", "uuid::Uuid"], contains["uuid"])]
/// ```
#[derive(Debug, Default)]
pub struct CompareToStrAttribute {
    pub equals: Vec<LitStr>,
    pub contains: Vec<LitStr>,
}
impl Parse for CompareToStrAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut equals = Vec::new();
        let mut contains = Vec::new();
        while !input.is_empty() {
            let peek = input.lookahead1();
            if peek.peek(keywords::equals) {
                input.parse::<keywords::equals>()?;
                let content;
                bracketed!(content in input);
                let equals_list = content.parse_terminated(Parse::parse, Token![,])?;
                equals = equals_list.into_iter().collect();
            } else if peek.peek(keywords::contains) {
                input.parse::<keywords::contains>()?;
                let content;
                bracketed!(content in input);
                let list = content.parse_terminated(Parse::parse, Token![,])?;
                contains = list.into_iter().collect();
            } else {
                return Err(peek.error());
            }
            consume_comma!(input);
        }
        Ok(Self { equals, contains })
    }
}
#[derive(Debug)]
pub struct CompareToStrVariant {
    pub name: Ident,
    pub fields_collection: TokenStream,
    pub attributes: CompareToStrAttribute,
}
impl TryFrom<Variant> for CompareToStrVariant {
    type Error = Error;

    fn try_from(value: Variant) -> std::result::Result<Self, Self::Error> {
        let mut compare_attr = CompareToStrAttribute::default();
        for attr in value.attrs {
            if attr.path().is_ident("compare_str") {
                let CompareToStrAttribute { equals, contains } = attr.parse_args()?;
                compare_attr.equals.extend(equals);
                compare_attr.contains.extend(contains);
            }
        }
        let fields_collection = match value.fields {
            Fields::Named(_) => {
                quote! {
                    { .. }
                }
            }
            Fields::Unnamed(_) => {
                quote! {
                    (..)
                }
            }
            Fields::Unit => {
                quote! {}
            }
        };
        Ok(Self {
            name: value.ident,
            fields_collection,
            attributes: compare_attr,
        })
    }
}

impl ToTokens for CompareToStrVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            fields_collection,
            attributes,
        } = self;
        let CompareToStrAttribute { equals, contains } = attributes;
        if contains.is_empty() {
            tokens.append_all(quote! {
                Self::#name #fields_collection =>{
                    return #(other == #equals)||*;
                }
            });
        } else {
            tokens.append_all(quote! {
            Self::#name #fields_collection =>{
                return if #(other == #equals)||* {
                    true
                }else{
                    #(other.contains(#contains))||*
                };
            }
            });
        }
    }
}

pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        ident, data, attrs, ..
    } = derive_input;
    let type_attr = attrs
        .into_iter()
        .find(|attr| attr.path().is_ident("compare_str"))
        .map(|v| v.parse_args::<TypeAttribute>())
        .transpose()?
        .unwrap_or_default();
    let data = into_enum!(data, ident, "CompareToStr");
    let variants: Result<Vec<CompareToStrVariant>> = data
        .variants
        .into_iter()
        .map(|v| {
            let mut result = CompareToStrVariant::try_from(v);
            if let Ok(variant) = &mut result {
                if type_attr.include_variant {
                    variant
                        .attributes
                        .equals
                        .push(LitStr::new(&variant.name.to_string(), variant.name.span()));
                }
            }
            result
        })
        .collect();
    let variants = variants?;
    let to_lower_case = if type_attr.to_lowercase {
        quote! {
            let other = other.to_lowercase();
        }
    } else {
        quote! {}
    };
    // TODO Improve Doc Comment to show what it is checking for
    let mut result = quote! {
        impl #ident{
            #[doc="Compares an enum variant to a str"]
            #[automatically_derived]
            pub fn equals_str(&self, other: impl core::convert::AsRef<str>) -> bool {
                let other = other.as_ref();
                #to_lower_case
                    match self {
                        #(#variants)*
                    }
            }
        }
    };
    if type_attr.partial_eq {
        let impl_trait = quote! {
            #[automatically_derived]
            impl core::cmp::PartialEq<str> for #ident {
                fn eq(&self, other: &str) -> bool {
                   #to_lower_case
                    match self {
                        #(#variants)*
                    }
                }
            }
            #[automatically_derived]
            impl core::cmp::PartialEq<&str> for #ident {
                fn eq(&self, other: &&str) -> bool {
                        let other = *other;
                                   #to_lower_case
                    match self {
                        #(#variants)*
                    }
                }
            }
        };
        result.append_all(impl_trait);
    }

    Ok(result)
}
