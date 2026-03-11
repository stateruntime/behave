#!/usr/bin/env bash
set -euo pipefail

cargo build --all-features

LIB_BEHAVE=$(find target/debug/deps -name 'libbehave-*.rlib' -print -quit)

if [ -z "${LIB_BEHAVE}" ]; then
  echo "error: compiled behave library not found in target/debug/deps" >&2
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
    -L dependency=target/debug/deps \
    --extern "behave=${LIB_BEHAVE}"
done
