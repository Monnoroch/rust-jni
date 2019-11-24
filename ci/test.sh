set -e

cargo build --verbose

# Unit tests only.
cargo test --verbose --lib

# All tests.
# TOOD(https://github.com/rust-lang/cargo/issues/5015): stop cd-ing into individual
# crates once the features bug is fixed.
(cd rust-jni && cargo test --verbose --features libjvm)
