set -e

# Build against master.
sed -i -e 's/rust-jni = ".*"/rust-jni = { path = "..\/..\/..\/rust-jni" }/g' examples/java-lib/Cargo.toml
sed -i -e 's/rust-jni-generator = ".*"/rust-jni-generator = { path = "..\/..\/..\/rust-jni\/generator" }/g' examples/java-lib/Cargo.toml
sed -i -e 's/java = ".*"/java = { path = "..\/..\/..\/rust-jni\/java" }/g' examples/java-lib/Cargo.toml
sed -i -e 's/rust-jni = ".*"/rust-jni = { path = "..\/..\/rust-jni" }/g' generator/Cargo.toml
sed -i -e 's/rust-jni = ".*"/rust-jni = { path = "..\/..\/rust-jni" }/g' java/Cargo.toml
