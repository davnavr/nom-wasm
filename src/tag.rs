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
        $(#[$enum_meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[repr($integer)]
        pub enum $enum_name {$(
            $(#[$case_meta])*
            $case_name = $case_value,
        )*}

        impl $enum_name {
            /// Converts from an integer value, returning `None` if the conversion failed.
            #[inline]
            pub const fn new(tag: $integer) -> Option<Self> {
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
