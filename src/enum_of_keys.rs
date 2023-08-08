pub trait KeyEnum {}
impl<T: KeyEnum> KeyEnum for &'_ T {}
pub trait HasKeyEnum {
    type KeyEnum<'a>: KeyEnum
    where
        Self: 'a;

    fn get_key(&self) -> Self::KeyEnum<'static>;
    /// On A KeyEnum that does not use Cow to store default this is not borrowed data
    fn get_key_borrowed(&self) -> Self::KeyEnum<'_>;
}

impl<'b, T> HasKeyEnum for &'b T
where
    T: HasKeyEnum,
{
    type KeyEnum<'a> =  <T as HasKeyEnum>::KeyEnum<'a> where T: 'a, Self: 'a;

    fn get_key(&self) -> T::KeyEnum<'static> {
        (*self).get_key()
    }

    fn get_key_borrowed(&self) -> Self::KeyEnum<'_> {
        (*self).get_key_borrowed()
    }
}
pub trait LookupByKey<'a> {
    type HasKeyEnum: HasKeyEnum;
    fn has_key(self, key: &'a <Self::HasKeyEnum as HasKeyEnum>::KeyEnum<'a>) -> bool;

    fn get_by_key(
        self,
        key: &'a <Self::HasKeyEnum as HasKeyEnum>::KeyEnum<'a>,
    ) -> Option<&'a Self::HasKeyEnum>;

    fn get_all_by_key(
        self,
        key: &'a <Self::HasKeyEnum as HasKeyEnum>::KeyEnum<'a>,
    ) -> Vec<&'a Self::HasKeyEnum>;
}
impl<'a, I, E> LookupByKey<'a> for I
where
    I: IntoIterator<Item = &'a E> + 'a,
    E: HasKeyEnum + PartialEq<E::KeyEnum<'a>> + 'a,
    &'a E: PartialEq<<E as HasKeyEnum>::KeyEnum<'a>>,
{
    type HasKeyEnum = E;

    fn has_key(self, key: &'a <Self::HasKeyEnum as HasKeyEnum>::KeyEnum<'a>) -> bool {
        self.into_iter().any(|e| e.eq(key))
    }

    fn get_by_key(
        self,
        key: &'a <Self::HasKeyEnum as HasKeyEnum>::KeyEnum<'a>,
    ) -> Option<&'a Self::HasKeyEnum> {
        self.into_iter().find(|e| e.eq(key))
    }

    fn get_all_by_key(
        self,
        key: &'a <Self::HasKeyEnum as HasKeyEnum>::KeyEnum<'a>,
    ) -> Vec<&'a Self::HasKeyEnum> {
        self.into_iter().filter(|e| e.eq(key)).collect()
    }
}
