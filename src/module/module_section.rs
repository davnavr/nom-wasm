use crate::{module, section::Section};

macro_rules! module_sections {
    ($(
        $(#[$meta:meta])*
        $name:ident($component:ty) = $id:literal $(impl $from:ident)?,
    )+) => {
        /// Represents a well-known WebAssembly module [*section*] or a [`CustomSection`].
        ///
        /// [*section*]: https://webassembly.github.io/spec/core/binary/modules.html#sections
        /// [`CustomSection`]: module::custom::CustomSection
        #[derive(Clone, Debug)]
        #[non_exhaustive]
        pub enum ModuleSection<'a> {$(
            $(#[$meta])*
            $name($component),
        )+}

        $crate::tag::enumeration! {
            /// Represents the [*id*] of a WebAssembly [`ModuleSection`](module::ModuleSection).
            ///
            /// [*id*]: https://webassembly.github.io/spec/core/binary/modules.html#sections
            #[non_exhaustive]
            pub ModuleSectionId : u8 {$(
                $(#[$meta])*
                $name = $id,
            )+}
        }

        impl<'a> ModuleSection<'a> {
            /// Gets the [*id*] for the section.
            ///
            /// [*id*]: https://webassembly.github.io/spec/core/binary/modules.html#sections
            pub fn id(&self) -> ModuleSectionId {
                match self {
                    $(Self::$name(_) => ModuleSectionId::$name,)*
                }
            }

            /// Attempts to interpret the contents of a WebAssembly module [`Section`].
            ///
            /// Returns `Ok(Ok(_))` if the section was a known module section or custom section.
            ///
            /// # Errors
            ///
            /// - Returns `Err(_)` if the [`Section`] is not a known module section or a custom section.
            /// - Returns `Ok(Err(_))` if the section was a known module section or custom
            ///   section, but it could not be parsed.
            pub fn interpret_section<'b, E>(
                section: &'b Section<'a>
            ) -> Result<crate::input::Result<Self, E>, &'b Section<'a>>
            where
                E: crate::error::ErrorSource<'a>,
            {
                todo!("bad {section:?}")
            }
        }

        $($(
            impl<'a> $from<$component> for ModuleSection<'a> {
                #[inline]
                fn from(value: $component) -> Self {
                    Self::$name(value)
                }
            }
        )?)+
    };
}

module_sections! {
    /// A *custom section*.
    ///
    /// Custom sections are ignored by the semantics of WebAssembly, and as such, can appear
    /// anywhere within a module.
    ///
    /// [*custom section*]: https://webassembly.github.io/spec/core/binary/modules.html#binary-customsec
    Custom(module::custom::CustomSection<'a>) = 0 impl From,
    /// The [*type section*].
    ///
    /// [*type section*]: https://webassembly.github.io/spec/core/binary/modules.html#type-section
    Type(module::TypeSec<'a>) = 1 impl From,
    /// The [*import section*].
    ///
    /// [*import section*]: https://webassembly.github.io/spec/core/binary/modules.html#import-section
    Import(module::ImportSec<'a>) = 2 impl From,
}
