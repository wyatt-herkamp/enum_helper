use std::str::FromStr;
use enum_helpers_macros::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

#[derive(Display, AsRefStr,EnumString, Serialize, Deserialize)]
pub enum TestOne{

    A, B,
    #[strum(default)]
    C(String)

}
#[derive(Display, AsRefStr,EnumString, Serialize, Deserialize)]
#[serde(as_ref, try_from_string)]
pub enum TestTwo{
    A, B, C(String)

}
impl TryFrom<String> for TestTwo{
    type Error = strum::ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        TestTwo::from_str(&value)
    }
}