//! Types representing, and traits and functions for parsing,
//! [WebAssembly types](https://webassembly.github.io/spec/core/binary/types.html).

mod func_type;
mod type_parsers;
mod val_type;

pub use crate::module::TypeIdx;
pub use func_type::{func_type, ParseFuncType};
pub use type_parsers::{ParseValType, ResultType};
pub use val_type::{BlockType, NumType, RefType, ValType, VecType};

/*
crate::tag::enumeration! {
    pub TypeTag : i32 {

    }
}
*/
