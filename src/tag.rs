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

        impl $enum_name {
            /// Converts from an integer value, returning `None` if the conversion failed.
            pub const fn new(tag: $integer) -> Option<Self> {
                // Usually optimizes into a single comparison in assembly
                match tag {
                    $($case_value => Some(Self::$case_name),)*
                    _ => None,
                }
            }

            #[doc = "Contains all `"]
            #[doc = stringify!($enum_name)]
            #[doc = "` variants."]
            pub const ALL: &[Self] = &[
                $(Self::$case_name,)*
            ];
        }

        impl From<$enum_name> for $integer {
            #[inline]
            fn from(tag: $enum_name) -> $integer {
                tag as $integer
            }
        }

        impl core::fmt::Debug for $enum_name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                const DEBUG_NAMES: &[&str] = &[
                    $(stringify!($case_name),)*
                ];

                f.debug_tuple(DEBUG_NAMES[*self as usize]).finish()
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
