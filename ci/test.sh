set -e

cargo build --verbose

# Unit tests only.
cargo test --verbose --lib

# All tests.
cargo test --verbose --features libjvm
