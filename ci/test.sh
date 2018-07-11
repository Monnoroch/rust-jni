set -e

cargo test --verbose --features libjvm
(cd examples/java-lib && ./test.sh)

if [[ ${TRAVIS_RUST_VERSION} == "nightly" ]]; then
	(cd generator && cargo test --verbose)
fi
