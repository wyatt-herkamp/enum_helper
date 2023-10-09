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

macro_rules! consume_comma {
    (
        $input:ident
    ) => {
        let _ = $input.parse::<syn::Token![,]>();
    };
}
pub(crate) use consume_comma;
