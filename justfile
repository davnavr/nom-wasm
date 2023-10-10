alias c := clippy_cfg_std
alias clippy := clippy_all_cfgs
alias d := doc
alias f := fmt
alias t := test

cfg_no_alloc := "--no-default-features --features arrayvec"
cfg_no_std := cfg_no_alloc + " --features alloc"

# Invokes fmt, clippy, then runs all unit tests
check: fmt clippy_all_cfgs test

# Runs all unit tests
test $RUST_BACKTRACE="1":
    cargo test

# Invoke fmt on the main source code
fmt *FLAGS='--all':
    cargo fmt {{FLAGS}}

# Invoke clippy for the full no_std configuration
clippy_cfg_no_alloc:
    cargo clippy {{cfg_no_alloc}}

# Invoke clippy for the no_std + alloc configuration
clippy_cfg_no_std:
    cargo clippy {{cfg_no_std}}

# Invoke clippy for the default configuration
clippy_cfg_std:
    cargo clippy

# Invoke clippy for all 3 main configurations
clippy_all_cfgs: clippy_cfg_std clippy_cfg_no_std clippy_cfg_no_alloc

# Invoke rustdoc; requires a nightly version of Rust
doc *FLAGS='--open':
    RUSTDOCFLAGS="--cfg doc_cfg" cargo +nightly doc {{FLAGS}}
