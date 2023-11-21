use nom_wasm::{
    error::VerboseError,
    isa::{instructions, ParseInstr as _},
};

#[test]
fn basic_expr() {
    use nom::Parser as _;
    use std::fmt::Write as _;

    let expr: &[u8] = &[0x20, 0x00, 0x41, 0x2A, 0x6A, 0x0F, 0x01, 0x0B];
    let mut results = allocator_api2::vec::Vec::with_capacity(6);

    let mut parser = instructions::Parser::<VerboseError, _>::new(&mut results);
    parser.parse_expr(expr).unwrap();

    let mut text = String::with_capacity(128);
    for instr in results.into_iter() {
        let _ = writeln!(&mut text, "{instr}");
    }

    insta::assert_display_snapshot!(&text);
}
