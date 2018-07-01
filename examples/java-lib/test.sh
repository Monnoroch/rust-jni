#!/bin/bash

set -e

echo "Running $0 $*..."

(cd java && (rm rustjni/test/*.class || true) && javac rustjni/test/*.java)
cargo test --features libjvm $*
