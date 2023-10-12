use crate::{
    storage::Vector,
    types::{ParseFuncType, ValType},
};
use nom::ToUsize;

/// Represents a WebAssembly [function type] with a heap allocation.
///
/// To parse a function type without allocating, use the [`types::func_type()`] parser instead.
///
/// [function type]: https://webassembly.github.io/spec/core/binary/types.html#function-types
/// [`types::func_type()`]: crate::types::func_type()
pub struct FuncType<V: Vector<Item = ValType>> {
    types: V::Boxed,
    parameter_count: u32,
    _marker: core::marker::PhantomData<fn() -> V>,
}

impl<V: Vector<Item = ValType> + Default> Default for FuncType<V> {
    #[inline]
    fn default() -> Self {
        Self::new(V::default(), 0)
    }
}

impl<V: Vector<Item = ValType>> FuncType<V> {
    fn new(types: V, parameters: u32) -> Self {
        assert!(parameters.to_usize() <= types.len());

        Self {
            types: types.into_boxed_slice(),
            parameter_count: parameters,
            _marker: core::marker::PhantomData,
        }
    }

    /// Parses a [`FuncType`].
    ///
    /// # Errors
    ///
    /// See the documentation for the [`types::func_type()`] parser for more information.
    ///
    /// [`types::func_type()`]: crate::types::func_type()
    pub fn parse<'a, E, B>(
        input: &'a [u8],
        builder: &mut BuildFuncType<V>,
    ) -> crate::Parsed<'a, Self, E>
    where
        E: crate::error::ErrorSource<'a>,
        V: Clone,
    {
        crate::types::func_type(input, &mut builder.builder)
            .map(|(input, builder)| (input, builder.current()))
    }

    /// Gets the parameter types.
    #[inline]
    pub fn parameters(&self) -> &[ValType] {
        &self.types[..self.parameter_count.to_usize()]
    }

    /// Gets the results types.
    #[inline]
    pub fn results(&self) -> &[ValType] {
        &self.types[self.parameter_count.to_usize()..]
    }

    /// Returns a boxed slice containing the parameter types followed by the result types.
    #[inline]
    pub fn into_types(self) -> V::Boxed {
        self.types
    }
}

impl<V: Vector<Item = ValType>> core::fmt::Debug for FuncType<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FuncType")
            .field("parameters", &self.parameters())
            .field("results", &self.results())
            .finish()
    }
}

#[derive(Clone, Default)]
struct Builder<V: Vector<Item = ValType> + Clone> {
    types: V,
    parameter_count: Option<u32>,
}

/// Contains a [`Vector`] buffer for use when parsing a [`FuncType`].
pub struct BuildFuncType<V: Vector<Item = ValType> + Clone> {
    builder: Builder<V>,
}

impl<V: Vector<Item = ValType> + Default + Clone> Default for BuildFuncType<V> {
    #[inline]
    fn default() -> Self {
        Self::with_buffer(V::default())
    }
}

impl<V: Vector<Item = ValType> + Clone> Clone for BuildFuncType<V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            builder: self.builder.clone(),
        }
    }
}

impl<V: Vector<Item = ValType> + Clone> BuildFuncType<V> {
    #[allow(missing_docs)]
    pub fn with_buffer(buffer: V) -> Self {
        Self {
            builder: Builder {
                types: buffer,
                parameter_count: None,
            },
        }
    }
}

impl<V: Vector<Item = ValType> + Clone> core::fmt::Debug for BuildFuncType<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BuildFuncType").finish_non_exhaustive()
    }
}

impl<V: Vector<Item = ValType> + Clone> Builder<V> {
    /// Gets the [`FuncType`] that was just parsed.
    fn current(&mut self) -> FuncType<V> {
        FuncType::new(self.types.clone(), self.parameter_count.unwrap_or(u32::MAX))
    }
}

impl<V: Vector<Item = ValType> + Clone> crate::types::ParseResultType for Builder<V> {
    fn with_count(&mut self, count: usize) {
        self.types.reserve(count);
        if self.parameter_count.is_none() {
            self.parameter_count = Some(count.try_into().unwrap_or(u32::MAX));
        }
    }

    #[inline]
    fn next_type(&mut self, value_type: ValType) {
        self.types.push(value_type);
    }
}

impl<V: Vector<Item = ValType> + Clone> ParseFuncType for Builder<V> {
    type ResultType<'a> = &'a mut Self where V: 'a;

    #[inline]
    fn parameters(&mut self) -> Self::ResultType<'_> {
        self.parameter_count = None;
        self
    }

    #[inline]
    fn results(&mut self) -> Self::ResultType<'_> {
        self
    }
}
