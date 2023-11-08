use core::fmt::{Debug, Formatter};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Inner {
    Byte(u8),
    Prefixed { prefix: u8, secondary: u32 },
    Missing,
    MissingActual { prefix: u8 },
}

crate::static_assert::check_size!(Option<InvalidOpcode>, <= 8);

/// Error type used when a WebAssembly instruction opcode is invalid.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct InvalidOpcode(Inner);

impl InvalidOpcode {
    #[inline]
    pub(in crate::isa) const fn new(opcode: u8, secondary: Option<u32>) -> Self {
        Self(if let Some(secondary) = secondary {
            Inner::Prefixed {
                prefix: opcode,
                secondary,
            }
        } else {
            Inner::Byte(opcode)
        })
    }

    #[inline]
    pub(in crate::isa) const fn missing_actual(prefix: u8) -> Self {
        Self(Inner::MissingActual { prefix })
    }

    pub(in crate::isa) const MISSING: Self = Self(Inner::Missing);
}

impl Debug for InvalidOpcode {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl core::fmt::Display for InvalidOpcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            Inner::Byte(opcode) => write!(f, "{opcode:#04X} is not a recognized opcode"),
            Inner::Prefixed { prefix, secondary } => write!(
                f,
                "opcode {secondary} following prefix byte {prefix:#04X} is not a recognized opcode"
            ),
            Inner::Missing => f.write_str("opcode was missing"),
            Inner::MissingActual { prefix } => write!(
                f,
                "missing actual opcode following prefix byte {prefix:#04X}"
            ),
        }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for InvalidOpcode {}
