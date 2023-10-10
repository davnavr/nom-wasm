//! Types representing, and traits and functions for parsing,
//! [WebAssembly types](https://webassembly.github.io/spec/core/binary/types.html).

mod func_type;
mod type_parsers;
mod val_type;

pub(crate) use func_type::DebugFuncType;

pub use crate::module::TypeIdx;
pub use func_type::{func_type, FuncTypeParser, ParseFuncType};
pub use type_parsers::{ResultType, ValTypeParser};
pub use val_type::{BlockType, NumType, RefType, ValType, VecType};

/*
crate::tag::enumeration! {
    pub TypeTag : i32 {

    }
}
*/
