set -e

cargo build

# Unit tests only.
cargo test --lib

# All tests.
(cd rust-jni && cargo test --features libjvm)
