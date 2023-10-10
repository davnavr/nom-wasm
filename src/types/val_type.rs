use crate::types::{self, TypeIdx};
use core::fmt::{Display, Formatter};

/// Represents a [WebAssembly number type].
///
/// [WebAssembly number type]: https://webassembly.github.io/spec/core/syntax/types.html#number-types
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum NumType {
    /// The 32-bit integer, **`i32`**.
    ///
    /// Under the [Basic C ABI], **`i32`** corresponds to the `int`, `signed int`, `unsigned int`,
    /// `long`, `signed long`, `unsigned long`, `size_t`, `enum`, and pointer types in C.
    ///
    /// [Basic C ABI]: https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md
    I32,
    /// The 64-bit integer, **`i64`**.
    ///
    /// Under the [Basic C ABI], **`i64`** corresponds to the `long long`, `signed long long`, and
    /// `unsigned long long` types in C.
    ///
    /// [Basic C ABI]: https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md
    I64,
    /// 32-bit IEEE-754 floating point (**`f32`**), sometimes referred to as `float`.
    F32,
    /// 64-bit IEEE-754 floating point (**`f64`**), sometimes referred to as `double`.
    F64,
}

/// Represents a [WebAssembly vector type].
///
/// [WebAssembly vector type]: https://webassembly.github.io/spec/core/syntax/types.html#vector-types
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum VecType {
    /// A 128-bit vector, introduced as part of the [fixed-width SIMD proposal].
    ///
    /// [fixed-width SIMD proposal]: https://github.com/webassembly/simd
    V128,
}

/// Represents a [WebAssembly reference type].
///
/// [WebAssembly reference type]: https://webassembly.github.io/spec/core/syntax/types.html#reference-types
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RefType {
    /// A **`funcref`**, a reference to a function.
    ///
    /// This type was originally known as **`anyfunc`** in the 2017 WebAssembly MVP.
    Func,
    /// An **`externref`**, an opaque reference to some object provided by the WebAssembly embedder.
    ///
    /// Introduced as part of the [reference types proposal].
    ///
    /// [reference types proposal]: https://github.com/WebAssembly/reference-types
    Extern,
}

/// Represents a [WebAssembly value type], which indicate the types of values.
///
/// [WebAssembly value type]: https://webassembly.github.io/spec/core/syntax/types.html#value-types
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ValType {
    /// The [**`i32`**](NumType::I32) numeric type.
    I32,
    /// The [**`i64`**](NumType::I64) numeric type.
    I64,
    /// The [**`f32`**](NumType::F32) numeric type.
    F32,
    /// The [**`f64`**](NumType::F64) numeric type.
    F64,
    /// The [**`funcref`**](RefType::Func) type.
    FuncRef,
    /// The [**`externref`**](RefType::Extern) type.
    ExternRef,
    /// The [**`v128`**](VecType::V128) type.
    V128,
}

/// Represents a [**`blocktype`**] which describes the types of the inputs and results of a [block].
///
/// [**`blocktype`**]: https://webassembly.github.io/spec/core/binary/instructions.html#binary-blocktype
/// [block]: https://webassembly.github.io/spec/core/binary/instructions.html#control-instructions
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BlockType {
    /// Indicates a block has no outputs.
    #[default]
    Empty,
    /// References a function type that describes the inputs and results for this block.
    Index(TypeIdx),
    /// A type describing the single output of a block.
    Inline(ValType),
}

impl From<NumType> for ValType {
    fn from(ty: NumType) -> Self {
        match ty {
            NumType::I32 => Self::I32,
            NumType::I64 => Self::I64,
            NumType::F32 => Self::F32,
            NumType::F64 => Self::F64,
        }
    }
}

impl From<RefType> for ValType {
    fn from(ty: RefType) -> Self {
        match ty {
            RefType::Extern => Self::ExternRef,
            RefType::Func => Self::FuncRef,
        }
    }
}

impl From<VecType> for ValType {
    fn from(ty: VecType) -> Self {
        match ty {
            VecType::V128 => Self::V128,
        }
    }
}

impl From<TypeIdx> for BlockType {
    fn from(index: TypeIdx) -> Self {
        Self::Index(index)
    }
}

impl From<ValType> for BlockType {
    fn from(ty: ValType) -> Self {
        Self::Inline(ty)
    }
}

impl Display for ValType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::FuncRef => "funcref",
            Self::ExternRef => "externref",
            Self::V128 => "v128",
        })
    }
}

impl Display for NumType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&ValType::from(*self), f)
    }
}

impl Display for RefType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&ValType::from(*self), f)
    }
}

/// Represents a WebAssembly [table type].
///
/// [table type]: https://webassembly.github.io/spec/core/binary/types.html#table-types
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct TableType {
    /// Gets the type of elements stored in the table.
    pub element_type: RefType,
    /// Determines the minimum and maximum number of elements that can be stored in the table, and
    /// whether or not it can be accessed in multiple agents.
    pub limits: types::Limits,
}

/// Represents a [WebAssembly memory type].
///
/// [WebAssembly memory type]: https://webassembly.github.io/spec/core/binary/types.html#memory-types
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct MemType {
    /// Indicates the minimum and maximum number of pages for the linear memory.
    pub limits: types::Limits,
}

impl From<types::Limits> for MemType {
    #[inline]
    fn from(limits: types::Limits) -> Self {
        Self { limits }
    }
}

/// Represents a [**`tagtype`**]. For more information, see the [exception handling proposal].
///
/// [**`tagtype`**]: https://webassembly.github.io/exception-handling/core/syntax/types.html#syntax-tagtype
/// [exception handling proposal]: https://github.com/WebAssembly/exception-handling/
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TagType {
    /// Describes an exception that can be thrown or caught, introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling/).
    ///
    /// The parameter types of the function type that is referred to by the index define the
    /// values that are thrown and caught.
    Exception(crate::module::TypeIdx),
}
