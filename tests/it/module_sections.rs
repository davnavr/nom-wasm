use nom_wasm::error::VerboseError;

/*
#[test]
fn type_sec_example() {
    let bytes = Bytes::from(&[
        1u8,  // count
        0x60, // func
        1,    // parameter count
        0x7F, // i32
        1,    // result count
        0x7E, // i64
    ]);

    let mut errors = InlineErrorReporter::<8>::new();
    let result = parser::Vector::with_parsed_count(&mut errors, bytes)
        .map(module::TypeSec::from)
        .map_err(|_| errors.expect_report());

    insta::assert_debug_snapshot!(result)
}
*/

#[test]
fn import_sec_example() {
    let mut bytes = Vec::with_capacity(128);
    bytes.extend([
        4, // count
        3, // module name length
    ]);
    bytes.extend(b"env");
    bytes.push(0xB); // name length
    bytes.extend(b"doSomeStuff");
    bytes.extend([
        0, // import func
        0, // typeidx
        3, // module name length
    ]);
    bytes.extend(b"env");
    bytes.push(6); // name length
    bytes.extend(b"memory");
    bytes.extend([
        2,    // import memory
        0,    // limit w/o maximum
        0x10, // limit minimum
        2,    // module name length
    ]);
    bytes.extend(b"rt");
    bytes.push(0xA); // name length
    bytes.extend(b"references");
    bytes.extend([
        1,    // import table,
        0x6F, // externref
        0, 0, // limits
        2, // module name length
    ]);
    bytes.extend(b"rt");
    bytes.push(8); // name length
    bytes.extend(b"stackptr");
    bytes.extend([
        3,    // import global
        0x7F, // i32
        1,    // mutable
    ]);

    let result = nom_wasm::module::ImportSec::parse::<VerboseError>(&bytes);

    insta::assert_debug_snapshot!(result);
}
