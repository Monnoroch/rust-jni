set -e

if [[ ${TRAVIS_RUST_VERSION} != "stable" ]]; then
    exit 0
fi

files=$(find . -type f -name "*.rs")

for file in $files; do
    rustfmt --edition 2021 --check "$file"
done
