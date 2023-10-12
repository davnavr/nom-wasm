//! Macros for checking compile time constraints.

macro_rules! check {
    ($expr:expr, $message:expr) => {
        const _: () = if !$expr {
            panic!($message)
        };
    };
}

macro_rules! check_size {
    ($ty:ty, <= $expr:expr) => {
        crate::static_assert::check!(
            core::mem::size_of::<$ty>() <= $expr,
            core::concat!(
                "size_of<",
                core::stringify!($ty),
                "> must not exceed ",
                $expr
            )
        );
    };
}

pub(crate) use check;
pub(crate) use check_size;
