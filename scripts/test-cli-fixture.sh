#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FIXTURE_SRC="${ROOT_DIR}/examples/cli-workspace"
TMP_DIR="$(mktemp -d)"
FIXTURE_DIR="${TMP_DIR}/cli-workspace"
CLI_BIN="${ROOT_DIR}/target/debug/cargo-behave"

cleanup() {
  rm -rf "${TMP_DIR}"
}

trap cleanup EXIT

cp -R "${FIXTURE_SRC}" "${FIXTURE_DIR}"

escaped_root="$(printf '%s\n' "${ROOT_DIR}" | sed 's/[\/&]/\\&/g')"
while IFS= read -r -d '' manifest; do
  tmp_manifest="${manifest}.tmp"
  sed "s#__BEHAVE_PATH__#${escaped_root}#g" "${manifest}" > "${tmp_manifest}"
  mv "${tmp_manifest}" "${manifest}"
done < <(find "${FIXTURE_DIR}" -name Cargo.toml -print0)

cargo build --features cli --bin cargo-behave
export CARGO_NET_OFFLINE=true

json_output="${TMP_DIR}/fixture-report.json"
"${CLI_BIN}" \
  behave \
  --output json \
  --manifest-path "${FIXTURE_DIR}/Cargo.toml" \
  --package cli-fixture-api \
  > "${json_output}"

grep -q '"command_success": true' "${json_output}"
grep -q '"total": 2' "${json_output}"
grep -q '"full_name": "checkout::alpha_case"' "${json_output}"
grep -q '"full_name": "checkout::zeta_case"' "${json_output}"
grep -q '"name": "alpha_case"' "${json_output}"
grep -q '"focused": true' "${json_output}"

if grep -q 'pricing::other_package_case' "${json_output}"; then
  echo "error: JSON report unexpectedly included tests from the unselected package" >&2
  exit 1
fi

alpha_line="$(grep -n '"full_name": "checkout::alpha_case"' "${json_output}" | cut -d: -f1)"
zeta_line="$(grep -n '"full_name": "checkout::zeta_case"' "${json_output}" | cut -d: -f1)"

if [ "${alpha_line}" -ge "${zeta_line}" ]; then
  echo "error: JSON report was not sorted deterministically" >&2
  exit 1
fi

junit_output="${TMP_DIR}/fixture-report.xml"
"${CLI_BIN}" \
  behave \
  --output junit \
  --manifest-path "${FIXTURE_DIR}/crates/api/Cargo.toml" \
  > "${junit_output}"

grep -q '<testsuite name="cargo-behave"' "${junit_output}"
grep -q 'classname="checkout" name="alpha_case"' "${junit_output}"
grep -q 'classname="checkout" name="zeta_case"' "${junit_output}"

history_file="${FIXTURE_DIR}/crates/api/target/behave/history.json"
if [ ! -f "${history_file}" ]; then
  echo "error: expected package-local flaky history file was not created" >&2
  exit 1
fi
