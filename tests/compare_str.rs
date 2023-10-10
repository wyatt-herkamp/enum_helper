use enum_helpers_macros::{CompareToStr, EnumOfKeys};
use strum::EnumString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UUIDFormat {
    Bytes,
    HyphenatedString,
    UnhyphenatedString,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UUIDVersion {
    Version1,
    Version2,
    Version3,
    Version4,
    Version5,
}
/// Time Types
#[derive(Debug, Clone, PartialEq, Eq, CompareToStr, EnumOfKeys)]
#[enum_of_keys(OtherTypesKeys, impl_common_traits, impl_strum)]
pub enum OtherTypes {
    #[compare_str(equals["uuid::Uuid", "Uuid"], contains["uuid"])]
    UUID {
        format: UUIDFormat,
        version: UUIDVersion,
    },
    #[compare_str(equals["PathBuf", "Path"])]
    FilePath,
    #[compare_str(equals["TupleThing"])]
    Tuple(String, String),
    #[compare_str(equals["TupleThing2"])]
    Tuple2(String),
}
#[derive(Debug, Clone, PartialEq, Eq, CompareToStr)]
#[compare_str(partial_eq = false, to_lowercase)]
pub enum OtherTypesNoPartialEq {
    #[compare_str(equals["uuid::Uuid", "Uuid"])]
    UUID {
        format: UUIDFormat,
        version: UUIDVersion,
    },
    #[compare_str(equals["PathBuf", "Path"])]
    FilePath,
    #[compare_str(equals["TupleThing"])]
    Tuple(String, String),
    #[compare_str(equals["TupleThing2"])]
    Tuple2(String),
}

#[test]
pub fn test() {
    assert_eq!(
        OtherTypes::UUID {
            format: UUIDFormat::Bytes,
            version: UUIDVersion::Version1,
        },
        "uuid::Uuid"
    );
    assert_eq!(
        OtherTypes::UUID {
            format: UUIDFormat::Bytes,
            version: UUIDVersion::Version1,
        },
        "Uuid"
    );
    assert_eq!(OtherTypes::FilePath, "PathBuf");
    assert_eq!(OtherTypes::FilePath, "Path");
}
