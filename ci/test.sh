set -e

javac_args=
cargo_args=
if [[ "${JDK}" == "" ]]; then
    # Code for local development.
    (rm examples/java-lib/java/rustjni/test/*.class || true)
    javac_args="--release 8"
    cargo_args="$*"
else
    cargo_args="--verbose --locked"
fi

javac $javac_args examples/java-lib/java/rustjni/test/*.java

cargo test $cargo_args
