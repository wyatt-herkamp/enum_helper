macro_rules! into_enum {
    ($i:ident,$t:ident, $m:literal) => {
        match $i {
            syn::Data::Enum(data_enum) => data_enum,
            _ => {
                return Err(Error::new(
                    $t.span(),
                    concat!($m, " can only be used with enums"),
                ))
            }
        }
    };
}
pub(crate) use into_enum;
