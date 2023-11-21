use crate::{
    error::{self, AddCause as _, ErrorCause},
    isa::{self, InvalidOpcode},
};

macro_rules! opcode_enum {
    ($(
        $opcode_case:ident $wasm_name:literal $pascal_ident:ident $({ $($_fields:tt)* })? $_snake_ident:ident;
    )*) => {
        /// Represents the full opcode of an instruction.
        ///
        /// Some instructions in WebAssembly are encoded with a byte prefix, followed by the actual
        /// opcode, encoded in a [*LEB128*](crate::values::leb128) format integer.
        #[derive(Clone, Copy, Eq, Hash, PartialEq)]
        #[non_exhaustive]
        #[allow(missing_docs)]
        pub enum Opcode {
            $($pascal_ident,)*
        }

        impl Opcode {
            /// Gets a list of all of the opcodes supported by [`nom-wasm`](crate).
            pub const ALL: &'static [Self] = &[$(Self::$pascal_ident,)*];
        }

        #[cfg(feature = "allow-unsafe")]
        impl Opcode {
            const WASM_NAMES: &'static [*const u8] = &[$($wasm_name.as_ptr(),)*];
            const WASM_NAME_LENS: &'static [u8] = &[$($wasm_name.len() as u8,)*];
            const DEBUG_NAMES: &'static [*const u8] = &[$(stringify!($pascal_ident).as_ptr(),)*];
            const DEBUG_NAME_LENS: &'static [u8] = &[$(stringify!($pascal_ident).len() as u8,)*];
        }

        #[cfg(not(feature = "allow-unsafe"))]
        impl Opcode {
            // Less efficient lookups
            const WASM_NAMES: &'static [&'static str] = &[$($wasm_name,)*];
            const DEBUG_NAMES: &'static [&'static str] = &[$(stringify!($pascal_ident),)*];
        }
    };
}

crate::isa::instr_definitions::all!(opcode_enum);
crate::static_assert::check_size!(Opcode, <= 2);

macro_rules! opcode_partial_eq {
    ($($other:ty,)*) => {$(
        impl PartialEq<$other> for Opcode {
            #[inline]
            fn eq(&self, other: &$other) -> bool {
                self == &other.to_opcode()
            }
        }
    )*};
}

opcode_partial_eq! {
    isa::ByteOpcode,
    isa::FCPrefixedOpcode,
    isa::V128Opcode,
    isa::FEPrefixedOpcode,
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

impl Opcode {
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
            ($($opcode:ty),*) => {
                match prefix {
                    $(
                        <$opcode>::PREFIX => {
                            let missing_opcode = || ErrorCause::Opcode(InvalidOpcode::missing_actual(<$opcode>::PREFIX));
                            let (input, actual) = crate::values::leb128_u32(input).add_cause_with(missing_opcode)?;
                            match <$opcode>::try_from(actual) {
                                Ok(opcode) => Ok((input, Self::from(opcode))),
                                Err(unrecognized) => Err(parse_failed(start, unrecognized)),
                            }
                        }
                    )*
                    _ => match isa::ByteOpcode::try_from(prefix) {
                        Ok(opcode) => Ok((input, Self::from(opcode))),
                        Err(unrecognized) => Err(parse_failed(start, unrecognized)),
                    },
                }
            };
        }

        parse_actual! {
            isa::FCPrefixedOpcode,
            isa::V128Opcode,
            isa::FEPrefixedOpcode
        }
    }

    /// Gets the name of the WebAssembly instruction that this opcode corresponds to, in
    /// the [WebAssembly text format].
    ///
    /// [WebAssembly text format]: https://webassembly.github.io/spec/core/text/instructions.html
    #[inline]
    pub const fn name(self) -> &'static str {
        // Bounds checks should be  eliminated by optimizer

        #[cfg(feature = "allow-unsafe")]
        {
            // Note that `get_unchecked()` can't be used in a `const` context
            let data: *const u8 = Self::WASM_NAMES[self as usize];
            let len = Self::WASM_NAME_LENS[self as usize];

            // Safety: self is a valid index, bytes is valid UTF-8
            unsafe {
                let bytes: &'static [u8] = core::slice::from_raw_parts(data, len as usize);
                return core::str::from_utf8_unchecked(bytes);
            }
        }

        #[cfg(not(feature = "allow-unsafe"))]
        return Self::WASM_NAMES[self as usize];
    }
}

impl core::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let index = *self as usize;
        let name: &'static str;

        #[cfg(feature = "allow-unsafe")]
        {
            // Safety: `self` is a valid index, `bytes` is valid UTF-8
            name = unsafe {
                let data = *Self::DEBUG_NAMES.get_unchecked(index);
                let len = *Self::DEBUG_NAME_LENS.get_unchecked(index);
                let bytes: &'static [u8] = core::slice::from_raw_parts(data, len as usize);
                core::str::from_utf8_unchecked(bytes)
            };
        }

        #[cfg(not(feature = "allow-unsafe"))]
        {
            name = Self::DEBUG_NAMES[index];
        }

        f.debug_tuple(name).finish()
    }
}

impl core::fmt::Display for Opcode {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}
