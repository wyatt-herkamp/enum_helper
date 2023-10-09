mod enum_of_keys;

#[cfg(feature = "derive")]
pub use enum_helpers_macros::EnumOfKeys;

#[cfg(feature = "derive")]
/// Allows you to compare an enum to a string
/// # Available Container Attributes
/// - partial_eq: bool.
/// Defaults to true.
/// If true will implement PartialEq<str> and PartialEq<&str> for the enum
/// - to_lowercase: bool.
/// Defaults to false.
/// If true will lowercase the string before comparing
/// # Available Field Attributes
/// - equals: [&str]. An Array of strings to compare to
/// - contains: [&str].
/// An Array of strings to check if the string contains
/// # Example
/// ```rust,ignore
/// use enum_helpers_macros::CompareToStr;
/// #[derive(Debug, Clone, PartialEq, Eq, CompareToStr)]
/// #[compare_str(partial_eq = false, to_lowercase)]
/// pub enum OtherTypesNoPartialEq {
///     #[compare_str(equals["uuid::Uuid", "Uuid"])]
///     UUID,
///     #[compare_str(equals["PathBuf", "Path"])]
///     FilePath,
///     #[compare_str(equals["TupleThing"])]
///     Tuple(String, String),
///     #[compare_str(equals["TupleThing2"])]
///     Tuple2(String),
/// }
/// ```
pub use enum_helpers_macros::CompareToStr;
pub use enum_of_keys::*;
