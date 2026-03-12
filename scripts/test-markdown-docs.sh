#!/usr/bin/env bash
set -euo pipefail

DOC_TARGET_DIR="${CARGO_TARGET_DIR:-target}/markdown-docs"

# Use a dedicated target dir so this script never reuses `behave` artifacts
# from other CI steps (e.g. `--no-default-features` test runs).
cargo build --all-features --target-dir "${DOC_TARGET_DIR}"

LIB_BEHAVE="$(ls -t "${DOC_TARGET_DIR}"/debug/deps/libbehave-*.rlib 2>/dev/null | head -n 1)"

if [ -z "${LIB_BEHAVE}" ]; then
  echo "error: compiled behave library not found in ${DOC_TARGET_DIR}/debug/deps" >&2
  exit 1
fi

for doc in \
  README.md \
  docs/USER_GUIDE.md \
  docs/MATCHERS.md \
  docs/CLI.md \
  docs/RELIABILITY.md; do
  echo "Testing markdown doc: ${doc}"
  rustdoc \
    --edition=2021 \
    --test "${doc}" \
    -L dependency="${DOC_TARGET_DIR}/debug/deps" \
    --extern "behave=${LIB_BEHAVE}"
done
