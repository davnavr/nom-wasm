use nom_wasm::{error::VerboseError, leb128};

#[test]
fn valid_u32() {
    macro_rules! assert_eq_decoded {
        ($expected:expr, $input:expr) => {
            assert_eq!(
                leb128::u32::<VerboseError>(&$input),
                Ok(([].as_slice(), $expected))
            );
        };
    }

    assert_eq_decoded!(0, [0]);
    assert_eq_decoded!(0x7F, [0x7F]);
    assert_eq_decoded!(0x80, [0x80, 1]);
    assert_eq_decoded!(3, [0x83, 0]); // Trailing zeroes are allowed, used in linker output
    assert_eq_decoded!(0x3FFF, [0xFF, 0x7F]);
    assert_eq_decoded!(0x4000, [0x80, 0x80, 1]);
    assert_eq_decoded!(0x1FFFFF, [0xFF, 0xFF, 0x7F]);
    assert_eq_decoded!(0x200000, [0x80, 0x80, 0x80, 1]);
    assert_eq_decoded!(15, [0x8F, 0x80, 0x80, 0]);
    assert_eq_decoded!(0x0FFFFFFF, [0xFF, 0xFF, 0xFF, 0x7F]);
    assert_eq_decoded!(0x10000000, [0x80, 0x80, 0x80, 0x80, 1]);
    assert_eq_decoded!(u32::MAX, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
}

#[test]
fn valid_s32() {
    macro_rules! assert_eq_decoded {
        ($expected:expr, $input:expr) => {
            assert_eq!(
                leb128::s32::<VerboseError>(&$input),
                Ok(([].as_slice(), $expected))
            );
        };
    }

    assert_eq_decoded!(0, [0]);
    assert_eq_decoded!(-1, [0x7F]);
    assert_eq_decoded!(63, [0x3F]);
    assert_eq_decoded!(-64, [0x40]);
    assert_eq_decoded!(-2, [0x7E]);
    assert_eq_decoded!(64, [0xC0, 0]);
    assert_eq_decoded!(128, [0x80, 1]);
    assert_eq_decoded!(i32::from(u8::MAX), [0xFF, 1]);
    assert_eq_decoded!(i32::from(i8::MIN), [0x80, 0x7F]);
    assert_eq_decoded!(i32::from(i8::MAX), [0xFF, 0]);
    assert_eq_decoded!(-2, [0xFE, 0x7F]);
    assert_eq_decoded!(8191, [0xFF, 0x3F]);
    assert_eq_decoded!(-8192, [0x80, 0x40]);
    assert_eq_decoded!(-2, [0xFE, 0xFF, 0x7F]);
    assert_eq_decoded!(1048575, [0xFF, 0xFF, 0x3F]);
    assert_eq_decoded!(-1048576, [0x80, 0x80, 0x40]);
    assert_eq_decoded!(i32::from(i16::MAX), [0xFF, 0xFF, 1]);
    assert_eq_decoded!(i32::from(i16::MIN), [0x80, 0x80, 0x7E]);
    assert_eq_decoded!(i32::from(u16::MAX), [0xFF, 0xFF, 0x03]);
    assert_eq_decoded!(134217727, [0xFF, 0xFF, 0xFF, 0x3F]);
    assert_eq_decoded!(-134217728, [0x80, 0x80, 0x80, 0x40]);
    assert_eq_decoded!(i32::MAX, [0xFF, 0xFF, 0xFF, 0xFF, 0x07]);
    assert_eq_decoded!(i32::MIN, [0x80, 0x80, 0x80, 0x80, 0x78]);
    assert_eq_decoded!(-17, [0x6F]);
}

#[test]
fn valid_s64() {
    macro_rules! assert_eq_decoded {
        ($expected:expr, $input:expr, $remaining:expr) => {
            assert_eq!(
                leb128::s64::<VerboseError>(&$input),
                Ok(($remaining.as_slice(), $expected))
            );
        };
        ($expected:expr, $input:expr) => {
            assert_eq_decoded!($expected, $input, [])
        };
    }

    assert_eq_decoded!(0, [0]);
    assert_eq_decoded!(-1, [0x7F]);
    assert_eq_decoded!(-17, [0x6F]);
    assert_eq_decoded!(
        -8029759185026510704,
        [0x90, 0xA1, 0xC2, 0x84, 0x89, 0x92, 0xA4, 0xC8, 0x90, 0x7F]
    );
    assert_eq_decoded!(
        11502565375,
        [0xFF, 0xEF, 0xEC, 0xEC, 0x2A, 0xEC, 0x31],
        [0xEC, 0x31]
    );
    assert_eq_decoded!(
        -1379794179797546992,
        [0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0xFF, 0xEC, 0x6C]
    );
}
