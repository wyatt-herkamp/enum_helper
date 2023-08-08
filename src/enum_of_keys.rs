use std::borrow::Borrow;

pub trait KeyEnum {}

pub trait HasKeyEnum {
    type KeyEnum<'a>: KeyEnum
    where
        Self: 'a;

    fn get_key(&self) -> Self::KeyEnum<'static>;
    /// On A KeyEnum that does not use Cow to store default this is not borrowed data
    fn get_key_borrowed(&self) -> Self::KeyEnum<'_>;
}
