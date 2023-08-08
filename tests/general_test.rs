use enum_helper::LookupByKey;
use enum_helpers_macros::EnumOfKeys;
use strum::EnumIter;
use strum::IntoEnumIterator;
#[derive(EnumOfKeys, EnumIter, Debug)]
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
#[derive(EnumOfKeys)]
#[enum_of_keys(TestEnumNewStructDefaultKeys)]
pub enum TestEnumNewStructDefault {
    A(String),
    #[enum_of_keys(default = a)]
    Default {
        a: String,
        b: u32,
    },
}
#[derive(EnumOfKeys)]
#[enum_of_keys(TestEnumNewMultiTupleDefaultKeys)]
pub enum TestEnumNewMultiTupleDefault {
    A(String),
    #[enum_of_keys(default = value)]
    Default(String, String),
}

#[test]
pub fn test() {
    let vec = TestEnum::iter().collect::<Vec<_>>();
    for i in TestEnumKeys::iter() {
        let option = vec.get_by_key(&i);
        assert!(option.is_some());
        println!("{:?}", option.unwrap());
    }
}
