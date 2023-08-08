mod attrs;
mod variant;

use crate::enum_of_keys_impl::attrs::{EnumOfKeysAttribute, InnerAttribute};
use crate::enum_of_keys_impl::variant::Variant;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{Attribute, Path, Result};
use syn::{DeriveInput, Error};

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
    let mut get_key_lines = Vec::with_capacity(data_enum.variants.len());
    let mut get_key_lines_owned = Vec::with_capacity(data_enum.variants.len());

    for variant in data_enum.variants {
        let variant = Variant::new(variant, enum_attributes.store_default_in_cow)?;

        get_key_lines.push(variant.create_get_key_line(&name, &enum_attributes.name));
        if enum_attributes.store_default_in_cow {
            get_key_lines_owned
                .push(variant.create_get_key_line_owned(&name, &enum_attributes.name))
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
            inner_attrs,
            variants,
            get_key_lines,
            enum_name,
            get_key_lines_owned,
        )?
    } else {
        expand_no_cow(name, inner_attrs, variants, get_key_lines, enum_name)
    };

    Ok(result)
}
fn expand_inner(
    enum_name: TokenStream,
    og_enum: &Ident,
    partial_eq_lines: Vec<TokenStream>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl enum_helper::KeyEnum for #enum_name{ }
        #[automatically_derived]
        impl ::core::cmp::PartialEq<#enum_name> for #og_enum{
            fn eq(&self, other: &#enum_name) -> bool {
                 match (self, other) {
                    #(#partial_eq_lines),*
                    ,
                    _ => false
                 }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialEq<#enum_name> for &'_ #og_enum{
            fn eq(&self, other: &#enum_name) -> bool {
                 match (self, other) {
                    #(#partial_eq_lines),*
                    ,
                    _ => false
                 }
            }
        }

        #[automatically_derived]
        impl ::core::cmp::PartialEq<#og_enum> for #enum_name{
            fn eq(&self, other: &#og_enum) -> bool {
                 match ( other,self) {
                    #(#partial_eq_lines),*
                    ,
                    _ => false
                 }
            }
        }

    }
}
fn expand_cow(
    name: Ident,
    inner_attrs: Vec<InnerAttribute>,
    variants: Vec<Variant>,
    get_key_lines: Vec<TokenStream>,
    enum_name: Path,
    get_key_owned_lines: Vec<TokenStream>,
) -> Result<TokenStream> {
    let mut to_owned_catches = Vec::new();
    for variant in variants.iter() {
        let variant_name = &variant.name;
        if variant
            .enum_of_keys_attr
            .as_ref()
            .and_then(|v| v.default.as_ref())
            .is_some()
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
    let mut result = quote! {
        #[automatically_derived]
        #(#inner_attrs)*
        pub enum #enum_name<'a>{
            #(#variants),*
        }

        #[automatically_derived]
        impl #enum_name<'_> {
            /// Creates a new copy of the Enum.
            ///
            /// For the Default variant it will create a new owned copy of the default value.
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
    };
    result.append_all(expand_inner(
        quote! { #enum_name<'_> },
        &name,
        variants
            .iter()
            .map(|v| v.create_partial_eq_line(&name, &enum_name))
            .collect::<Vec<_>>(),
    ));
    Ok(result)
}
fn expand_no_cow(
    name: Ident,
    inner_attrs: Vec<InnerAttribute>,
    variants: Vec<Variant>,
    get_key_lines: Vec<TokenStream>,
    enum_name: Path,
) -> TokenStream {
    let mut result = quote! {
        #[automatically_derived]
        #(#inner_attrs)*
        pub enum #enum_name{
            #(#variants),*
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
    };
    result.append_all(expand_inner(
        quote! { #enum_name },
        &name,
        variants
            .iter()
            .map(|v| v.create_partial_eq_line(&name, &enum_name))
            .collect::<Vec<_>>(),
    ));
    result
}
