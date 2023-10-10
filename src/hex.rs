#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct Hex(pub(crate) u8);

impl core::fmt::Debug for Hex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02X}", self.0)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct Bytes<'a>(pub(crate) &'a [u8]);

impl core::fmt::Debug for Bytes<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|b| Hex(*b)))
            .finish()
    }
}
