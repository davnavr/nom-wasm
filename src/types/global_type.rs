use crate::types::ValType;

/// Indicates whether a WebAssembly [**`global`**] is mutable.
///
/// [**`global`**]: https://webassembly.github.io/spec/core/syntax/modules.html#globals
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[allow(clippy::exhaustive_enums)]
pub enum Mutability {
    /// A [**const**] global is one whose value is only assigned once, when the module is
    /// instantiated.
    ///
    /// [**const**]: https://webassembly.github.io/spec/core/syntax/types.html#syntax-mut
    Constant,
    /// A [**var**] global is mutable, and can have a value assigned any time.
    ///
    /// This requires support for the [mutable globals proposal].
    ///
    /// [**var**]: https://webassembly.github.io/spec/core/syntax/types.html#syntax-mut
    /// [mutable globals proposal]: https://github.com/WebAssembly/mutable-global
    Variable,
}

/// Represents a [**`globaltype`**], which indicates the type of value stored in a WebAssembly
/// [**`global`**] and whether it is mutable.
///
/// [**`global`**]: https://webassembly.github.io/spec/core/syntax/modules.html#globals
/// [**`globaltype`**]: https://webassembly.github.io/spec/core/syntax/types.html#global-types
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct GlobalType {
    /// Whether or not the global is mutable.
    pub mutability: Mutability,
    /// The type of the value stored in the global.
    pub value_type: ValType,
}
