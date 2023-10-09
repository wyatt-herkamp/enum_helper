mod enum_of_keys;

#[cfg(feature = "derive")]
pub use enum_helpers_macros::EnumOfKeys;

#[cfg(feature = "derive")]
/// Allows you to compare an enum to a string
pub use enum_helpers_macros::CompareToStr;
pub use enum_of_keys::*;
