crate::index::definitions! {
    /// A [`typeidx`] is an index into the [*type section*].
    ///
    /// [`typeidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-typeidx
    /// [*type section*]: https://webassembly.github.io/spec/core/binary/modules.html#type-section
    struct TypeIdx = "type";

    /// A [`funcidx`] refers to an [imported function] or a function defined in the [*function section*].
    ///
    /// [`funcidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-funcidx
    /// [imported function]: https://webassembly.github.io/spec/core/syntax/modules.html#syntax-importdesc
    /// [*function section*]: https://webassembly.github.io/spec/core/binary/modules.html#function-section
    struct FuncIdx = "func";

    /// A [`tableidx`] refers to an [imported table] or a table defined in the [*table section*].
    ///
    /// [`tableidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-tableidx
    /// [imported table]: https://webassembly.github.io/spec/core/syntax/modules.html#syntax-importdesc
    /// [*table section*]: https://webassembly.github.io/spec/core/binary/modules.html#table-section
    struct TableIdx = "table";

    /// A [`memidx`] refers to an [imported memory] or a memory defined in the [*memory section*].
    ///
    /// [`memidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-memidx
    /// [imported memory]: https://webassembly.github.io/spec/core/syntax/modules.html#syntax-importdesc
    /// [*memory section*]: https://webassembly.github.io/spec/core/binary/modules.html#memory-section
    struct MemIdx = "mem";

    /// A [`globalidx`] refers to an [imported global] or a global defined in the [*global section*].
    ///
    /// [`globalidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-globalidx
    /// [imported global]: https://webassembly.github.io/spec/core/syntax/modules.html#syntax-importdesc
    /// [*global section*]: https://webassembly.github.io/spec/core/binary/modules.html#global-section
    struct GlobalIdx = "global";

    /// An [`elemidx`] refers to [element segments] in the [*element section*].
    ///
    /// [`elemidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-elemidx
    /// [element segments]: https://webassembly.github.io/spec/core/syntax/modules.html#syntax-elem
    /// [*element section*]: https://webassembly.github.io/spec/core/binary/modules.html#element-section
    struct ElemIdx = "elem";

    /// A [`dataidx`] refers to [data segments] in the [*data section*].
    ///
    /// [`dataidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-dataidx
    /// [data segments]: https://webassembly.github.io/spec/core/syntax/modules.html#syntax-data
    /// [*data section*]: https://webassembly.github.io/spec/core/binary/modules.html#data-section
    struct DataIdx = "data";

    /// A [`localidx`] refers to the parameters and local variables of a function. The types of
    /// each local variable are defined in the [*function section*].
    ///
    /// [`localidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-localidx
    /// [*function section*]: https://webassembly.github.io/spec/core/binary/modules.html#function-section
    struct LocalIdx = "local";

    /// A [`labelidx`] refers to [structured control instructions] within the code of a function.
    ///
    /// [`labelidx`]: https://webassembly.github.io/spec/core/binary/modules.html#binary-labelidx
    /// [structured control instructions]: https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control
    struct LabelIdx = "label";

    /// A [`tagidx`] refers to a [*tag*s] in the [*tag section*] introduced as part of the
    /// [exception handling proposal].
    ///
    /// [`tagidx`]: https://webassembly.github.io/exception-handling/core/syntax/modules.html#syntax-tagidx
    /// [*tag*s]: https://webassembly.github.io/exception-handling/core/syntax/modules.html#tags
    /// [*tag section*]: https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section
    /// [exception handling proposal]: https://github.com/WebAssembly/exception-handling
    struct TagIdx = "tag";
}
