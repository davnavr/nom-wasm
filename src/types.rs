//! Types representing, and traits and functions for parsing,
//! [WebAssembly types](https://webassembly.github.io/spec/core/binary/types.html).

mod func_type;
mod global_type;
mod limits;
mod type_parsers;
mod val_type;

pub use crate::module::TypeIdx;
pub use func_type::{func_type_with, ResultTypeIter};
pub use global_type::{GlobalType, Mutability};
pub use limits::{IdxType, LimitBounds, Limits, Sharing};
pub use type_parsers::ValTypeParser;
pub use val_type::{BlockType, MemType, NumType, RefType, TableType, TagType, ValType, VecType};

/*
crate::tag::enumeration! {
    pub TypeTag : i32 {

    }
}
*/
