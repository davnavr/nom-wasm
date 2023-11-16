alias c := clippy_cfg_default
alias d := doc
alias f := fmt
alias t := test

feat_unsafe := "--features allow-unsafe"
cfg_no_alloc := "--no-default-features"
cfg_no_std := cfg_no_alloc + " --features alloc --features allocator-api2"

# Invokes fmt, clippy, then runs all unit tests
check: fmt clippy test

# Runs all unit tests
test $RUST_BACKTRACE="1":
    cargo test
    cargo test {{cfg_no_std}} --features std

# Invoke fmt on the main source code
fmt *FLAGS='--all':
    cargo fmt {{FLAGS}}

# Invoke clippy for the full `no_std` configuration
clippy_cfg_no_alloc:
    cargo clippy {{cfg_no_alloc}}
    cargo clippy {{cfg_no_alloc}} {{feat_unsafe}}

# Invoke clippy for the `alloc` configuration
clippy_cfg_no_std:
    cargo clippy {{cfg_no_std}}
    cargo clippy {{cfg_no_std}} {{feat_unsafe}}

# Invoke clippy for the default configuration
clippy_cfg_default:
    cargo clippy {{cfg_no_std}} --features std
    cargo clippy

# Invoke clippy for all 3 main configurations
clippy: clippy_cfg_default clippy_cfg_no_std clippy_cfg_no_alloc

# Invoke rustdoc; requires a nightly version of Rust
doc *FLAGS='--open':
    RUSTDOCFLAGS="--cfg doc_cfg" cargo +nightly doc {{FLAGS}}
