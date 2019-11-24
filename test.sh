cargo build

# No doc tests.
cargo test --lib --tests

# All tests.
cargo test --features libjvm
