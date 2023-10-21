macro_rules! enumeration_basic {
    (
        $(#[$enum_meta:meta])*
        pub $enum_name:ident : $integer:ty {
            $(
                $(#[$case_meta:meta])*
                $case_name:ident = $case_value:literal,
            )*
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Eq, Hash, PartialEq)]
        pub enum $enum_name {$(
            $(#[$case_meta])*
            $case_name = $case_value,
        )*}

        // Needed for perfect hashing, TODO: Debug lookup table
        crate::static_assert::check_size!($enum_name, <= core::mem::size_of::<usize>());

        impl $enum_name {
            /// Converts from an integer value, returning `None` if the conversion failed.
            pub const fn new(tag: $integer) -> Option<Self> {
                // Usually optimizes into a single comparison in assembly
                match tag {
                    $($case_value => Some(Self::$case_name),)*
                    _ => None,
                }
            }
        }

        impl From<$enum_name> for $integer {
            #[inline]
            fn from(tag: $enum_name) -> $integer {
                tag as $integer
            }
        }

        impl core::fmt::Debug for $enum_name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                #[derive(Clone, Copy)]
                #[repr(usize)]
                enum Key {
                    $($case_name,)*
                }

                const DEBUG_NAMES: &[*const u8] = &[$(stringify!($case_name).as_ptr()),*];
                const DEBUG_NAME_LENS: &[u8] = &[$(stringify!($case_name).len() as u8),*];

                let key = match self {
                    $(Self::$case_name => Key::$case_name,)*
                };

                // Safety: key is always in bounds, pointer is to valid UTF-8
                let name: &'static str = unsafe {
                    let ptr: *const u8 = *DEBUG_NAMES.get_unchecked(key as usize);
                    let len: u8 = *DEBUG_NAME_LENS.get_unchecked(key as usize);
                    let bytes: &'static [u8] = core::slice::from_raw_parts(ptr, len as usize);
                    core::str::from_utf8_unchecked(bytes)
                };

                f.debug_tuple(name).finish()
            }
        }
    };
}

macro_rules! enumeration {
    (
        $(#[$enum_meta:meta])*
        pub $enum_name:ident : $integer:ty {
            $(
                $(#[$case_meta:meta])*
                $case_name:ident = $case_value:literal,
            )*
        }
    ) => {
        $crate::tag::enumeration_basic! {
            $(#[$enum_meta])*
            pub $enum_name : $integer {
                $(
                    $(#[$case_meta])*
                    $case_name = $case_value,
                )*
            }
        }

        impl TryFrom<$integer> for $enum_name {
            type Error = $crate::error::InvalidTag;

            #[inline]
            fn try_from(tag: $integer) -> Result<Self, Self::Error> {
                if let Some(ok) = Self::new(tag) {
                    Ok(ok)
                } else {
                    Err(<Self::Error>::$enum_name(tag.into()))
                }
            }
        }
    };
}

pub(crate) use enumeration;
pub(crate) use enumeration_basic;
