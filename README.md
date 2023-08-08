# Enum Helpers


### Enum Of Keys
Generates a Second Enum of just the variant names of the first Enum
```rust
#[derive(EnumOfKeys)]
#[enum_of_keys(TestEnumKeys)]
#[enum_attr(derive(Debug, Clone, PartialEq, Eq))]
#[non_exhaustive]
pub enum TestEnum {
    A(String),
    B,
    C{
        a: String,
        b: u32,
    },
    #[enum_of_keys(default)]
    Default(String),
}
```
Will generate
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TestEnumKeys {
    A,
    B,
    C,
    Default(String),
}
impl enum_helper::KeyEnum for TestEnumKeys {}
impl enum_helper::HasKeyEnum for TestEnum {
    type KeyEnum<'a> = TestEnumKeys where Self: 'static;
    fn get_key(&self) -> Self::KeyEnum<'static> {
        match self {
            TestEnum::A(..) => TestEnumKeys::A,
            TestEnum::B => TestEnumKeys::B,
            TestEnum::C { .. } => TestEnumKeys::C,
            TestEnum::Default(value) => TestEnumKeys::Default(value.clone())
        }
    }
    fn get_key_borrowed(&self) -> Self::KeyEnum<'static> {
        match self {
            TestEnum::A(..) => TestEnumKeys::A,
            TestEnum::B => TestEnumKeys::B,
            TestEnum::C { .. } => TestEnumKeys::C,
            TestEnum::Default(value) => TestEnumKeys::Default(value.clone())
        }
    }
}
impl ::core::cmp::PartialEq<TestEnumKeys> for TestEnum {
    fn eq(&self, other: &TestEnumKeys) -> bool {
        match (self, other) {
            (TestEnum::A(..), TestEnumKeys::A) => true,
            (TestEnum::B, TestEnumKeys::B) => true,
            (TestEnum::C { .. }, TestEnumKeys::C) => true,
            (TestEnum::Default(a), TestEnumKeys::Default(b)) => a == b,
            _ => false
        }
    }
}
impl ::core::cmp::PartialEq<TestEnum> for TestEnumKeys {
    fn eq(&self, other: &TestEnum) -> bool {
        match (other, self) {
            (TestEnum::A(..), TestEnumKeys::A) => true,
            (TestEnum::B, TestEnumKeys::B) => true,
            (TestEnum::C { .. }, TestEnumKeys::C) => true,
            (TestEnum::Default(a), TestEnumKeys::Default(b)) => a == b,
            _ => false
        }
    }
}
```