use core::fmt::{Debug, Formatter};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Inner {
    Byte(u8),
    Prefixed { opcode: u8, secondary: u32 },
    Missing,
}

const _SIZE_CHECK: () = if core::mem::size_of::<InvalidOpcode>() > 8 {
    panic!("size_of<InvalidOpcode> must be at most 8 bytes")
};

/// Error type used when a WebAssembly instruction opcode is invalid.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct InvalidOpcode(Inner);

impl InvalidOpcode {
    #[inline]
    pub(in crate::isa) const fn new(opcode: u8, secondary: Option<u32>) -> Self {
        Self(if let Some(secondary) = secondary {
            Inner::Prefixed { opcode, secondary }
        } else {
            Inner::Byte(opcode)
        })
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
            Inner::Prefixed { opcode, secondary } => write!(
                f,
                "opcode {secondary} following prefix {opcode:#04X} is not a recognized opcode"
            ),
            Inner::Missing => f.write_str("opcode was missing"),
        }
    }
}

#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl std::error::Error for InvalidOpcode {}
