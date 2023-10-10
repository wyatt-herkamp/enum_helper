mod compare_str;
mod enum_of_keys_impl;
pub(crate) mod utils;

use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};

///
/// Attributes:
/// - `#[enum_of_keys(KeyEnumName)]` - This attribute is required and is used to specify the name of the KeyEnum and any derives that should be added to it
///    - Options:
///         - First Value is the name of the KeyEnum (Required)
///         - `default_in_cow` Will put the default Variant in a Cow
/// - `#[enum_attr(...)]` - This is used to specify any attributes that should be added to the KeyEnum
/// - `#[enum_of_keys(..)]` On a variant to specify any options for the variant.
///    - Options:
///         - `default` - This is used to specify the default variant.
///
/// ```rust, ignore
/// use enum_helpers_macros::EnumOfKeys;
/// #[derive(Debug, Clone, PartialEq, Eq, EnumOfKeys)]
/// #[enum_of_keys(SMTPServerExtensionKey)]  // Anything within the derive attribute will be added to the KeyEnum
/// #[enum_attr(derive(Clone, Copy, Debug, PartialEq, Eq, strum::AsRefStr, strum::IntoStaticStr, strum::EnumIs,strum::EnumString,strum::Display, strum::EnumIter))]
/// #[enum_attr(strum(serialize_all = "UPPERCASE"))]
/// #[non_exhaustive] // Non exhaustive is automatically added to the KeyEnum if it is on the main enum
/// pub enum SMTPServerExtension {
///     Size(u64),
///     StartTLS,
///     Auth(Vec<String>),
///     #[enum_of_keys(default)]
///     #[enum_attr(strum(default))]
///     Other(String),
/// }
/// ```
/// Would generate the following code:
/// ```rust, ignore
/// extern crate strum;
/// #[derive(Clone, Copy, Debug, PartialEq, Eq, strum::AsRefStr, strum::IntoStaticStr, strum::EnumIs,strum::EnumString,strum::Display, strum::EnumIter)]
/// #[strum(serialize_all = "UPPERCASE")]
/// pub enum SMTPServerExtensionKey {
///     Size,
///     StartTLS,
///     Auth,
///     #[strum(default)]
///     Other(String),
/// }
/// ```
/// Generates the following. Trait Impls Too
///  - [HasEnumKey](enum_of_keys_impl::HasEnumKey)
///  - [PartialEq] for KeyEnum and OriginalEnum
#[proc_macro_derive(EnumOfKeys, attributes(enum_of_keys, enum_attr, compare_str))]
pub fn enum_of_keys(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Check if its an enum
    if let syn::Data::Enum(_) = &input.data {
        match enum_of_keys_impl::expand(input) {
            Ok(ok) => ok.into(),
            Err(err) => err.to_compile_error().into(),
        }
    } else {
        syn::Error::new_spanned(input, "EnumOfKeys can only be derived for enums")
            .to_compile_error()
            .into()
    }
}

#[proc_macro_derive(CompareToStr, attributes(compare_str))]
pub fn compare_to_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match compare_str::expand(input) {
        Ok(ok) => ok.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
