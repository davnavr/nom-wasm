/// Represents a `v128` WebAssembly value.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct V128(pub [u8; 16]);

impl V128 {
    #[allow(missing_docs)]
    pub fn parse<'a, E>(input: &'a [u8]) -> crate::Parsed<'a, Self, E>
    where
        E: crate::error::ErrorSource<'a>,
    {
        if let Some(bytes) = input.get(..16) {
            Ok((&input[16..], Self(bytes.try_into().unwrap())))
        } else {
            Err(nom::Err::Failure(E::from_error_kind(
                input,
                nom::error::ErrorKind::Eof,
            )))
        }
    }
}

impl core::fmt::Debug for V128 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#034X}", u128::from_le_bytes(self.0))
    }
}
