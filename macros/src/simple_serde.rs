use crate::utils::consume_comma;
use proc_macro2::{ TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Path};
#[derive(Debug, Default)]
pub enum DeserializeMethod{
    #[default]
    FromStr,
    TryFromString,
}
#[derive(Debug, Default)]
pub struct SerdeSettings {
    /// Will use as_ref() instead of to_string() when serializing
    ///
    /// The type must implement AsRef<str>
    pub has_as_ref_str: bool,
    pub deserialize_method: DeserializeMethod,
}
mod keywords {
    use syn::custom_keyword;
    custom_keyword!(as_ref);
    custom_keyword!(from_str);
    custom_keyword!(try_from_string);
}

impl Parse for SerdeSettings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut serde_settings = Self::default();
        while !input.is_empty() {
            let peek = input.lookahead1();
            if peek.peek(keywords::as_ref) {
                input.parse::<keywords::as_ref>()?;
                serde_settings.has_as_ref_str = true;
            } else if peek.peek(keywords::from_str) {
                input.parse::<keywords::from_str>()?;
                serde_settings.deserialize_method = DeserializeMethod::FromStr;
            } else if peek.peek(keywords::try_from_string) {
                input.parse::<keywords::try_from_string>()?;
                serde_settings.deserialize_method = DeserializeMethod::TryFromString;
            } else {
                return Err(peek.error());
            }
            consume_comma!(input);
        }
        Ok(serde_settings)
    }
}

pub(crate) fn expand_serialize(derive_input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput { ident, attrs, .. } = derive_input;
    let serde_settings = attrs
        .iter()
        .find(|attr| attr.path().is_ident("serde"))
        .map(|attr| attr.parse_args::<SerdeSettings>())
        .transpose()?
        .unwrap_or_default();
    let name_as_path = Path::from(ident);
    let result = expand_inner_serialize(&serde_settings, &name_as_path);
    Ok(result)
}
pub(crate) fn expand_deserialize(derive_input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput { ident, attrs, .. } = derive_input;
    let serde_settings = attrs
        .iter()
        .find(|attr| attr.path().is_ident("serde"))
        .map(|attr| attr.parse_args::<SerdeSettings>())
        .transpose()?
        .unwrap_or_default();
    let name_as_path = Path::from(ident);
    let result = expand_inner_deserialize(&serde_settings, &name_as_path);
    Ok(result)
}
pub(crate) fn expand_inner_serialize(settings:  &SerdeSettings, name: &Path) -> TokenStream{
    let serialize_body = if settings.has_as_ref_str {
        quote! {
            serializer.serialize_str(self.as_ref())
        }
    } else {
        quote! {
            let string = self.to_string();
            serializer.serialize_str(&string)
        }
    };

    let result = quote! {
        const _: () = {
                    #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;

        #[automatically_derived]
        impl _serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: _serde::Serializer,
            {
                #serialize_body
            }
        }
        };
        };
    return result;
}
pub(crate) fn expand_inner_deserialize(settings: &SerdeSettings, name: &Path) -> TokenStream {

    let deserialize_method = match settings.deserialize_method {
        DeserializeMethod::FromStr => quote! {
            use std::str::FromStr;
            use _serde::de::{Error, Unexpected, Visitor};
            use _serde::Deserialize;
            struct VisitorImpl;
            impl<'de> Visitor<'de> for VisitorImpl {
                type Value = #name;
                fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    formatter.write_str("a valid enum variant")
                }
                fn visit_str<E>(self, value: &str) -> ::core::result::Result<Self::Value, E>
                where
                    E: Error,
                {
                    #name::from_str(value).map_err(|e| {
                        E::custom(e)
                    })
                }
                fn visit_string<E>(self, value: String) -> ::core::result::Result<Self::Value, E>
                where
                    E: Error,
                {
                    #name::from_str(&value).map_err(|e| {
                        E::custom(e)
                    })
                }
            }
            deserializer.deserialize_str(VisitorImpl)

        },
        DeserializeMethod::TryFromString => quote! {
            use core::convert::TryFrom;
            use _serde::de::{Error, Unexpected, Visitor};
            use _serde::Deserialize;
             struct VisitorImpl;
            impl<'de> Visitor<'de> for VisitorImpl {
                type Value = #name;
                fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    formatter.write_str("a valid enum variant")
                }
                fn visit_str<E>(self, value: &str) -> ::core::result::Result<Self::Value, E>
                where
                    E: Error,
                {
                    #name::try_from(value.to_string()).map_err(|e| {
                        E::custom(e)
                    })
                }
                fn visit_string<E>(self, value: String) -> ::core::result::Result<Self::Value, E>
                where
                    E: Error,
                {
                    #name::try_from(value).map_err(|e| {
                        E::custom(e)
                    })
                }
            }
            deserializer.deserialize_string(VisitorImpl)
        },
    };
    let result = quote! {
        const _: () = {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate serde as _serde;
        #[automatically_derived]
        impl<'de> _serde::Deserialize<'de> for #name{
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: _serde::Deserializer<'de>,
            {
                #deserialize_method
            }
        }
            };
    };
    return result;
}
