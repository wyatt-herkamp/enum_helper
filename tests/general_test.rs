use enum_helpers_macros::EnumOfKeys;
use std::borrow::{Borrow, Cow};

#[derive(EnumOfKeys)]
#[enum_of_keys(TestEnumKeys)]
#[enum_attr(derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
    strum::EnumIter
))]
#[enum_attr(strum(serialize_all = "UPPERCASE"))]
#[non_exhaustive]
pub enum TestEnum {
    A(String),
    B,
    C {
        a: String,
        b: u32,
    },
    #[enum_of_keys(default)]
    #[enum_attr(strum(default))]
    Default(String),
}

#[derive(EnumOfKeys)]
#[enum_of_keys(TestEnumCowKeys, default_in_cow)]
pub enum TestEnumCow {
    A(String),
    #[enum_of_keys(default)]
    Default(String),
}

#[test]
pub fn test() {}
