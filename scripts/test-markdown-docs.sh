#!/usr/bin/env bash
set -euo pipefail

cargo build --all-features

# `find ... -quit` can pick an older `libbehave-*.rlib` built with a different
# feature set (e.g. default features only), which makes doc-tests flaky.
# Pick the most recently modified artifact from the `--all-features` build.
LIB_BEHAVE="$(ls -t target/debug/deps/libbehave-*.rlib 2>/dev/null | head -n 1)"

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
