use crate::{
    error::{self, AddCause as _, ErrorCause},
    isa::{FCPrefixedOpcode, FEPrefixedOpcode, InvalidOpcode, Opcode, V128Opcode},
};

macro_rules! instr_kind {
    ($(
        $(#[$case_meta:meta])*
        $case:ident($case_value:ty),
    )*) => {
        /// Represents the full opcode of an instruction.
        ///
        /// Some instructions in WebAssembly are encoded with a byte prefix, followed by the actual
        /// opcode, encoded in a [*LEB128*](crate::values::leb128) format integer.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        pub enum InstrKind {$(
            $(#[$case_meta])*
            $case($case_value),
        )*}

        $(
            impl From<$case_value> for InstrKind {
                #[inline]
                fn from(opcode: $case_value) -> Self {
                    Self::$case(opcode)
                }
            }

            impl PartialEq<$case_value> for InstrKind {
                #[inline]
                fn eq(&self, other: &$case_value) -> bool {
                    matches!(self, Self::$case(me) if me == other)
                }
            }
        )*
    };
}

instr_kind! {
    /// Encodes an instruction with a single byte *opcode*.
    Byte(Opcode),
    /// Encodes an instruction prefixed with the byte `0xFC`.
    FCPrefixed(FCPrefixedOpcode),
    /// Encodes a 128-bit vector instruction, which is prefixed with the byte `0xFD`.
    ///
    /// This prefix is used for 128-bit vector instructions (`v128.*`, `i8x16.*`, etc.).
    V128(V128Opcode),
    /// Encodes an instruction prefixed with the byte `0xFE`.
    ///
    /// This prefix is used for atomic memory instructions (`memory.atomic.*` and `*.atomic.*`).
    FEPrefixed(FEPrefixedOpcode),
}

fn parse_failed<'a, E>(input: &'a [u8], error: InvalidOpcode) -> nom::Err<E>
where
    E: error::ErrorSource<'a>,
{
    nom::Err::Failure(E::from_error_kind_and_cause(
        input,
        error::ErrorKind::Tag,
        ErrorCause::Opcode(error),
    ))
}

impl InstrKind {
    /// Parses a WebAssembly instruction *opcode*.
    ///
    /// # Error
    ///
    /// Returns an error if the *opcode* bytes could not be parsed, or if the *opcode* was not
    /// recognized.
    pub fn parse<'a, E>(input: &'a [u8]) -> crate::Parsed<'a, Self, E>
    where
        E: error::ErrorSource<'a>,
    {
        let start = input;
        let (input, prefix) = if let Some((prefix, remaining)) = input.split_first() {
            (remaining, *prefix)
        } else {
            return Err(parse_failed(start, InvalidOpcode::MISSING));
        };

        macro_rules! parse_actual {
            ($(
                $opcode:ty => $case:ident,
            )*) => {
                match prefix {
                    $(
                        <$opcode>::PREFIX => {
                            let missing_opcode = || ErrorCause::Opcode(InvalidOpcode::missing_actual(<$opcode>::PREFIX));
                            let (input, actual) = crate::values::leb128_u32(input).add_cause_with(missing_opcode)?;
                            match <$opcode>::try_from(actual) {
                                Ok(opcode) => Ok((input, Self::$case(opcode))),
                                Err(unrecognized) => Err(parse_failed(start, unrecognized)),
                            }
                        }
                    )*
                    _ => match Opcode::try_from(prefix) {
                        Ok(opcode) => Ok((input, Self::Byte(opcode))),
                        Err(unrecognized) => Err(parse_failed(start, unrecognized)),
                    },
                }
            };
        }

        parse_actual! {
            FCPrefixedOpcode => FCPrefixed,
            V128Opcode => V128,
            FEPrefixedOpcode => FEPrefixed,
        }
    }
}

macro_rules! instr_kind_name_case {
    (Byte $pascal_ident:ident) => {
        InstrKind::Byte(Opcode::$pascal_ident)
    };
    (FCPrefixed $pascal_ident:ident) => {
        InstrKind::FCPrefixed(FCPrefixedOpcode::$pascal_ident)
    };
    (V128 $pascal_ident:ident) => {
        InstrKind::V128(V128Opcode::$pascal_ident)
    };
    (FEPrefixed $pascal_ident:ident) => {
        InstrKind::FEPrefixed(FEPrefixedOpcode::$pascal_ident)
    };
}

macro_rules! instr_kind_name {
    ($(
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $snake_ident:ident;
    )*) => {
        impl InstrKind {
            /// Gets the name of the WebAssembly instruction that this opcode corresponds to, in
            /// the [WebAssembly text format].
            ///
            /// [WebAssembly text format]: https://webassembly.github.io/spec/core/text/instructions.html
            pub fn name(&self) -> &'static str {
                match self {
                    $(instr_kind_name_case!($opcode_case $pascal_ident) => $wasm_name,)*
                }
            }
        }
    };
}

crate::isa::instr_definitions::all!(instr_kind_name);

impl core::fmt::Display for InstrKind {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}
