mod attrs;

use crate::enum_of_keys_impl::attrs::{EnumOfKeysAttribute, InnerAttribute, VariantAttribute};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use std::env::var;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{Attribute, Fields, Meta, Path, Result};
use syn::{DeriveInput, Error};

#[derive(Debug)]
pub struct Variant {
    pub name: Ident,
    pub enum_of_keys_attr: Option<VariantAttribute>,
    pub inner_attrs: Vec<InnerAttribute>,
    pub default_in_cow: bool,
    pub fields_collection: TokenStream,
}
impl Variant {
    pub fn new(variant: syn::Variant, default_in_cow: bool) -> Result<Self> {
        let attributes: Option<VariantAttribute> = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("enum_of_keys"))
            .map(|v| v.parse_args())
            .transpose()?;
        let inner_attrs = find_and_parse_inner_attrs(&variant.attrs)?;
        if attributes.as_ref().map(|v| v.default).unwrap_or(false) {
            if let Fields::Unnamed(value) = &variant.fields {
                if value.unnamed.len() != 1 {
                    return Err(Error::new(
                        variant.span(),
                        "The default can only be a single value Tuple Variant",
                    ));
                }
            } else {
                return Err(Error::new(
                    variant.span(),
                    "The default can only be a single value Tuple Variant",
                ));
            }
        }
        let fields_collection = match variant.fields {
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

        Ok(Variant {
            name: variant.ident,
            enum_of_keys_attr: attributes,
            inner_attrs,
            default_in_cow,
            fields_collection,
        })
    }

    pub fn create_get_key_line_owned(&self, enum_name: Ident, key_enum_name: Path) -> TokenStream {
        let Self {
            name,
            fields_collection,
            enum_of_keys_attr,
            ..
        } = self;
        if enum_of_keys_attr
            .as_ref()
            .map(|v| v.default)
            .unwrap_or(false)
        {
            quote! {
                #enum_name::#name(value) => #key_enum_name::#name(::std::borrow::Cow::Owned(value.clone()))
            }
        } else {
            quote! {
                #enum_name::#name #fields_collection => #key_enum_name::#name
            }
        }
    }
    pub fn create_get_key_line(&self, enum_name: Ident, key_enum_name: Path) -> TokenStream {
        let Self {
            name,
            fields_collection,
            enum_of_keys_attr,
            ..
        } = self;
        if enum_of_keys_attr
            .as_ref()
            .map(|v| v.default)
            .unwrap_or(false)
        {
            if self.default_in_cow {
                quote! {
                    #enum_name::#name(value) => #key_enum_name::#name(::std::borrow::Cow::Borrowed(value))
                }
            } else {
                quote! {
                    #enum_name::#name(value) => #key_enum_name::#name(value.clone())
                }
            }
        } else {
            quote! {
                #enum_name::#name #fields_collection => #key_enum_name::#name
            }
        }
    }

    pub fn create_partial_eq_line(&self, enum_name: Ident, key_enum_name: Path) -> TokenStream {
        let Self {
            name,
            enum_of_keys_attr,
            fields_collection,
            ..
        } = self;
        if enum_of_keys_attr
            .as_ref()
            .map(|v| v.default)
            .unwrap_or(false)
        {
            quote! {
                (#enum_name::#name(a), #key_enum_name::#name(b)) => a == b
            }
        } else {
            quote! {
                (#enum_name::#name #fields_collection, #key_enum_name::#name) => true
            }
        }
    }
}
impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Variant {
            name,
            inner_attrs,
            enum_of_keys_attr,
            ..
        } = self;
        if let Some(value) = enum_of_keys_attr.as_ref() {
            if value.default {
                let value = if self.default_in_cow {
                    quote! {
                        #(#inner_attrs)*
                        #name(::std::borrow::Cow<'a, str>)
                    }
                } else {
                    quote! {
                         #(#inner_attrs)*
                         #name(String)
                    }
                };
                tokens.append_all(value);
                return;
            }
        }
        let value = quote! {
            #(#inner_attrs)*
            #name
        };
        tokens.append_all(value);
    }
}
pub fn find_and_parse_inner_attrs(attrs: &Vec<Attribute>) -> Result<Vec<InnerAttribute>> {
    let mut result = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("enum_attr") {
            result.push(attr.parse_args()?)
        }
    }

    Ok(result)
}
pub(crate) fn expand(derive_input: DeriveInput) -> Result<TokenStream> {
    let name = derive_input.ident;
    let enum_attributes: EnumOfKeysAttribute = derive_input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("enum_of_keys"))
        .ok_or(Error::new(name.span(), "Missing enum_of_keys attribute"))?
        .parse_args()?;
    let mut inner_attrs = find_and_parse_inner_attrs(&derive_input.attrs)?;
    if let Some(value) = derive_input
        .attrs
        .iter()
        .find(|v| v.path().is_ident("non_exhaustive"))
    {
        inner_attrs.push(InnerAttribute {
            meta: value.meta.clone(),
        })
    }
    let data_enum = match derive_input.data {
        syn::Data::Enum(data_enum) => data_enum,
        _ => {
            return Err(Error::new(
                name.span(),
                "EnumOfKeys can only be used with enums",
            ))
        }
    };
    let mut variants = Vec::with_capacity(data_enum.variants.len());
    let mut partial_eq_lines = Vec::with_capacity(data_enum.variants.len());
    let mut get_key_lines = Vec::with_capacity(data_enum.variants.len());
    let mut get_key_lines_owned = Vec::with_capacity(data_enum.variants.len());

    for variant in data_enum.variants {
        let variant = Variant::new(variant, enum_attributes.store_default_in_cow)?;
        partial_eq_lines
            .push(variant.create_partial_eq_line(name.clone(), enum_attributes.name.clone()));
        get_key_lines.push(variant.create_get_key_line(name.clone(), enum_attributes.name.clone()));
        if enum_attributes.store_default_in_cow {
            get_key_lines_owned
                .push(variant.create_get_key_line_owned(name.clone(), enum_attributes.name.clone()))
        }
        variants.push(variant);
    }

    let EnumOfKeysAttribute {
        name: enum_name,
        store_default_in_cow,
    } = enum_attributes;
    let result = if store_default_in_cow {
        expand_cow(
            name,
            &mut inner_attrs,
            &mut variants,
            &mut partial_eq_lines,
            &mut get_key_lines,
            enum_name,
            &mut get_key_lines_owned,
        )?
    } else {
        expand_no_cow(
            name,
            &mut inner_attrs,
            &mut variants,
            &mut partial_eq_lines,
            &mut get_key_lines,
            enum_name,
        )
    };

    Ok(result)
}

fn expand_cow(
    name: Ident,
    inner_attrs: &mut Vec<InnerAttribute>,
    variants: &mut Vec<Variant>,
    partial_eq_lines: &mut Vec<TokenStream>,
    get_key_lines: &mut Vec<TokenStream>,
    enum_name: Path,
    get_key_owned_lines: &mut Vec<TokenStream>,
) -> Result<TokenStream> {
    let mut to_owned_catches = Vec::new();
    for variant in variants.iter() {
        let variant_name = &variant.name;
        if variant
            .enum_of_keys_attr.as_ref()
            .map(|v| v.default)
            .unwrap_or_default()
        {
            to_owned_catches.push(quote! {
                #enum_name::#variant_name(v) => #enum_name::#variant_name(::std::borrow::Cow::Owned(v.as_ref().to_owned()))
            })
        } else {
            to_owned_catches.push(quote! {
                #enum_name::#variant_name => #enum_name::#variant_name

            })
        }
    }
    let result = quote! {
        #[automatically_derived]
        #(#inner_attrs)*
        pub enum #enum_name<'a>{
            #(#variants),*
        }
        #[automatically_derived]
       impl enum_helper::KeyEnum for #enum_name<'_>{ }
        #[automatically_derived]
        impl #enum_name<'_> {
            pub fn to_owned(&self) -> #enum_name<'static>{
                match self{
                    #(#to_owned_catches),*
                }
            }
        }
        #[automatically_derived]
        impl enum_helper::HasKeyEnum for #name{
            type KeyEnum<'a> = #enum_name<'a> where Self: 'a;
            fn get_key(&self) -> Self::KeyEnum<'static>{
                match self{
                    #(#get_key_owned_lines),*
                }
            }
             fn get_key_borrowed(&self) -> Self::KeyEnum<'_>{
                 match self{
                    #(#get_key_lines),*
                }
             }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialEq<#enum_name<'_>> for #name{
            fn eq(&self, other: &#enum_name<'_>) -> bool {
                 match (self, other) {
                    #(#partial_eq_lines),*
                    ,
                    _ => false
                 }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialEq<#name> for #enum_name<'_>{
            fn eq(&self, other: &#name) -> bool {
                 match ( other,self) {
                    #(#partial_eq_lines),*
                    ,
                    _ => false
                 }
            }
        }
    };
    Ok(result)
}
fn expand_no_cow(
    name: Ident,
    inner_attrs: &mut Vec<InnerAttribute>,
    variants: &mut Vec<Variant>,
    partial_eq_lines: &mut Vec<TokenStream>,
    get_key_lines: &mut Vec<TokenStream>,
    enum_name: Path,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        #(#inner_attrs)*
        pub enum #enum_name{
            #(#variants),*
        }
        #[automatically_derived]
        impl enum_helper::KeyEnum for #enum_name{

        }
        #[automatically_derived]
        impl enum_helper::HasKeyEnum for #name{
            type KeyEnum<'a> = #enum_name where Self: 'static;
            fn get_key(&self) -> Self::KeyEnum<'static>{
                match self{
                    #(#get_key_lines),*
                }
            }
             fn get_key_borrowed(&self) -> Self::KeyEnum<'static>{
                 match self{
                    #(#get_key_lines),*
                }
             }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialEq<#enum_name> for #name{
            fn eq(&self, other: &TestEnumKeys) -> bool {
                 match (self, other) {
                    #(#partial_eq_lines),*
                    ,
                    _ => false
                 }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialEq<#name> for #enum_name{
            fn eq(&self, other: &#name) -> bool {
                 match ( other,self) {
                    #(#partial_eq_lines),*
                    ,
                    _ => false
                 }
            }
        }
    }
}
