//! Macros for checking compile time constraints.

macro_rules! check {
    ($expr:expr, $message:expr) => {
        const _: () = if !$expr {
            panic!($message)
        };
    };
}

macro_rules! check_size {
    ($ty:ty, <= $expr:literal) => {
        crate::static_assert::check!(
            core::mem::size_of::<$ty>() <= $expr,
            concat!("size_of<", stringify!($ty), "> cannot exceed ", $expr,)
        );
    };
    ($ty:ty, <= $expr:expr) => {
        crate::static_assert::check!(
            core::mem::size_of::<$ty>() <= $expr,
            concat!(
                "size_of<",
                stringify!($ty),
                "> cannot exceed `",
                stringify!($expr),
                "`"
            )
        );
    };
}

macro_rules! object_safe {
    ($($trait:tt)+) => {
        const _: core::marker::PhantomData<&'static dyn $($trait)+> = core::marker::PhantomData;
    };
}

pub(crate) use check;
pub(crate) use check_size;
pub(crate) use object_safe;
