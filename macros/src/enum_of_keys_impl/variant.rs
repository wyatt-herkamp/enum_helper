use crate::enum_of_keys_impl::attrs::{InnerAttribute, VariantAttribute};
use crate::enum_of_keys_impl::find_and_parse_inner_attrs;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::Error;
use syn::{Fields, Path, Result};

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
        let mut attributes: Option<VariantAttribute> = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("enum_of_keys"))
            .map(|v| v.parse_args())
            .transpose()?;
        let inner_attrs = find_and_parse_inner_attrs(&variant.attrs)?;
        if let Some( default_value) = attributes.as_mut().and_then(|v| v.default.as_mut()) {
            match &variant.fields {
                Fields::Named(named) => {
                    if named.named.len() == 1 {
                        let default_value_ident = &default_value.key_name.clone();
                        default_value.unwrap_variant = Some(quote! {
                            {#default_value_ident}
                        })
                    } else {
                        let default_value_ident = &default_value.key_name.clone();
                        default_value.unwrap_variant = Some(quote! {
                            {#default_value_ident, ..}
                        })
                    }
                }
                Fields::Unnamed(value) => {
                    if value.unnamed.len() != 1 {
                        let default_value_ident = &default_value.key_name.clone();
                        default_value.unwrap_variant = Some(quote!(
                            (#default_value_ident, ..)
                        ))
                    }else{

                        let default_value_ident = &default_value.key_name.clone();
                        default_value.unwrap_variant = Some(quote!(
                            (#default_value_ident)
                        ))
                    }
                }
                Fields::Unit => {
                    return Err(Error::new(
                        variant.span(),
                        "The default value can not be a unit variant",
                    ));
                }
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

    pub fn create_get_key_line_owned(
        &self,
        enum_name: &Ident,
        key_enum_name: &Path,
    ) -> TokenStream {
        let Self {
            name,
            fields_collection,
            enum_of_keys_attr,
            ..
        } = self;
        if let Some(default_value) = enum_of_keys_attr.as_ref().and_then(|v| v.default.as_ref()) {
            let unwrap_variant = &default_value
                .unwrap_variant
                .as_ref()
                .expect("unwrap_variant");
            let key_name = &default_value.key_name;
            quote! {
                #enum_name::#name #unwrap_variant  => #key_enum_name::#name(::std::borrow::Cow::Owned(#key_name.to_owned()))
            }
        } else {
            quote! {
                #enum_name::#name #fields_collection => #key_enum_name::#name
            }
        }
    }
    pub fn create_get_key_line(&self, enum_name: &Ident, key_enum_name: &Path) -> TokenStream {
        let Self {
            name,
            fields_collection,
            enum_of_keys_attr,
            ..
        } = self;
        if let Some(default_value) = enum_of_keys_attr.as_ref().and_then(|v| v.default.as_ref()) {
            let unwrap_variant = &default_value
                .unwrap_variant
                .as_ref()
                .expect("unwrap_variant");
            let key_name = &default_value.key_name;
            if self.default_in_cow {
                quote! {
                    #enum_name::#name #unwrap_variant => #key_enum_name::#name(::std::borrow::Cow::Borrowed(&#key_name))
                }
            } else {
                quote! {
                    #enum_name::#name #unwrap_variant => #key_enum_name::#name(#key_name.clone())
                }
            }
        } else {
            quote! {
                #enum_name::#name #fields_collection => #key_enum_name::#name
            }
        }
    }

    pub fn create_partial_eq_line(&self, enum_name: &Ident, key_enum_name: &Path) -> TokenStream {
        let Self {
            name,
            enum_of_keys_attr,
            fields_collection,
            ..
        } = self;
        if let Some(default_value) = enum_of_keys_attr.as_ref().and_then(|v| v.default.as_ref()) {
            let unwrap_variant = &default_value
                .unwrap_variant
                .as_ref()
                .expect("unwrap_variant");
            let key_name = &default_value.key_name;
            quote! {
                (#enum_name::#name #unwrap_variant, #key_enum_name::#name(b)) => #key_name == b
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
        if enum_of_keys_attr
            .as_ref()
            .and_then(|v| v.default.as_ref())
            .is_some()
        {
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
        let value = quote! {
            #(#inner_attrs)*
            #name
        };
        tokens.append_all(value);
    }
}
