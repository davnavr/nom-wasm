//! Types representing and functions for parsing
//! [WebAssembly types](https://webassembly.github.io/spec/core/binary/types.html).

mod parse_type;
mod val_type;

pub use crate::module::TypeIdx;
pub use val_type::{BlockType, NumType, RefType, ValType, VecType};

/*
crate::tag::enumeration! {
    pub TypeTag : i32 {

    }
}
*/
