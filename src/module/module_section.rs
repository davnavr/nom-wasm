use crate::module;

macro_rules! module_sections {
    ($(
        $(#[$meta:meta])*
        $name:ident($component:ty) = $id:literal $(impl $from:ident)?,
    )+) => {
        /// Represents a well-known WebAssembly [`Module`] [*section*] or a [`CustomSection`].
        ///
        /// [`Module`]: module::Module
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
        }
    };
}

module_sections! {
    /// A *custom section*.
    ///
    /// Custom sections are ignored by the semantics of WebAssembly, and as such, can appear
    /// anywhere within a module.
    ///
    /// [*custom section*]: https://webassembly.github.io/spec/core/binary/modules.html#binary-customsec
    Custom(module::custom::CustomSection<'a>) = 0,
    /// The [*type section*].
    ///
    /// [*type section*]: https://webassembly.github.io/spec/core/binary/modules.html#type-section
    Type(module::TypeSec<'a>) = 1 impl From,
}
