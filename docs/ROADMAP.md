# Roadmap to v1.0

The path from v0.5.0 to v1.0 — every planned feature, its rationale, what we
learned from competitors, and what we deliberately will not build.

## Vision

> **behave** is the integrated BDD testing framework for Rust: one DSL, one
> assertion library, one CLI — cohesive, zero-runtime, compiling to ordinary
> `#[test]` functions.

The moat is cohesion. The combo of rstest + googletest + pretty_assertions gives
users fixtures, matchers, and diffs — but requires learning 3 crates with
incompatible attribute styles that [don't compose cleanly][test-case-duplication].
behave is one import, one mental model.

[test-case-duplication]: https://github.com/frondeus/test-case/issues/146

## Competitive update (2026-03-12)

When this roadmap started, the "nested suite / RSpec-like" niche in Rust looked
quiet. That is no longer true. There are now actively-maintained integrated BDD
frameworks and BDD-adjacent crates that overlap with behave's surface:

- **spectacular** — RSpec-like suites with rich hooks/context injection and
  async runtime support: <https://spectacular.vercel.app/>
- **rsspec** — RSpec/Ginkgo-inspired suites with workflow knobs like retries,
  timeouts, focus enforcement: <https://docs.rs/rsspec>
- **rstest-bdd** — Gherkin feature files with step definitions powered by
  `rstest` fixtures: <https://lib.rs/crates/rstest-bdd>
- **test-casing** — parameterized tests plus decorators like retries/timeouts:
  <https://docs.rs/test-casing>

**Roadmap implication:** workflow features (tags, filtering, skip/xfail, retry,
timeouts, focus guard) are not "polish" — they are adoption features. behave
must keep its differentiation (explicit setup, ordinary `#[test]` output,
excellent failures, compile-time budget), while meeting baseline workflow
expectations early.

For a deeper competitive snapshot, see [feature_competition.md](feature_competition.md).

## Principles (informed by competitor analysis)

These principles come from studying the mistakes and successes of RSpec, pytest,
Jest, Go testing, Vitest, Catch2, rstest, nextest, and others. Each principle
links to the evidence behind it.

### 1. Explicit over magic

RSpec's `let` creates ["mystery guests"][mystery-guest] where setup is invisible.
pytest's `conftest.py` creates [cascading implicit configuration][conftest-chaos].
Jest injects globals that break linting. behave's `setup` blocks are eager,
visible, and paste directly into the test body. No lazy evaluation, no implicit
injection, no globals.

[mystery-guest]: https://thoughtbot.com/blog/lets-not
[conftest-chaos]: https://github.com/pytest-dev/pytest/issues/13913

### 2. Duplication over the wrong abstraction

RSpec shared examples are [widely considered an anti-pattern][shared-antipattern]:
ghost variables, exponential test growth, debugging nightmares with bracket
coordinates like `[1:2:5:1]`. Solnic's "5 Rules of Simple RSpec Tests"
[explicitly advocates][solnic-rules] higher tolerance for duplication.
Tests are documentation — readability beats DRY.

[shared-antipattern]: https://dev.to/epigene/the-case-against-shared-examples-39kh
[solnic-rules]: https://solnic.dev/the-5-rules-of-simple-rspec-tests/

### 3. Flat over nested

RuboCop-RSpec [enforces max 2 levels][rubocop-nesting] of nesting. 8th Light
calls deep nesting ["context puzzles"][context-puzzles]. Each level adds implicit
state. behave supports nesting for readability but the docs should model 2-3
levels as the sweet spot.

[rubocop-nesting]: https://github.com/rubocop/rubocop-rspec
[context-puzzles]: https://8thlight.com/insights/nested-contexts-are-only-kind-of-evil

### 4. Error messages are everything

[Every][pytest-messages] [framework][go-messages] [analysis][jest-messages]
converges: the #1 quality dimension is what happens when a test fails. Expected
value, actual value, colored diff, source location. One matcher with perfect
error messages beats ten with mediocre ones.

[pytest-messages]: https://enterprisecraftsmanship.com/posts/assertion-messages-in-tests/
[go-messages]: https://www.alexedwards.net/blog/the-9-go-test-assertions-i-use
[jest-messages]: https://kentcdodds.com/blog/improve-test-error-messages-of-your-abstractions

### 5. Compile time is existential

C++ doctest achieves [<20ms overhead][doctest-speed]. pretty_assertions triggers
[96.5 GB allocation][pa-oom] on large inputs. rstest users report
[minutes-long compilation][rstest-compile] for large parameterized suites.
Every macro expansion, every generic instantiation matters. behave must set a
compile-time budget and measure it.

[doctest-speed]: https://github.com/doctest/doctest
[pa-oom]: https://github.com/rust-pretty-assertions/rust-pretty-assertions/issues/124
[rstest-compile]: https://lobste.rs/s/wrq7iv/guide_test_parametrization_rust

### 6. Per-test isolation is correct

pytest's session-scoped fixtures [break in parallel][xdist-session]. rstest's
`#[once]` [never drops][rstest-once] and is [incompatible with nextest][nextest-once].
Per-test setup with explicit `OnceLock` in user code is the right pattern for
expensive shared resources.

[xdist-session]: https://github.com/pytest-dev/pytest-xdist/issues/271
[rstest-once]: https://github.com/la10736/rstest/issues/209
[nextest-once]: https://github.com/nextest-rs/nextest/issues/209

### 7. Work with the ecosystem, not against it

test-case [shadows an unstable compiler attribute][testcase-unstable] — "liable
to stop working in any future release of Rust." behave generates standard
`#[test]` functions, works with `cargo test`, `cargo-nextest`, IDEs, and CI
without special configuration. Never break this.

[testcase-unstable]: https://github.com/frondeus/test-case/issues/131

---

## Release Plan

### v0.6.0 — "Better Parameterization"

**Theme:** Close the parameterization gap with rstest/test-case. Make
table-driven testing best-in-class.

#### Feature: Test matrix (Cartesian product)

**What:** `matrix` keyword generates tests from all combinations of input
dimensions.

```rust
behave! {
    "formatting" {
        matrix [1, 2, 3] x ["a", "b"] |n, s| {
            let result = format!("{n}{s}");
            expect!(result).to_not_be_empty()?;
        }
    }
}
// Generates 6 tests: case_0_0 through case_2_1
```

**Why:** Both rstest (`#[values]`) and test-case (`#[test_matrix]`) have this.
It is the #1 parameterization feature behave lacks. Without it, users must
write O(n*m) explicit tuples in `each` blocks.

**Competitor evidence:**
- rstest `#[values(1, 2, 3)]` on multiple params generates Cartesian product
- test-case `#[test_matrix([1, 2], ["a", "b"])]` does the same
- Go table-driven tests with nested loops are the manual equivalent

**Implementation:** Parser addition for `matrix [...] x [...] |params| { body }`
syntax. Codegen generates nested module with `case_I_J` functions. Inherits
setup/teardown/tokio/focus from parent like `each` does.

**Effort:** Medium (parser + codegen, follows `each` pattern).

#### Feature: Named test cases in `each`

**What:** Optional string label as first element of each tuple becomes the test
function name instead of `case_N`.

```rust
behave! {
    "http status" {
        each [
            ("ok", 200, true),
            ("not_found", 404, false),
            ("server_error", 500, false),
        ] |name, code, success| {
            expect!(code < 400).to_equal(success)?;
        }
    }
}
// Generates: ok, not_found, server_error (not case_0, case_1, case_2)
```

**Why:** rstest users [complain about numeric prefixes][rstest-names] (#167).
Descriptive test names make `cargo test` output and CI reports far more readable.
When a test fails, `http_status::server_error` is immediately informative;
`http_status::case_2` is not.

[rstest-names]: https://github.com/la10736/rstest/issues/167

**Implementation:** During parsing, check if the first element of each tuple is
a string literal. If so, slugify it as the test function name. Fall back to
`case_N` if absent or if the slug collides.

**Effort:** Small (parser tweak + slug generation already exists).

#### Feature: `xfail` keyword

**What:** Mark a test as expected-to-fail. The test runs; it passes if the body
fails, and fails loudly if the body unexpectedly passes.

```rust
behave! {
    "known bugs" {
        xfail "off-by-one in leap year calc" {
            expect!(days_in_feb(2024)).to_equal(29)?;
        }
    }
}
```

**Why:** Different from `pending` (which skips via `#[ignore]`). `xfail` still
runs the test, which means:
- Known bugs remain visible in test output
- When the bug is fixed, the test fails — prompting the developer to remove
  `xfail` and promote it to a real test
- CI tracks regression without manual intervention

**Competitor evidence:**
- pytest `@pytest.mark.xfail` — one of the most-used markers
- RSpec `pending` (confusingly) runs the test and fails if it passes
- Google Test `EXPECT_DEATH` serves a similar "expect failure" role

**Implementation:** Codegen wraps the test body in a block that catches
`Result::Err` and returns `Ok(())`, and converts `Ok(())` into
`Err("expected failure but test passed")`.

**Effort:** Small (codegen wrapper, new AST node).

---

### v0.7.0 — "Tags & Filtering"

**Theme:** Graduate from `focus`/`pending` to real test metadata. Give users
control over which tests run.

#### Feature: Tag-based test metadata

**What:** `tag` keyword attaches arbitrary string labels to groups or tests.
Tags are encoded in the generated test function name and filterable by
`cargo behave`.

```rust
behave! {
    "database" tag "slow", "integration" {
        "creates a user" {
            // ...
        }
    }

    "parser" tag "fast" {
        "handles empty input" {
            // ...
        }
    }
}
```

```bash
cargo behave --tag integration          # run only tagged tests
cargo behave --exclude-tag slow         # skip slow tests
cargo behave --tag fast --tag unit      # union: fast OR unit
```

**Why:** Every major framework has this. RSpec has metadata tags. pytest has
markers. Google Test has `--gtest_filter`. JUnit 5 has `@Tag`. Currently behave
only has `focus` and `pending`, which are binary. Real-world test suites need:
`slow`, `integration`, `network`, `database`, `nightly`, `flaky`.

**Implementation:**
- Parser: `tag "label"` syntax before group/test blocks
- Codegen: encode tags in the test function name as `__TAG_slow__` segments
  (same pattern as `__FOCUS__` / `__PENDING__`)
- CLI: `--tag` and `--exclude-tag` flags filter by tag presence in test names
- JUnit/JSON output: strip tag prefixes for clean display names

`focus` and `pending` become syntactic sugar for special tags internally.

**Effort:** Medium (parser + codegen + CLI filter logic).

#### Feature: Focus-only mode + CI guard

**What:** Add two CLI controls around focused tests:

- `cargo behave --focus` runs only focused tests *if any exist*; otherwise runs
  the full suite.
- `cargo behave --fail-on-focus` exits non-zero if any focused tests exist
  (protects CI from accidentally committed focus markers).

**Why:** Focus markers are a useful workflow tool, but they become a footgun
without a guard. Other frameworks treat this seriously (for example, rsspec
supports a "fail on focus" mode).

**Implementation:**
- Reuse the existing `__FOCUS__` marker in generated names and CLI parsing.
- `--focus`: list tests, detect focused subset, then filter execution to them.
- `--fail-on-focus`: detect focused tests and error out with a helpful summary.

**Effort:** Small (CLI only).

#### Feature: Runtime conditional skip

**What:** `skip_when!` macro evaluates a condition at runtime. If true, the test
is reported as skipped (not passed, not failed).

```rust
behave! {
    "redis integration" {
        "stores a value" {
            skip_when!(std::env::var("REDIS_URL").is_err(), "REDIS_URL not set");
            // ... test body
        }
    }
}
```

**Why:** The [#1 most-wanted Rust testing feature][skip-rfc]. libtest reports
early-returning tests as "passed," which is misleading. Since behave generates
the test body, it can emit a return with a message that `cargo-behave`'s parser
recognizes as "skipped" and reports accordingly in tree/JSON/JUnit output.
Frameworks like rsspec also provide runtime skipping, which is a strong signal
that this is workflow-critical, not niche.

[skip-rfc]: https://github.com/rust-lang/rust/issues/68007

**Implementation:**
- Macro: `skip_when!(condition, reason)` expands to an early return with a
  printed sentinel message (e.g., `BEHAVE_SKIP: reason`)
- CLI parser: detect the sentinel in test output and report as "skipped"
- `cargo test` (without CLI): test passes silently (safe fallback)
- Tree output: show skipped tests with a distinct marker

**Constraint:** With `cargo test` alone (no `cargo-behave`), skipped tests show
as passing. This matches the libtest limitation and is documented. The CLI
provides the correct reporting.

**Effort:** Small-Medium (macro + CLI parser enhancement).

#### Feature: Watch mode for `cargo-behave`

**What:** `cargo behave --watch` re-runs tests when source files change.

```bash
cargo behave --watch                    # watch src/ and tests/
cargo behave --watch --tag fast         # watch, only run fast tests
```

**Why:** Vitest proved watch mode is the [#1 DX feature][vitest-watch] that
makes a test framework feel fast. Jest's watch mode re-runs only affected tests
based on the dependency graph. For v0.7, a simpler approach: re-run all tests on
any `.rs` file change in `src/` or `tests/`.

[vitest-watch]: https://vitest.dev/guide/features.html

**Implementation:**
- Use the `notify` crate to watch filesystem events
- Debounce with 200ms delay
- Re-invoke the existing `cargo behave` pipeline
- Clear terminal between runs
- Show "watching for changes..." when idle

**Effort:** Medium (new CLI mode, `notify` dependency behind `cli` feature).

---

### v0.8.0 — "Type Power"

**Theme:** Leverage Rust's type system for testing patterns no other crate
offers. Build unique differentiators.

#### Feature: Typed test generation

**What:** `each_type` generates the same test body for multiple concrete types.

```rust
behave! {
    "numeric identity" {
        each_type [i32, i64, u32, u64, f32, f64] {
            "zero plus zero equals zero" {
                let zero = <T as Default>::default();
                expect!(zero + zero).to_equal(zero)?;
            }
        }
    }
}
// Generates: i32::zero_plus_zero_equals_zero, i64::..., etc.
```

**Why:** Google Test's `TYPED_TEST_SUITE` is [widely used][gtest-typed] for
testing generic code. **No Rust testing crate has this.** It plays directly to
Rust's strengths (generics, trait bounds). The alternative today is copy-pasting
test bodies or writing a custom macro.

[gtest-typed]: http://google.github.io/googletest/advanced.html#typed-tests

**Implementation:**
- Parser: `each_type [Type1, Type2, ...] { ... }` where `T` is implicitly
  available in the body
- Codegen: for each type, generate a module named after the type containing all
  inner tests, with `type T = ConcreteType;` at the top
- Inherits setup/teardown/tags from parent

**Effort:** Medium (parser + codegen, follows `each` pattern).

#### Feature: Pattern matching matcher

**What:** `to_match_pattern!` uses Rust's pattern syntax for structural
assertions.

```rust
expect!(result).to_match_pattern!(Ok(value) if value > 0)?;
expect!(event).to_match_pattern!(Event::Click { x: 0..=100, .. })?;
```

**Why:** More idiomatic than `to_satisfy` for enum variants and struct
destructuring. Rust's pattern matching is one of its best features — the
assertion library should leverage it. Produces clear error messages:
"expected: to match `Ok(value) if value > 0`, actual: `Err("timeout")`".

**Implementation:** New declarative macro that wraps `matches!()` with
`MatchError` construction including the pattern as a string. Cannot be a method
on `Expectation<T>` because patterns are not expressions.

**Effort:** Small (declarative macro, no proc macro changes).

#### Feature: Partial struct matching (feature-flagged)

**What:** Assert that a `serde_json::Value` (or any `Serialize` type) contains
a subset of expected fields.

```rust
expect!(response_json).to_contain_fields(json!({
    "status": 200,
    "ok": true
}))?;
// passes even if response_json has additional fields
```

**Why:** Jest's `toMatchObject` is [one of its most-used matchers][jest-match].
API testing commonly checks specific fields without asserting the entire
structure. Rust's strong typing makes this less necessary for known types, but
JSON APIs and dynamic data benefit greatly.

[jest-match]: https://jestjs.io/docs/expect#tomatchobjectobject

**Implementation:**
- Behind `serde` feature flag (adds `serde` + `serde_json` to core deps)
- Recursive comparison: for each key in expected, assert it exists in actual
  with the same value
- Array matching: expected array elements must appear in the same positions

**Effort:** Medium (new feature flag, recursive comparator, error formatting).

#### Feature: Matcher essentials (paths, errors, sets, sequences)

**What:** Expand behave's "daily driver" matcher set to cover the types and
assertion shapes that appear constantly in real Rust test suites:

- **Sequences (Vec/slice):** `to_contain_exactly`, `to_contain_exactly_in_any_order`,
  `to_start_with_elements`, `to_end_with_elements`, `to_be_sorted`
- **Sets:** `HashSet` / `BTreeSet` matchers (`to_contain`, subset/superset, length, empty)
- **Paths/filesystem metadata:** `to_exist`, `to_not_exist`, `to_be_file`, `to_be_dir`,
  `to_have_extension`, `to_have_file_name`
- **Process/CLI output:** `ExitStatus` / `Output` matchers (`to_exit_successfully`,
  `to_have_exit_code`, `to_have_stdout_containing`, `to_have_stderr_containing`)
- **Errors:** match `Display` output and traverse `std::error::Error::source` chains
- **Option/Result inner matching:** predicate-based and matcher-based (`..._and`, `..._matching`)
- **String quality-of-life:** `to_be_empty` / `to_not_be_empty`, `to_have_char_count`,
  optional case/whitespace normalization matchers
- **Float shape:** `to_be_nan`, `to_be_finite`, `to_be_infinite`

**Why:** This is a major adoption lever. Competitor assertion libraries treat
these as baseline because they reduce boilerplate and improve diagnostics:

- googletest-rust ships rich container matchers (ordered/unordered) plus float
  shape and string matchers ([gtest-matchers]).
- speculoos has dedicated assertion modules for `path`, `hashset`, `iter`,
  `numeric`, and `json` ([speculoos-modules]).
- hamcrest2 includes path-exists matchers and collection "contains exactly/in
  order" matchers ([hamcrest2]).

Without these, teams often adopt a "best-of-breed stack" (rstest + a matcher
crate + nextest) and only consider behave for the DSL.

[gtest-matchers]: https://docs.rs/googletest/latest/googletest/matchers/index.html
[speculoos-modules]: https://docs.rs/speculoos/latest/speculoos/
[hamcrest2]: https://docs.rs/hamcrest2/latest/hamcrest2/

**Implementation:** Add new matcher modules behind sensible feature flags:

- `std` (default): path, set, error-chain matchers
- `serde` (optional): JSON pointer / partial-field matchers for `serde_json::Value`

Design requirements:
- failure messages must be audited and consistent with existing `MatchError`
  formatting
- large-value truncation applies to collection/JSON outputs (ties into v0.9 audit)

**Effort:** Large (multiple matcher families + error message formatting + tests).

#### Feature: Optional domain matcher packs (feature-flagged or companion crates)

**What:** Offer domain-focused matcher packs that users can opt into without
forcing every dependency into the core crate.

These should target *high-frequency integration-test domains* where the default
Rust assertion surface is painfully low-level.

**Candidate packs:**

- **HTTP** (`http` feature or `behave-http` crate): status/header matchers for
  `http::Response<_>`, plus JSON-body helpers when combined with `serde`.
- **URLs** (`url` feature or `behave-url`): assert query parameters, host/path
  normalization, and scheme/port rules.
- **UUIDs** (`uuid` feature or `behave-uuid`): version/variant + parse validity.
- **SemVer** (`semver` feature or `behave-semver`): `Version` comparisons and
  requirement satisfaction (`VersionReq`).
- **Tracing logs** (`tracing` feature or `behave-tracing`): capture spans/events
  and assert that expected events were emitted (useful for observability-heavy code).

**Why:** This avoids bloat while still meeting "real suite" needs. It also
keeps behave honest about its identity: core matchers should stay generic;
domain packs are optional.

**Implementation options:**
- **Feature flags in behave** for small, MSRV-compatible deps.
- **Companion crates** (recommended when dependencies are large or MSRV-risky),
  implemented via `BehaveMatch<T>` and/or extension traits.

**Constraints:**
- behave currently builds **all features on docs.rs**. Any new optional feature
  must either keep the crate's MSRV intact or live in a companion crate so docs
  keep building without forcing a toolchain bump.

**Effort:** Medium (API design + packaging + docs; per-pack varies).

---

### v0.9.0 — "Polish & Robustness"

**Theme:** Production-ready quality. Every edge case handled, every error
message perfected, every integration documented.

#### Feature: Failure output audit

**What:** Systematically review and improve every matcher's failure message.

For every matcher:
1. Test with intentionally wrong values
2. Ensure output includes: actual value, expected value, matcher description
3. With `color` feature: colored diff for multi-line values
4. Size guard: truncate values >10KB in output (pretty_assertions'
   [96.5 GB allocation][pa-oom] is the cautionary tale)
5. Nested combinator failures: indented, numbered sub-failures

**Why:** This is the single most important quality investment. Go developers
argue the [biggest value of a test is when it fails][go-fail]. pytest's
assertion rewriting succeeds because it shows exactly what went wrong. behave's
matchers are already good — this pass makes them excellent.

[go-fail]: https://www.alexedwards.net/blog/the-9-go-test-assertions-i-use

**Effort:** Medium (systematic audit, no new features).

#### Feature: Compile-time budget

**What:** Add benchmarks measuring `behave!` macro expansion overhead.

Measure:
- 10 tests, 100 tests, 500 tests, 1000 tests
- Incremental recompilation after changing one test
- Compare against equivalent `#[test]` functions (baseline)

Set a budget:
- <500ms incremental for 100 tests
- <2s incremental for 500 tests
- <10% overhead vs bare `#[test]` baseline

If over budget, optimize codegen:
- Reduce generated code size
- Factor common code into runtime helpers called from generated tests
- Audit `syn` usage in the proc macro for unnecessary parsing

**Why:** C++ doctest targets <20ms. rstest users report minutes-long builds for
large suites. Compile time is existential for Rust testing frameworks.

**Effort:** Medium (benchmarking infrastructure, potential codegen optimization).

#### Feature: nextest integration documentation

**What:** Dedicated section in USER_GUIDE.md showing that behave tests work
with `cargo-nextest` out of the box.

```bash
cargo nextest run                       # all behave tests work
cargo nextest run -E 'test(~checkout)'  # filter by name
```

Document:
- behave tests are standard `#[test]` functions — nextest just works
- nextest's process-per-test isolation is complementary to behave's per-test
  setup/teardown
- `cargo behave` and `cargo nextest` serve different purposes: behave provides
  BDD tree output and history-based flaky detection; nextest provides process
  isolation and retry-based flaky detection

**Why:** nextest is widely adopted in CI. Documenting compatibility signals
maturity and avoids the perception that behave requires its own runner.

**Effort:** Small (documentation only).

#### Feature: CLI filter expressions

**What:** `cargo behave --filter` supports structured expressions beyond name
matching.

```bash
cargo behave --filter 'tag(slow) and not tag(flaky)'
cargo behave --filter 'status(failed-last-run)'
cargo behave --filter 'name(~checkout) and tag(integration)'
```

**Why:** nextest's [filter expressions][nextest-filter] are the model. Simple
name matching is insufficient for large test suites with tags. This completes
the tag system introduced in v0.7.

[nextest-filter]: https://nexte.st/docs/filtersets/

**Implementation:**
- Small expression parser in the CLI
- Operators: `and`, `or`, `not`, parentheses
- Predicates: `tag(x)`, `name(~pattern)`, `status(passed|failed|flaky|skipped)`
- Status predicates use the history file

**Effort:** Medium (expression parser, integration with tag system).

#### Feature: CLI retry on failure

**What:** `cargo behave --retry N` re-runs failed tests up to N times.

```bash
cargo behave --retry 3                  # retry failures up to 3 times
cargo behave --retry 2 --tag integration
```

If a test passes on retry, mark it as potentially flaky in the history.
Complement the existing history-based detection (across runs) with per-run
retry detection.

**Why:** nextest has `--retries`. pytest-rerunfailures is popular. But
[retries should diagnose, not mask][retry-caveat] — surface retried tests
prominently in output, never silently hide them.

[rsspec-retries]: https://docs.rs/rsspec
[test-casing]: https://docs.rs/test-casing

**Competitor evidence:**
- rsspec exposes retries/timeouts as first-class suite options ([rsspec][rsspec-retries])
- test-casing provides retry/timeouts as Rust test "decorators" ([test-casing][test-casing])

[retry-caveat]: https://blog.codepipes.com/testing/software-testing-antipatterns.html

**Effort:** Medium (test re-invocation, output aggregation).

---

### v0.10.0 — "Docs & Adoption"

**Theme:** Lower every barrier to adoption. Make behave discoverable and
approachable. The best features in the world do not matter if nobody knows
they exist.

#### Feature: Migration guides

**What:** Dedicated guides showing how to migrate from common Rust testing
stacks to behave. Lower the switching cost — the single biggest predictor
of framework adoption.

Guides:
- **From `assert_eq!`:** side-by-side comparison, incremental adoption path
  (use matchers first, graduate to DSL later)
- **From rstest:** how to convert `#[fixture]` + `#[case]` to `setup` + `each`
- **From test-case:** how to convert `#[test_case]` to `each` / `matrix`
- **From googletest:** matcher equivalence table (`assert_that!` → `expect!`,
  `expect_that!` → `SoftErrors`, `verify_that!` → `expect!` with `?`)

**Why:** Vitest won partly by being Jest-compatible. pytest won partly by
supporting unittest tests. Framework adoption is driven by switching cost.

**Effort:** Medium (documentation, no code changes).

#### Feature: `cargo behave init` scaffolding

**What:** A CLI command that sets up a new project for behave:

```bash
cargo behave init
# → Adds behave to [dev-dependencies] in Cargo.toml
# → Creates tests/suite.rs with a starter behave! block
# → Optionally creates behave.toml with sensible defaults
# → Prints "Run `cargo test` to see your first behave test"
```

**Why:** Jest's zero-config setup was a key adoption driver. Every second
of friction during first use is a potential drop-off.

**Effort:** Small (CLI command, template files).

#### Feature: `--output github` for CI annotations

**What:** When running in GitHub Actions, emit `::error file=...,line=...::message`
workflow commands that annotate failures directly in the PR diff view.

```bash
cargo behave --output github
# Emits: ::error file=src/auth.rs,line=42::expected: to equal "admin", actual: "guest"
```

Also generate a Job Summary with a collapsible test results table (pass/fail/
skip/flaky counts) using `$GITHUB_STEP_SUMMARY`.

**Why:** The current pipeline (cargo test → JUnit XML → third-party action →
annotations) requires 3 tools. Direct annotation eliminates the entire chain.
Developers review PRs in the GitHub UI — seeing failures inline with code
changes saves minutes per failure and eliminates context switching.

**Competitor evidence:**
- dorny/test-reporter has 1.5K+ stars but requires JUnit parsing
- ctrf-io/github-test-reporter exists for the same reason
- GitHub has a [long-standing feature request][gh-test-results] for native test dashboards

[gh-test-results]: https://github.com/orgs/community/discussions/163123

**Effort:** Small-Medium (workflow command format, summary markdown generation).

#### Feature: Matcher gap closure

**What:** Fill the remaining gaps in behave's matcher surface so users never need
another assertion crate. The v0.8 and v0.9 releases built the foundation; this
release closes the long tail that drives "I still need googletest for X" moments.

**Process output matchers** (behind `std`):
```rust
let output = Command::new("my-tool").arg("--version").output()?;
expect!(output).to_exit_successfully()?;
expect!(output).to_have_exit_code(0)?;
expect!(output).to_have_stdout_containing("v1.2.3")?;
expect!(output).to_have_stderr_empty()?;
```

These were planned for v0.8 but deferred. `std::process::Output` is the single
most-tested type in CLI tool projects and currently requires manual
`String::from_utf8` + `assert!` boilerplate.

**Iterator matchers** (no feature gate — core Rust):
```rust
expect!(0..5).to_yield_count(5)?;
expect!(items.iter()).to_all_yield_matching(|x| x > &0, "positive")?;
expect!(stream).to_yield_elements([1, 2, 3])?;
```

Iterators are the most generic abstraction in Rust. Having matchers that work on
any `IntoIterator` lets users test generators, custom iterators, and streaming
APIs without collecting first.

**Smart pointer pass-through** (behind `std`):
```rust
let boxed: Box<String> = Box::new("hello".into());
expect!(boxed).to_start_with("hel")?;  // auto-derefs to String matchers

let shared: Arc<Vec<i32>> = Arc::new(vec![1, 2, 3]);
expect!(shared).to_contain(&2)?;  // auto-derefs to Vec matchers
```

Blanket `Expectation` impls for `Box<T>`, `Arc<T>`, `Rc<T>`, `Cow<'_, T>` that
transparently delegate to the inner type's matchers. Eliminates `.as_ref()` and
`*` noise in assertions.

**Serde round-trip matcher** (behind `json` feature):
```rust
expect!(my_config).to_survive_json_roundtrip()?;
// Serializes to JSON, deserializes back, asserts equality
// Failure shows the diff between original and round-tripped value
```

The #1 serde-related bug: a type serializes but does not deserialize back to the
same value. This one-liner catches field renames, missing defaults, and custom
deserializer bugs.

**Byte/binary matchers** (behind `std`):
```rust
expect!(payload).to_start_with_bytes(&[0xFF, 0xD8])?;  // JPEG magic
expect!(buffer).to_have_byte_length(1024)?;
```

Useful for protocol testing, file format validation, and crypto output checks.

**Why all of these matter:** The matcher audit identified that behave has 89
matchers across 17 type categories — but `process::Output`, iterators, smart
pointers, and binary data are the four most common "escape hatches" where users
fall back to raw `assert!`. Closing these gaps eliminates the last reason to
reach for a separate assertion crate.

**Tests:**

Process output matchers:
- `to_exit_successfully` passes for exit code 0, fails for non-zero
- `to_exit_successfully` failure message shows actual exit code and stderr
- `to_have_exit_code(n)` matches exact code; fails with "expected: 0, actual: 1"
- `to_have_stdout_containing(s)` matches substring in stdout; fails when absent
- `to_have_stdout_containing` handles non-UTF-8 stdout gracefully (lossy conversion)
- `to_have_stderr_empty` passes when stderr is empty; fails showing stderr content
- `to_have_stderr_containing(s)` matches substring in stderr
- All process matchers work with `.not()` negation
- Failure messages include both stdout and stderr for context

Iterator matchers:
- `to_yield_count(n)` passes for exact count; consumes the iterator
- `to_yield_count` works on `Range`, `Vec::iter()`, `HashMap::keys()`, custom iterators
- `to_yield_count` failure shows "expected: 3 elements, actual: 5 elements"
- `to_all_yield_matching(f, desc)` passes when all elements satisfy predicate
- `to_all_yield_matching` short-circuits on first failure; message includes the failing element
- `to_all_yield_matching` passes for empty iterator (vacuous truth)
- `to_yield_elements([...])` passes when iterator produces exact sequence
- `to_yield_elements` failure shows diff between expected and actual sequences
- Iterator matchers do not require `Clone` on the iterator (consume once)

Smart pointer pass-through:
- `Box<String>` supports all `String` matchers (`to_start_with`, `to_contain`, etc.)
- `Arc<Vec<i32>>` supports all `Vec` matchers (`to_contain`, `to_be_empty`, etc.)
- `Rc<i32>` supports ordering matchers (`to_be_greater_than`, etc.)
- `Cow<str>` supports string matchers in both `Borrowed` and `Owned` variants
- Nested smart pointers: `Box<Arc<String>>` auto-derefs through both layers
- `.not()` negation works through smart pointer wrappers
- Failure messages show the inner value, not the wrapper type

Serde round-trip:
- `to_survive_json_roundtrip` passes for types with symmetric ser/de
- `to_survive_json_roundtrip` fails when `#[serde(skip)]` field is lost
- `to_survive_json_roundtrip` fails when `#[serde(rename)]` breaks round-trip
- Failure message shows diff between original and round-tripped value
- Works on any `T: Serialize + DeserializeOwned + PartialEq + Debug`
- Compile error (not runtime) when `T` does not implement required traits

Byte/binary matchers:
- `to_start_with_bytes(&[u8])` matches prefix; fails showing hex of first N bytes
- `to_start_with_bytes` on empty slice passes for any input
- `to_have_byte_length(n)` matches exact length; fails with "expected: 1024, actual: 512"
- Both work on `Vec<u8>`, `&[u8]`, and `Bytes` (if behind a feature flag)
- `.not()` negation works on all byte matchers

**Effort:** Medium (multiple small matcher families, each following established
patterns).

#### Feature: VS Code snippets and DSL reference

**What:** Ship a `.vscode/behave.code-snippets` file and expand the `behave!`
doc comment into a comprehensive DSL keyword reference.

Snippets:
- `btest` → skeleton test inside a group
- `bgroup` → skeleton group with setup
- `beach` → `each` block with named cases
- `bmatrix` → `matrix` block
- `bsetup` → `setup { }` block
- `bteardown` → `teardown { }` block

Doc comment expansion: the `behave!` proc macro doc comment should be a
self-contained quick reference listing every keyword (`setup`, `teardown`,
`each`, `matrix`, `each_type`, `focus`, `pending`, `xfail`, `tag`, `timeout`,
`tokio;`, `paused_time;`), each with a 3-line example.

**Why:** Autocompletion does not work inside proc macros. When a user types
inside `behave! { ... }`, rust-analyzer cannot suggest DSL keywords because
they are not Rust identifiers. Snippets and rich doc comments are the only
compensation. The `expect!()` chain works perfectly in IDEs because it
returns `Expectation<T>` with real methods — but the structural DSL has
zero discoverability.

**Tests:**
- Each snippet expands to syntactically valid `behave!` content
- Doc comment examples are extracted and compiled as doctests

**Effort:** Small (snippets file, doc comment expansion).

#### Feature: README and docs overhaul

**What:** Comprehensive README with:
- Quick-start example (copy-paste to working test in 30 seconds)
- Feature comparison table vs rstest + googletest + pretty_assertions
- "Why behave?" section targeting the fragmented-stack problem
- Matcher cheat sheet (organized by type, with one-liner examples)
- Link to full documentation on docs.rs
- Badges (crates.io, docs.rs, CI, MSRV)
- Editor configuration section (VS Code, Neovim, Helix) for optimal
  behave DX — following [Leptos's editor DX page][leptos-dx] pattern
- "AI-friendly" callout: explain why `expect!(x).to_verb(y)?` is
  easier for Copilot/Claude to learn than scattered `assert!` macros

[leptos-dx]: https://book.leptos.dev/getting_started/leptos_dx.html

**Why:** The README is the landing page. 90% of crate evaluations happen there.

**Tests:**
- All code examples in README compile (extracted as doctests or CI-checked)
- Matcher cheat sheet covers every public matcher method
- Feature comparison table is factually accurate vs latest rstest/googletest

**Effort:** Medium (writing, no code).

#### Feature: nextest compatibility documentation

**What:** Dedicated section showing that behave tests work with `cargo-nextest`
out of the box:

```bash
cargo nextest run                       # all behave tests work
cargo nextest run -E 'test(~checkout)'  # filter by name
```

Document:
- behave tests are standard `#[test]` functions — nextest just works
- nextest's process-per-test isolation is complementary to behave's per-test
  setup/teardown
- `cargo behave` and `cargo nextest` serve different purposes: behave provides
  BDD tree output and history-based flaky detection; nextest provides process
  isolation and retry-based flaky detection

Also document the compile-time advantage: one `behave!` block = one test
binary, avoiding the N-binary integration test problem.

**Why:** nextest is widely adopted in CI. Compatibility signals maturity.

**Effort:** Small (documentation only).

#### Feature: DX quality fixes (audit)

**What:** Address all 34 bugs found during the codebase DX audit. These are
quality issues across error messages, API ergonomics, code duplication, CLI
behavior, and test coverage that should be fixed before v1.0 for a polished
developer experience.

Organized into six categories:

---

**Category 1: Error quality (6 fixes)**

**Fix 1 — Truncation message hardcodes "10KB" (`src/error.rs:108`)**

The `truncate_value` helper says `[truncated at 10KB, total N bytes]` but the
threshold is a constant. If the constant changes, the message becomes a lie.

Fix: derive the label from the constant (`format!("{}", LIMIT / 1024)`).

Tests:
- Truncation message matches actual limit when `TRUNCATION_LIMIT` is 10240
- Change limit to 20480 → message says "20KB" (compile-time or cfg-test override)
- Non-truncated values: no suffix appended

**Fix 2 — `MatchError::new` takes owned Strings (`src/error.rs:68`)**

`pub fn new(description: String, actual: String, expected: String)` forces
allocation even for static messages like `"to be true"`. Change to
`impl Into<Cow<'static, str>>` or `impl Into<String>`.

Fix: accept `impl Into<Cow<'static, str>>` for all three fields. This is
backward-compatible since `String` converts to `Cow::Owned`.

Tests:
- `MatchError::new("desc", "actual", "expected")` works with `&str` (no alloc)
- `MatchError::new(format!("..."), ...)` works with owned `String`
- `Cow::Borrowed` round-trips through `Display`

**Fix 3 — `expect_panic!` swallows panic payload (`src/lib.rs:283-298`)**

`expect_panic!` catches the panic but does not include the panic message in
the success or failure output. When the test fails (no panic occurred), the
error message is generic.

Fix: on success, store the payload for debugging. On failure (`expect_no_panic!`
case), include the panic message in `MatchError`.

Tests:
- `expect_panic!(panic!("boom"))` succeeds and panic message is accessible
- `expect_no_panic!(panic!("boom"))` failure includes `"boom"` in error output
- `expect_panic!(|| {})` failure message says "expected panic but none occurred"
- `expect_panic!` with non-string payload (e.g., `panic!(42)`) shows `"Box<dyn Any>"`

**Fix 4 — `expect_match!` no truncation on Debug output (`src/lib.rs:368`)**

`expect_match!` formats the value with `Debug` but does not apply the 10KB
truncation that `MatchError` applies elsewhere.

Fix: use `truncate_value` on the `Debug` output.

Tests:
- `expect_match!` on a value with >10KB Debug output truncates the message
- Small values: no truncation applied
- Truncated output includes the `[truncated at ...]` suffix

**Fix 5 — `to_satisfy` takes `&str` but `check` takes `impl Display` (`src/expectation.rs`)**

Inconsistent API: `to_satisfy(predicate, &str)` requires a string literal,
while other APIs accept `impl Display`. Change to `impl Display` or
`impl Into<Cow<'static, str>>` for consistency.

Fix: change `&str` to `impl Into<Cow<'static, str>>`.

Tests:
- `to_satisfy(|x| x > &0, "positive")` still works (backward compat)
- `to_satisfy(|x| x > &0, format!("greater than {}", 0))` now accepted
- Negated: `.not().to_satisfy(...)` displays description correctly

**Fix 6 — Double-negation `.not().not()` no warning (`src/expectation.rs`)**

`.not().not()` silently becomes a no-op, which is almost certainly a bug.

Fix: add a `#[must_use = "double negation is likely a mistake"]` attribute,
or track negation count and emit a compile-time or runtime diagnostic.

Tests:
- `.not().not()` produces a deprecation warning or doc-level guidance
- Single `.not()` behaves as before
- `.not()` correctly inverts the matcher result

---

**Category 2: API ergonomics (3 fixes)**

**Fix 7 — No `into_value()` on `Expectation` (`src/expectation.rs`)**

`Expectation<T>` wraps a value but there is no way to extract it after
assertions. Add `into_value(self) -> T` for chaining with other logic.

Fix: add `pub fn into_value(self) -> T` method.

Tests:
- `let v = expect!(42).into_value(); assert_eq!(v, 42);` works
- `expect!(vec![1,2,3]).into_value().len()` returns 3
- Works after `.not()` — returns the original value regardless of negation state

**Fix 8 — `BehaveMatch::description` returns `&str` not `Cow` (`src/custom.rs:41`)**

The `BehaveMatch` trait's `description` method returns `&str`, forcing
implementors to allocate a `String` and then borrow it (or use a field).
Change to `Cow<'_, str>` so static descriptions are zero-cost and dynamic
descriptions can return owned strings.

Fix: change `fn description(&self) -> &str` to `fn description(&self) -> Cow<'_, str>`.

Tests:
- Custom matcher returning `Cow::Borrowed("static desc")` works
- Custom matcher returning `Cow::Owned(format!("dynamic {}", x))` works
- All built-in matchers compile with the new signature (blanket compatibility)
- `clippy::unnecessary_literal_bound` warning in smoke.rs resolved

**Fix 9 — `smoke.rs` clippy `unnecessary_literal_bound` (`tests/smoke.rs`)**

The `clippy::unnecessary_literal_bound` lint fires because custom matchers in
tests return `&'static str` when `Cow` would be better. This is resolved by
Fix 8 (changing the trait to `Cow`).

Tests:
- `cargo clippy --all-features --all-targets` passes with zero warnings
- Custom matchers in smoke.rs use `Cow` return type

---

**Category 3: Code quality (3 fixes)**

**Fix 10 — ~400 lines duplicated between `Vec<T>` and `&[T]` (`src/matchers/collections.rs`)**

Nearly identical matcher impls for `Vec<T>` and `&[T]`. Factor into a shared
impl on `&[T]` with a blanket for `Vec<T>` via `AsRef<[T]>` or `Deref`.

Fix: implement matchers on `&[T]` and provide a blanket delegation from
`Expectation<Vec<T>>` that calls `.as_slice()` internally.

Tests:
- All existing collection matcher tests still pass for both `Vec<T>` and `&[T]`
- `to_contain`, `to_be_empty`, `to_have_length` work identically on both types
- `to_contain_exactly`, `to_start_with_elements` work on both types
- Negation works on both types
- No behavioral change — purely internal refactor

**Fix 11 — `to_be_empty`/`to_not_be_empty` duplicated for String/&str (`src/matchers/strings.rs`)**

Same pattern as collections: duplicated impls for `String` and `&str`.

Fix: implement on `&str` and delegate from `Expectation<String>` via `as_str()`.

Tests:
- `expect!("").to_be_empty()` passes for `&str` and `String`
- `expect!("hello").to_not_be_empty()` passes for both
- `.not().to_be_empty()` works on both types

**Fix 12 — `smoke.rs` uses `unreachable!()` in URL test (`tests/smoke.rs:1240`)**

A test uses `unreachable!()` which violates the crate's `panic = "deny"` lint.
Replace with proper error handling.

Fix: replace `unreachable!()` with `return Err(...)` or restructure the test.

Tests:
- `cargo clippy --all-features --all-targets -- -D warnings` passes
- URL test still exercises the intended code path

---

**Category 4: CLI quality (9 fixes)**

**Fix 13 — `humanize` double underscores → double spaces (`src/cli/render.rs:108`)**

The `humanize` function that converts module paths to display names turns `__`
into two spaces instead of one, producing "auth  login" from "auth__login".

Fix: replace `__` with a single space, or handle consecutive underscores.

Tests:
- `humanize("auth__login")` → `"auth login"` (single space)
- `humanize("a_b")` → `"a b"` (single underscore → single space)
- `humanize("a___b")` → `"a b"` (triple underscore → single space)
- `humanize("__leading")` → `"leading"` (trim leading)

**Fix 14 — Group nodes no outcome rollup coloring (`src/cli/render.rs`)**

Group nodes in the tree output are always white/default, even when all children
failed. They should reflect the aggregate outcome.

Fix: walk children and color the group red if any child failed, yellow if any
skipped/flaky (but none failed), green if all passed.

Tests:
- Group with all-pass children renders in green/default
- Group with any failed child renders in red
- Group with skip but no fail renders in yellow
- Nested groups: parent reflects worst child outcome
- Empty group: default color

**Fix 15 — JUnit `<testcase>` no `time` attribute (`src/cli/output.rs:183-188`)**

JUnit XML `<testcase>` elements lack the `time` attribute. Many CI tools
(Jenkins, GitLab) parse this for timing reports.

Fix: parse timing from cargo test output (if available via `--show-output`)
and include `time="0.001"` in each `<testcase>`.

Tests:
- JUnit output includes `time` attribute on every `<testcase>`
- `time` value is a positive decimal (seconds with milliseconds)
- If timing unavailable, `time="0"` as fallback
- `<testsuite>` `time` attribute sums all `<testcase>` times

**Fix 16 — JUnit `escape_xml` missing control characters (`src/cli/output.rs:260-275`)**

The XML escaper handles `&`, `<`, `>`, `'`, `"` but not control characters
(0x00-0x1F except 0x09, 0x0A, 0x0D) which are illegal in XML 1.0.

Fix: strip or escape control characters to produce valid XML.

Tests:
- String containing `\x00` through `\x08` → characters removed
- `\x09` (tab), `\x0A` (newline), `\x0D` (CR) → preserved
- `\x0B`, `\x0C`, `\x0E`-`\x1F` → removed
- Output passes XML validation (well-formed check)

**Fix 17 — JSON output exposes `__TAG_`/`__FOCUS__` prefixes (`src/cli/output.rs`)**

JSON report output includes raw `__TAG_xxx__` and `__FOCUS__` internal prefixes
in test names instead of stripping them like the tree renderer does.

Fix: apply the same prefix-stripping logic used in tree rendering to JSON output.

Tests:
- JSON test name for `__TAG_slow____FOCUS__auth::login` → `"auth::login"`
- JSON includes a separate `"tags": ["slow"]` array
- JSON includes `"focused": true` boolean
- Round-trip: parse JSON → all names are human-readable

**Fix 18 — `parser.rs` doesn't capture failure messages (`src/cli/parser.rs`)**

The test output parser captures pass/fail status but not the actual failure
message from cargo test output. This means `--output json` and `--output junit`
cannot include failure details.

Fix: capture the failure message block (lines between `---- test_name ----`
and the next test separator) and store in `TestResult`.

Tests:
- Parse cargo test output with a failure → `TestResult.failure_message` populated
- Parse output with multiple failures → each test has its own message
- Parse output with no failures → `failure_message` is `None`
- JUnit `<failure>` element includes the captured message
- JSON `"failure"` field includes the captured message

**Fix 19 — `--retry` regex metacharacter escaping (`src/bin/cargo-behave.rs:255`)**

When `--retry` constructs a regex to filter failed tests for re-run, test names
containing regex metacharacters (`[`, `]`, `(`, `)`) are not escaped, causing
regex compilation failures.

Fix: use `regex::escape()` on test names before building the filter pattern.

Tests:
- Retry with test name containing `[0]` (from parameterized tests) works
- Retry with test name containing `(` and `)` works
- Normal test names (no metacharacters) still work

**Fix 20 — `--no-color` and `NO_COLOR` not coordinated (`src/bin/cargo-behave.rs`)**

The CLI has `--no-color` flag and checks `NO_COLOR` env var, but they use
different code paths and may not be consistent.

Fix: unify into a single `color_enabled()` check that respects: (1) `--no-color`
flag, (2) `NO_COLOR` env var, (3) terminal detection (`isatty`).

Tests:
- `--no-color` flag disables color regardless of terminal
- `NO_COLOR=1` env var disables color
- Both set: no color (consistent)
- Neither set + TTY: color enabled
- Neither set + pipe: color disabled

**Fix 21 — `run_watch` swallows all errors (`src/bin/cargo-behave.rs:116-120`)**

The watch mode loop catches all errors silently, which means file watcher
failures, permission errors, etc. are invisible to the user.

Fix: log errors to stderr and only swallow expected transient errors (like
debounce conflicts).

Tests:
- Watch mode error on invalid path → error printed to stderr
- Watch mode survives transient notify errors (debounce)
- Watch mode exit on fatal error (e.g., filesystem unmounted) with message

---

**Category 5: CLI error quality (4 fixes)**

**Fix 22 — `OutputParse` dead code variant (`src/cli/error.rs`)**

The `CliError::OutputParse` variant exists but is never constructed.

Fix: either use it (in the parser when output is malformed) or remove it.

Tests:
- If kept: malformed cargo test output produces `OutputParse` error
- If removed: `cargo clippy` passes with no dead code warning

**Fix 23 — Error messages don't suggest fixes (`src/cli/error.rs`)**

CLI errors say what went wrong but not how to fix it. For example, "config
file not found" should suggest "run `cargo behave init` to create one".

Fix: add a `suggestion` field to `CliError` variants and display it.

Tests:
- Config parse error → suggests checking TOML syntax
- Missing `cargo test` → suggests installing Rust toolchain
- Invalid `--output` format → suggests valid formats
- Filter parse error → shows syntax hint (see also Fix 31)

**Fix 24 — `history.rs` serde errors lose source type (`src/cli/history.rs:189-191`)**

`serde_json` errors are wrapped into `CliError` losing the original error's
`.source()` chain.

Fix: store the original error via `#[source]` or manual `Error::source()` impl.

Tests:
- Corrupted history JSON → error message includes serde_json details
- `Error::source()` returns the inner `serde_json::Error`
- Error display chain: `"failed to parse history: invalid type at line 3"`

**Fix 25 — `--help` no usage examples (`src/bin/cargo-behave.rs`)**

`cargo behave --help` shows flags but no examples of common workflows.

Fix: add `after_help` in clap with common usage patterns.

Tests:
- `cargo behave --help` output contains "Examples:" section
- Examples include: basic run, tag filtering, retry, output formats
- Examples are syntactically correct (parseable by clap)

---

**Category 6: Codegen & filter quality (5 fixes)**

**Fix 26 — `xfail` catches `Err` but not panics (`macros/src/codegen.rs:127-156`)**

`xfail` tests pass when the body returns `Err`, but a panic propagates as a
real failure. Expected-to-panic tests should also be caught.

Note: this is also tracked as a standalone feature in v0.14 with a broader
scope. The v0.10 fix wraps the xfail body in `std::panic::catch_unwind`.

Fix: wrap the test body in `catch_unwind`, treat `Err(panic)` as "expected
failure" (same as `Err` from matchers).

Tests:
- `xfail` test that panics → passes (expected failure)
- `xfail` test that returns `Err` → passes (existing behavior)
- `xfail` test that returns `Ok` → fails (unexpected pass, existing behavior)
- `xfail` test that panics with a message → pass, message captured
- `xfail` + `timeout`: timeout before panic → timeout error, not xfail pass

**Fix 27 — `gen_sync_teardown_timeout` loses panic message (`macros/src/codegen.rs:370-373`)**

When a sync test with timeout panics, the generated code uses `recv_timeout`
which loses the original panic payload/message.

Fix: capture the `JoinHandle` result and extract the panic payload for display.

Tests:
- Sync test + timeout that panics → failure message includes panic payload
- Sync test + timeout that completes → no change in behavior
- Sync test + timeout that exceeds deadline → timeout message shown

**Fix 28 — Matrix test names are positional only (`macros/src/codegen.rs:583-599`)**

Matrix tests generate `case_0_0`, `case_0_1`, etc., which are meaningless.
Unlike `each` which supports named cases, matrix has no naming mechanism.

Fix: support optional string labels per dimension, falling back to indices.

Tests:
- `matrix ["small", "large"] x ["fast", "slow"]` → `small_fast`, `small_slow`, etc.
- `matrix [1, 2] x [3, 4]` → `case_0_0`, `case_0_1` (current behavior preserved)
- Mixed: `matrix ["small", "large"] x [1, 2]` → `small_0`, `small_1`
- Labels are slugified (spaces → underscores, special chars removed)

**Fix 29 — Filter: no escaping for special chars (`src/cli/filter.rs`)**

Tag and name values in filter expressions are not escaped, so
`tag(c++)` or `name(test[0])` causes parse errors.

Fix: support quoted values: `tag("c++")`, `name("test[0]")`.

Tests:
- `tag("c++")` matches tests tagged `c++`
- `name("test[0]")` matches test path containing `test[0]`
- Unquoted values still work for simple alphanumeric names
- Mismatched quotes produce a clear parse error

**Fix 30 — Filter: no syntax hint on parse error (`src/cli/filter.rs`)**

When a filter expression fails to parse, the error is generic with no hint
about valid syntax.

Fix: include a brief syntax reference in the error message.

Tests:
- Invalid filter `tag(` → error includes "expected closing parenthesis"
- Invalid filter `and tag(x)` → error includes "expected expression before 'and'"
- Error includes "syntax: tag(name) and/or/not name(pattern)"

---

**Category 7: Test coverage (3 fixes)**

**Fix 31 — No negative test cases for error messages (`tests/smoke.rs`)**

Integration tests only check that matchers pass. There are no tests verifying
the content of failure messages — the most important DX surface.

Fix: add tests that intentionally fail and assert on the error message content.

Tests:
- `to_equal` failure message contains both actual and expected values
- `to_contain` failure message shows the collection and missing element
- `to_be_true` failure message shows `"actual: false"`
- `to_match_regex` failure includes the regex pattern that failed
- Negated matcher failures: `.not().to_equal(5)` with value 5 shows proper message
- Truncation: large value failure message is truncated with suffix
- At least 20 negative test cases covering the most common matchers

**Fix 32 — `atty_stdout` naming (`src/bin/cargo-behave.rs:496-498`)**

The function is named `atty_stdout` but the crate does not use the `atty`
crate — it uses `crossterm`. Rename to `stdout_is_terminal` or similar.

Fix: rename to `is_terminal_stdout` or `stdout_is_tty`.

Tests:
- Function compiles and works correctly (existing behavior)
- Name matches the implementation (crossterm-based detection)

**Fix 33 — No `smoke.rs` negative tests for `smoke.rs` URL matcher (`tests/smoke.rs`)**

URL matcher tests only check the happy path. Add negative cases.

Tests:
- `to_have_scheme("https")` fails on `http://example.com` with proper message
- `to_have_host("example.com")` fails on `other.com` with proper message
- `to_have_query_param("missing")` fails with "expected: to have query param"

**Fix 34 — `expect!` / `expect_match!` no source location (`src/lib.rs`)**

Failed assertions don't include `file:line` in the error output. Users must
read the full backtrace to find which assertion failed.

Note: source location is also part of the v0.11 expression decomposition
feature. The v0.10 fix adds basic `file!()` / `line!()` capture to
`MatchError` without the full decomposition.

Fix: `expect!` macro captures `file!()` and `line!()` and passes them
to `MatchError`. Display includes `at: file:line` footer.

Tests:
- Failed `expect!(x).to_equal(y)` error includes `at: tests/foo.rs:42`
- Failed `expect_match!` error includes source location
- Failed `expect_panic!` error includes source location
- `SoftErrors` individual failures each include their own source location
- Source location does not appear in passing test output (zero overhead)

---

**Cross-cutting notes:**

Several v0.10 DX fixes overlap with features planned in later versions:
- **Fix 26** (xfail panics) is the minimal fix; v0.14 expands this with
  richer panic payload handling and `xfail "reason"` syntax
- **Fix 34** (source location) is the foundation; v0.11 builds full
  expression decomposition on top of it
- **Fixes 9, 10, 11** (collection/string dedup, to_all_satisfy detail) lay
  the groundwork for v0.11's improved collection failure messages

**Effort:** Large (34 fixes across 15 files; most individually small but
collectively significant).

---

### v0.11.0 — "Smart Failures"

**Theme:** Make failure output so good that developers never need to add
`println!` or re-run a test to understand what went wrong. In Rust, where
every recompilation costs seconds to minutes, eliminating a single
"add debug output → recompile → re-run" cycle is worth more than any
new matcher.

#### Feature: Expression decomposition in `expect!` (Swift `#expect`-style)

**What:** When an `expect!` assertion fails, automatically decompose the
expression and show the evaluated value of every sub-expression. This is
the single most impactful DX feature across all testing frameworks.

Today:
```
expect!(score)
  actual: 42
expected: to equal 100
```

Goal:
```
expect!(user.age >= 18)
  Expectation failed: (user.age → 15) >= 18
  at: tests/auth.rs:42
```

```
expect!(items).to_contain(target)?
  actual: [1, 2, 3]
expected: to contain 5
  where: items = [1, 2, 3]
         target = 5
  at: tests/checkout.rs:87
```

For combinator failures, show which specific sub-matcher failed:
```
all_of failed (1 of 3 matchers):
  ✓ to be greater than 0
  ✓ to be less than 100
  ✗ to be even
    actual: 7
```

**Why:** Swift Testing's `#expect(user.age >= 18)` showing `(user.age → 15) >= 18`
is the [gold standard][swift-expect]. Catch2's `REQUIRE(a == 1)` with expansion
`3 == 1` pioneered this in C++. pytest's assertion rewriting does it by
rewriting AST at import time. Developers spend [30–50% of time debugging][debug-cost];
this eliminates the "add println → recompile → re-run" cycle that costs
minutes per iteration in Rust.

The key insight: `expect!(a + b == c)` can be parsed by a proc macro into
separate evaluation of `a`, `b`, `c`, `a + b`, and the comparison. On failure,
all sub-values are captured. On success, zero overhead — values are only
formatted when the assertion fails.

[swift-expect]: https://developer.apple.com/xcode/swift-testing/
[debug-cost]: https://thenewstack.io/how-much-time-do-developers-spend-actually-writing-code/

**Implementation:**
- Convert `expect!` from `macro_rules!` to a proc macro (or a hybrid)
- Parse the inner expression with `syn` to identify sub-expressions
- Generate code that evaluates each sub-expression into a local variable
- On failure, format all sub-values into `MatchError`
- Backward-compatible: `expect!(val).to_matcher()` chain still works
- Simple expressions (`expect!(x)`) have no additional cost
- Complex expressions (`expect!(a.method() > b.field)`) show both sides

**Tests:**

Expression decomposition:
- `expect!(a == b)` failure shows `(a → 3) == (b → 5)`
- `expect!(a > b)` failure shows `(a → 2) > (b → 10)`
- `expect!(vec.len() == 3)` failure shows `(vec.len() → 5) == 3`
- `expect!(a + b == c)` failure shows `(a + b → 7) == (c → 10)`
- `expect!(result.is_ok())` failure shows `(result → Err("...")).is_ok() → false`
- Nested field access: `expect!(user.address.city == "NYC")` shows each level
- Method chain: `expect!(s.trim().len() > 0)` shows intermediate values
- Passing assertions: zero allocation, no formatting overhead
- Non-decomposable expressions: fall back to `stringify!` (current behavior)

Source location:
- Every `MatchError` includes `file:line` from `file!()` and `line!()`
- Multi-assertion tests: each failure points to its specific line
- Negated assertions (`.not()`) include correct location

Backward compatibility:
- `expect!(val).to_equal(x)?` still works identically
- `expect!(val).not().to_equal(x)?` still works
- Existing matchers do not need changes
- `SoftErrors::check()` captures location from the check site

**Effort:** Medium-Large (proc macro expression parsing, error formatting).

#### Feature: `require!` macro (unwrap-or-fail)

**What:** A companion to `expect!` that unwraps `Option` and `Result` values,
failing the test with a clear message if the value is `None` or `Err`.

```rust
behave! {
    "user lookup" {
        "returns the correct user" {
            let user = require!(find_user(42));  // fails if None
            expect!(user.name).to_equal("Alice")?;

            let config = require!(load_config()); // fails if Err
            expect!(config.timeout).to_equal(30)?;
        }
    }
}
```

Failure output:
```
require!(find_user(42))
  expected: to be Some / Ok
  actual: None
  at: tests/user.rs:15
```

**Why:** Swift Testing's `#require` is one of its most-loved features. In
Rust, `?` handles `Result` but not `Option` in test functions (which return
`Result<(), Box<dyn Error>>`). Developers currently write
`let user = find_user(42).ok_or("user not found")?;` which is noisy and
loses the expression context. `require!` is the test-specific equivalent of
`unwrap()` without violating the "no unwrap" rule.

**Implementation:**
- `macro_rules!` macro that matches on `Option<T>` and `Result<T, E>`
- For `Option`: returns `Err(MatchError::new(...))` if `None`, unwraps if `Some`
- For `Result`: returns `Err(MatchError::new(...))` if `Err`, unwraps if `Ok`
- Captures expression text, file, and line via `stringify!`, `file!`, `line!`

**Tests:**

Option handling:
- `require!(Some(42))` returns `42`
- `require!(None::<i32>)` returns `Err` with message showing "actual: None"
- `require!(map.get("key"))` returns the value or fails with expression text
- Return type is `T`, not `Expectation<T>`

Result handling:
- `require!(Ok::<i32, String>(42))` returns `42`
- `require!(Err::<i32, String>("oops".into()))` returns `Err` with "actual: Err(\"oops\")"
- Error message includes the inner error's Display output
- Works with `?` propagation in test functions

Source location:
- Failure message includes `file:line` pointing at the `require!` call
- Multiple `require!` in one test each point to their own line

Edge cases:
- `require!(nested_option.flatten())` works (any `Option<T>`)
- `require!(async_fn().await)` works (evaluates the expression first)
- `require!` in `SoftErrors::check()` — should it be allowed? Probably not,
  since `require!` is a hard failure

**Effort:** Small (declarative macro, follows `expect!` pattern).

#### Feature: Improved collection/property failure messages

**What:** When matchers that check properties fail, show the useful
information — not just the assertion description.

Today:
```
expect!(vec)
  actual: [1, 2, 3, 4, 5, 6, 7, 8, 9]
expected: to all satisfy "to be even"
```

Goal:
```
expect!(vec)
  actual: [1, 2, 3, 4, 5, 6, 7, 8, 9]
expected: to all satisfy "to be even"
  failing: [0] = 1, [2] = 3, [4] = 5, [6] = 7, [8] = 9
```

Specific improvements:
- `to_all_satisfy`: show which elements failed and their indices
- `to_any_satisfy`: show "none of N elements matched" with first few elements
- `to_none_satisfy`: show which elements unexpectedly matched
- `to_contain_all_of`: show which expected elements are missing
- `to_have_length`: show "actual length: 5" not just the full collection
- `to_contain_exactly_in_any_order`: show missing and extra elements separately

**Why:** The DX audit found that collection matchers are the #1 source of
"I can see it failed but I have to manually figure out why" moments. When
a 50-element vector fails `to_contain_all_of`, showing which elements are
missing saves minutes of visual diffing. googletest-rust and Kotest both
show this level of detail.

**Implementation:**
- Modify each collection matcher's failure path to compute additional context
- Add a `context` field to `MatchError` (optional, only populated on failure)
- Display the context as indented "detail" lines below the main error
- Truncation still applies to the detail lines

**Tests:**

`to_all_satisfy`:
- Failure on `[1, 2, 3]` with "even" shows `failing: [0] = 1, [2] = 3`
- Failure on 100-element vec truncates after first 10 failing elements
- All-pass: no context computed (zero overhead)

`to_contain_all_of`:
- `[1, 2, 3]` missing `[4, 5]` shows `missing: [4, 5]`
- All present: no context computed

`to_have_length`:
- Failure shows `actual length: 5, expected: 3` before the collection value
- For large collections, the value itself is truncated but length is always shown

`to_contain_exactly_in_any_order`:
- `[1, 2, 3]` vs `[2, 3, 4]` shows `missing: [4], extra: [1]`
- Duplicate handling: `[1, 1, 2]` vs `[1, 2, 2]` shows `missing: [2], extra: [1]`

**Effort:** Medium (modify each collection matcher's error path).

#### Feature: Enhanced error context (expression introspection)

**What:** Beyond expression decomposition, add source location and variable
context to every `MatchError`.

```
expect!(items).to_contain(target)?
  actual: [1, 2, 3]
expected: to contain 5
  where: items = [1, 2, 3]
         target = 5
  at: tests/checkout.rs:42
```

**Implementation:**
- `expect!` macro captures `file!()` and `line!()` into `MatchError`
- Matchers that take arguments include the argument name and value
- `MatchError::Display` formats the rich context when available
- No runtime cost for passing tests (context only computed on failure)

**Effort:** Medium (macro rework, error formatting overhaul).

#### Feature: Combinator ergonomics (`all_of!` / `any_of!` macros)

**What:** Replace the verbose `Box::new(...) as Box<dyn BehaveMatch<T>>`
ceremony with helper macros:

Today:
```rust
let m = all_of(vec![
    Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
    Box::new(IsEven),
]);
```

Goal:
```rust
let m = all_of![IsPositive, IsEven];
expect!(value).to_match(any_of![to_be_greater_than(10), to_equal(0)])?;
```

**Why:** The DX audit found that combinators require excessive ceremony.
The `as Box<dyn BehaveMatch<i32>>` type annotation is only needed on the
first element for type inference, but users do not know this. A macro
hides the boxing entirely.

**Implementation:**
- `all_of!` and `any_of!` declarative macros that box each argument
- Infer `T` from the first matcher's type
- Re-export from prelude

**Tests:**

Macro usage:
- `all_of![m1, m2]` compiles and works like `all_of(vec![Box::new(m1), Box::new(m2)])`
- `any_of![m1, m2, m3]` works with 3+ matchers
- `all_of![]` empty: returns vacuous-truth matcher (same as `all_of(vec![])`)
- `any_of![]` empty: returns always-fail matcher
- Nested: `all_of![any_of![m1, m2], m3]` works
- Inline matchers: `all_of![to_be_greater_than(0), to_be_less_than(100)]` works

Type inference:
- No explicit type annotation needed
- Works with custom `BehaveMatch<T>` implementations
- Compile error if matchers have incompatible types

**Effort:** Small (declarative macros, no proc macro changes).

#### Feature: Failure detail capture in CLI parser

**What:** Extend `cargo behave`'s test output parser to capture failure
messages from `cargo test` output and include them in tree rendering,
JUnit, and JSON reports.

Today `cargo behave` shows:
```
  ✗ auth::login::rejects_expired_token
```

Goal:
```
  ✗ auth::login::rejects_expired_token
    expected: to be err
    actual: Ok(Token { id: 42 })
```

**Why:** The DX audit found this is the single biggest gap in the CLI
experience. When a test fails, the user must scroll through cargo's raw
stderr to find the error message. Every other test runner (pytest, Jest,
nextest) shows the failure inline.

**Implementation:**
- Parse `---- test_name stdout ----` sections from cargo test output
- Extract `MatchError` formatted output (look for `expect!` / `actual:` /
  `expected:` patterns)
- Store failure message in `TestResult`
- Display inline in tree output (indented under the failing test)
- Include in JUnit `<failure message="...">` element
- Include in JSON output `error_message` field

**Tests:**

Capture:
- Single-line failure: `expected: to equal 42` captured
- Multi-line failure: diff output with `+`/`-` lines captured
- Panic message: `thread panicked at` captured
- Non-behave failure: `assert_eq!` output captured
- No stdout section: no failure message (graceful degradation)

Display:
- Tree output: failure message indented 4 spaces under test name
- Long messages: truncated to 3 lines in tree, full in JUnit/JSON
- Color: failure message inherits red from parent test line

JUnit:
- `<failure message="expected: to equal 42">full output</failure>`
- Multi-line output preserved in CDATA section

JSON:
- `"error_message": "expected: to equal 42\nactual: 100"` field added

**Effort:** Medium (parser enhancement, output formatting).

#### Feature: `--slow-threshold` and test timing in output

**What:** Show per-test execution duration in tree output. Flag tests
exceeding a configurable threshold.

```bash
cargo behave --slow-threshold 500ms

  ✓ auth::login (2ms)
  ✓ auth::permissions (4ms)
  ⚠ database::migration (1.2s) [SLOW]
  ✓ parser::empty_input (0ms)

  3 passed, 0 failed, 1 slow (>500ms)
  Total: 1.21s
```

Also available as a `behave.toml` default:
```toml
[defaults]
slow_threshold_ms = 500
```

**Why:** Slow tests are invisible until they accumulate into minutes-long
CI runs. Making them visible creates pressure to fix them. nextest
surfaces slow tests; `cargo test` does not.

**Implementation:**
- Parse test duration from `cargo test` output (available in `pretty` format)
- Display duration next to each test in tree output
- Highlight tests exceeding threshold in yellow with `[SLOW]` marker
- Show total execution time in summary line
- JUnit `<testcase>` elements get `time` attribute (currently missing)

**Tests:**

Timing display:
- Tree output shows `(Nms)` next to each test name
- Tests under 1ms show `(0ms)`
- Tests over 1s show `(1.2s)` not `(1234ms)`
- Summary line includes total time: `3 passed, 0 failed (1.21s)`

Slow threshold:
- `--slow-threshold 500ms`: tests at 501ms+ show `[SLOW]` in yellow
- `--slow-threshold 0`: all tests marked slow (edge case)
- No threshold: timing shown but no `[SLOW]` marker
- `behave.toml` `slow_threshold_ms` is loaded and respected
- CLI flag overrides config file
- Slow count in summary: `1 slow (>500ms)`

JUnit integration:
- `<testcase time="0.502">` attribute added to JUnit output
- JSON output includes `duration_ms` field per test

**Effort:** Small-Medium (parser enhancement for timing, display logic).

#### Feature: OSC 8 clickable file:line links

**What:** Emit [OSC 8 hyperlinks][osc8] in terminal output so that
test locations and failure source lines are clickable in supported
terminals (iTerm2, Terminal.app, GNOME Terminal, Windows Terminal,
WezTerm, Alacritty).

```
  ✗ auth::login::rejects_expired_token
    at tests/auth.rs:87     ← clickable, opens editor at line 87
    expected: to be err
    actual: Ok(Token { ... })
```

The link format:
```
\033]8;;file:///path/to/tests/auth.rs:87\033\\tests/auth.rs:87\033]8;;\033\\
```

**Why:** Neither `cargo test` nor nextest emit clickable source links.
This is a first-mover opportunity in the Rust testing ecosystem. When a
test fails, the developer's next action is always "go to that file and
line." Clickable links eliminate the manual copy-paste-search step.
Supported by all major modern terminals.

[osc8]: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda

**Implementation:**
- Detect terminal OSC 8 support via `$TERM_PROGRAM` or `$COLORTERM`
- Wrap file:line references in OSC 8 escape sequences
- Disabled when `NO_COLOR` is set or output is not a TTY
- Works with `--output tree` (default) only; ignored in JSON/JUnit

**Tests:**

Link generation:
- `file:///abs/path/to/file.rs:42` format for local files
- Relative paths resolved to absolute via `std::env::current_dir()`
- Line number included in the URI fragment or query

Detection:
- OSC 8 emitted when `$TERM_PROGRAM` is iTerm2/WezTerm/etc.
- OSC 8 suppressed when stdout is not a TTY
- OSC 8 suppressed when `NO_COLOR` is set
- OSC 8 suppressed when `--no-color` flag is passed

Fallback:
- Non-OSC8 terminals: plain `tests/auth.rs:87` text (current behavior)
- Piped output: no escape sequences

**Effort:** Small (escape sequence wrapping, terminal detection).

#### Feature: Failure summary at end of output

**What:** After all tests complete, print a dedicated "Failures" section
that lists every failed test with its file:line and one-line error summary.

```
Failures:

  1) auth::login::rejects_expired_token
     at tests/auth.rs:87
     expected: to be err, actual: Ok(Token { ... })

  2) database::migration::handles_concurrent_writes
     at tests/db.rs:142
     expected: to equal 1, actual: 2

2 of 47 tests failed
```

**Why:** In large suites, failures scroll off the terminal. Developers
must scroll back through hundreds of lines to find them. A summary at the
end is the single most requested output feature across frameworks.

**Effort:** Small (already have the data, just formatting).

#### Feature: `--output markdown` for PR comments

**What:** Machine-readable markdown output suitable for posting as a
GitHub PR comment via CI:

```markdown
## Test Results: 46 passed, 1 failed, 2 skipped

<details><summary>Failed (1)</summary>

| Test | Location | Error |
|------|----------|-------|
| `auth::login::expired` | `tests/auth.rs:87` | expected: to be err |

</details>
```

**Why:** Teams use CI bots to post test results on PRs. Markdown output
eliminates the JUnit-to-markdown conversion step.

**Tests:**

Markdown format:
- Header includes pass/fail/skip/flaky counts
- Failed tests in a `<details>` block with table
- Passed tests in a separate collapsed `<details>` block
- Skipped tests listed with reason
- Flaky tests listed with retry count
- Empty results: "No tests found" message
- All test names have internal prefixes stripped

Table formatting:
- Pipe characters in test names or error messages are escaped
- Long error messages truncated to 100 chars with `...`
- Backtick-wrapped code elements in test name column

CI integration:
- `cargo behave --output markdown > $GITHUB_STEP_SUMMARY` works
- Output is valid GitHub-Flavored Markdown (tables, details, code spans)

**Effort:** Small (new output formatter).

#### Feature: `--output agent` for AI coding assistants

**What:** Minimal output mode optimized for LLM token efficiency. Only
failures and their complete error messages. No passing tests, no colors,
no tree structure.

```bash
cargo behave --output agent

FAIL auth::login::rejects_expired_token
  at tests/auth.rs:87
  expected: to be err
  actual: Ok(Token { id: 42, expired: true })

FAIL database::migration::concurrent_writes
  at tests/db.rs:142
  expected: to equal 1
  actual: 2

2 failed, 45 passed, 2 skipped
```

Auto-detected when running inside known AI agent environments (Claude Code,
Cursor, Copilot Workspace) via environment variables.

**Why:** Vitest introduced an [agent reporter][vitest-agent] in 2025 that
suppresses all passing test output and console logs. AI coding agents
process test output as context — every unnecessary token wastes context
window and increases cost. A developer using Claude Code to fix a test
failure needs only the failure message, not 200 lines of passing tests.

[vitest-agent]: https://github.com/vitest-dev/vitest/pull/9779

**Implementation:**
- New output format: `agent`
- Auto-detection: check `$CLAUDE_CODE`, `$CURSOR_SESSION`, `$GITHUB_COPILOT`
- Print only `FAIL` + test name + error details
- End with one-line summary
- No ANSI colors (agents parse text, not terminal escapes)
- Structured enough for AI to parse: `FAIL`, `at`, `expected`, `actual`

**Tests:**

Output content:
- Passing tests: suppressed entirely
- Failing tests: full error details with file:line
- Skipped tests: suppressed (only count in summary)
- Flaky tests: shown as FLAKY with retry info
- Summary line always present

Auto-detection:
- `$CLAUDE_CODE=1` → agent output
- `$CURSOR_SESSION` set → agent output
- No env vars + interactive terminal → default tree output
- Explicit `--output agent` overrides auto-detection
- `--output tree` overrides auto-detection even in agent environment

Format stability:
- `FAIL` prefix is always first on the line (parseable by regex)
- `at` line always follows `FAIL` line
- `expected`/`actual` always present for assertion failures
- Non-assertion failures (panics, timeouts) use `error:` instead

**Effort:** Small (new output formatter, environment detection).

---

### v0.12.0 — "Async & Integration"

**Theme:** Make async testing first-class and integrate with the broader
Rust ecosystem. Async code is the majority of production Rust — testing
it should be as ergonomic as sync code.

#### Feature: `paused_time;` DSL keyword

**What:** Generate `#[tokio::test(start_paused = true)]` for deterministic
time-dependent tests.

```rust
behave! {
    "rate limiter" {
        tokio;
        paused_time;

        "allows burst then blocks" {
            let limiter = RateLimiter::new(10, Duration::from_secs(1));
            for _ in 0..10 {
                expect!(limiter.try_acquire()).to_be_ok()?;
            }
            expect!(limiter.try_acquire()).to_be_err()?;
            tokio::time::advance(Duration::from_secs(1)).await;
            expect!(limiter.try_acquire()).to_be_ok()?;
        }
    }
}
```

**Why:** Time-dependent tests are the #1 source of async flakiness. tokio's
`start_paused` mode is powerful but most developers do not know it exists.
Making it a first-class DSL keyword surfaces this capability.

**Effort:** Small (codegen change, add attribute to generated function).

#### Feature: Stream/Future matchers

**What:** Matchers for async streams and futures behind the `tokio` feature:

```rust
expect!(stream).to_produce([1, 2, 3]).await?;
expect!(stream).to_produce_in_order([1, 2, 3]).await?;
expect!(future).to_complete_within(Duration::from_secs(5)).await?;
expect!(future).to_resolve_to(42).await?;
```

**Why:** Testing async streams is a common pain point. Developers write
ad-hoc collection loops and timeout wrappers for every project. Dedicated
matchers eliminate this boilerplate and provide clear failure messages.

**Effort:** Medium (new matcher family, async trait bounds).

#### Feature: `--runner nextest` interop mode

**What:** Delegate test execution to nextest while keeping behave's tree
rendering, tag filtering, and flaky detection.

```bash
cargo behave --runner nextest           # uses nextest for execution
cargo behave --runner nextest --tag fast --retry 2
```

Behave would:
1. Resolve tags and filters to a list of test names
2. Invoke `cargo nextest run --exact <names> --message-format json`
3. Parse nextest's structured output
4. Render using behave's tree renderer

**Why:** nextest is better at execution (process-per-test, 3x speed). behave
is better at authoring and presentation. Combining them gives users the
best of both worlds without forcing a choice.

**Effort:** Medium (nextest output parsing, command delegation).

#### Feature: `to_eventually` retry matcher

**What:** A matcher that retries an assertion with configurable timeout
and polling interval, for testing async state changes and eventual
consistency.

```rust
behave! {
    "event processing" {
        tokio;

        "order status updates within 5 seconds" {
            submit_order(&order).await;
            expect!(|| get_order_status(&order.id))
                .to_eventually(to_equal(OrderStatus::Confirmed))
                .within(Duration::from_secs(5))
                .polling_every(Duration::from_millis(100))
                .await?;
        }
    }
}
```

Also works with sync closures (polls in a loop with `thread::sleep`):
```rust
expect!(|| cache.get("key"))
    .to_eventually(to_be_some())
    .within(Duration::from_secs(2))?;
```

**Why:** Playwright's auto-waiting eliminates the #1 source of flaky E2E
tests. Vitest's `expect.poll()` brings this to unit tests. In Rust,
integration tests that wait for background tasks, message queues, or
database replication need retry logic. Today every project writes its own
`loop { if condition { break; } sleep(100ms); }` wrapper.

**Implementation:**
- `to_eventually(matcher)` takes any `BehaveMatch<T>` matcher
- `.within(duration)` sets the total timeout (default 5s)
- `.polling_every(duration)` sets the poll interval (default 100ms)
- The closure is called repeatedly until the matcher passes or timeout
- On timeout, the last failure is returned with added context showing
  the number of attempts and total elapsed time
- Async variant uses `tokio::time::sleep`; sync uses `std::thread::sleep`

**Tests:**

Basic retry:
- Value that becomes ready after 500ms passes with 5s timeout
- Value that never becomes ready fails after timeout with attempt count
- Immediate pass: closure called once, returns immediately
- Failure message: `"timed out after 5.0s (50 attempts): expected: to equal 42, last actual: 41"`

Configuration:
- Custom timeout: `within(Duration::from_millis(200))` times out faster
- Custom interval: `polling_every(Duration::from_millis(10))` polls faster
- Default timeout (5s) when `.within()` not called
- Default interval (100ms) when `.polling_every()` not called

Async:
- Works inside `tokio;` blocks with `.await`
- Uses `tokio::time::sleep` (respects `paused_time;`)
- Timeout uses `tokio::time::timeout`

Sync:
- Works without `tokio;`
- Uses `std::thread::sleep` for polling

Edge cases:
- Closure that panics: panic propagated, not retried
- Closure that takes longer than interval: next poll delayed accordingly
- Matcher that involves `.not()`: negated matching retried correctly
- Zero timeout: closure called once, behaves like a normal assertion

**Effort:** Medium (new matcher wrapper, async/sync variants, timeout logic).

#### Feature: Tracing/logging integration

**What:** A `tracing;` group keyword that initializes a tracing subscriber
scoped to the test, capturing spans and events.

```rust
behave! {
    "request handler" {
        tracing;  // captures tracing output per test

        "logs request start" {
            handle_request(&req);
            expect!(test_tracing::events())
                .to_contain_message("request started")?;
        }
    }
}
```

**Why:** `test-log` (3.1M downloads/mo) and `tracing-test` (2.4M downloads/mo)
show massive demand. Every Rust project using tracing writes per-test
subscriber setup code. A DSL keyword eliminates this boilerplate.

**Implementation:** Behind a `tracing` feature flag. Generates
`tracing_test::traced_test` or a lightweight subscriber wrapper.

**Effort:** Medium (new feature flag, codegen integration).

---

### v0.13.0 — "Test Intelligence"

**Theme:** Make behave smarter about *which* tests to run and *how* to
present results. Move from "run everything" to "run what matters."

#### Feature: `--affected` (test impact analysis)

**What:** Only run tests whose dependencies were modified since the last
commit (or a specified base ref).

```bash
cargo behave --affected                 # vs HEAD~1
cargo behave --affected --base main     # vs main branch
```

How it works:
1. Parse `cargo metadata` to get the crate dependency graph
2. Use `git diff --name-only <base>` to find changed source files
3. Map changed files → changed modules → tests that import those modules
4. Pass the filtered test list to `cargo test --exact`
5. Heuristic fallback: if `Cargo.toml`, `build.rs`, or proc-macro sources
   change, run everything

**Why:** This is the highest-ROI feature for large codebases. Gradle's
Predictive Test Selection [cuts test times by 70-90%][gradle-pts].
Spotify runs 50,000+ tests but only executes the affected subset per commit.
In Rust, where compilation is slow, skipping unaffected test binaries
entirely saves both compile time and execution time.

[gradle-pts]: https://gradle.com/develocity/product/predictive-test-selection/

**Effort:** Large (dependency graph analysis, git integration, heuristics).

#### Feature: `--failed-last` (re-run failures)

**What:** Re-run only tests that failed in the previous `cargo behave` run.

```bash
cargo behave                            # 3 failures
# fix code...
cargo behave --failed-last              # re-run only those 3
```

Uses the history file (already exists for flaky detection) to track
last-run results.

**Why:** The most common inner-loop pattern: run all → fix failures →
re-run failures → repeat. Today developers must manually copy test names.
pytest's `--lf` (last failed) is one of its most-used flags.

**Effort:** Small (history file already exists, just needs a filter).

#### Feature: Enhanced flaky detection reporting

**What:** When a test is detected as flaky (via history or retry), include
diagnostic context:

```
⚡ database::migration (FLAKY)
  Last 10 runs: ✓✓✓✓✓✗✓✓✓✗
  Source unchanged since: 2026-03-14
  Likely cause: non-deterministic (timing? shared state?)
  Suggestion: add tag "flaky" and investigate
```

**Why:** Google reports that 84% of pass-to-fail transitions are flaky, and
developers spend [2% of coding time][google-flaky] investigating them.
Surfacing the history pattern directly in test output saves the manual
investigation step.

[google-flaky]: https://testing.googleblog.com/2016/05/flaky-tests-at-google-and-how-we.html

**Effort:** Small-Medium (history data already tracked, formatting and
heuristics for "likely cause").

#### Feature: Interactive watch mode

**What:** Extend `--watch` with keyboard shortcuts for interactive
test filtering and re-running, modeled after Vitest/Jest watch mode.

```
cargo behave --watch

  ✓ auth::login (2ms)
  ✓ auth::permissions (4ms)
  ✗ database::migration (1.2s)

  2 passed, 1 failed

  Watch keys:
    a  rerun all tests
    f  rerun only failed tests
    p  filter by file pattern
    t  filter by test name pattern
    q  quit

  Press a key...
```

When the user presses `p`, an inline prompt appears:
```
  Filter by file: test_auth█
  Matching: 3 tests in 1 file
```

**Why:** Vitest's interactive watch mode is the [#1 cited DX
feature][vitest-watch-dx] that makes developers "never want to go back."
Jest pioneered this with `o` (changed files only), `p` (file filter),
`t` (test filter). Rust's compile times make the feedback loop even
more important — developers want to narrow the re-run scope without
restarting the process.

[vitest-watch-dx]: https://vitest.dev/guide/features.html

**Implementation:**
- Use `crossterm` (already a dependency behind `cli` feature) for raw
  terminal input
- Disable canonical mode to read single keypresses
- On `f`: re-run only tests that failed in the last run (uses history)
- On `p`/`t`: enter inline filter mode with live test count
- On `a`: clear filter and re-run all
- On `Ctrl-C` or `q`: clean exit, restore terminal settings
- On `Ctrl-Z` (SIGTSTP): restore terminal settings before suspending
  (nextest's approach)

**Tests:**

Keyboard handling:
- `a` triggers full re-run
- `f` triggers re-run of only previously failed tests
- `q` cleanly exits watch mode
- `Ctrl-C` cleanly exits (SIGINT handler restores terminal)
- Unknown keys: ignored, no error

Filter mode:
- `p` enters file filter: typing narrows matched tests in real-time
- `t` enters test name filter: typing narrows matched tests
- `Enter` confirms filter and re-runs matched tests
- `Escape` exits filter mode without applying
- Empty filter: runs all tests (same as `a`)
- Regex pattern: `p` with `auth.*` matches `test_auth.rs` and `tests/auth/`

Integration:
- Filter persists across re-runs until cleared with `a`
- File change during filter mode: queued, applied after filter confirmed
- `--watch` with `--tag`: tag filter AND watch filter combined
- Non-TTY: watch mode skips interactive features, just re-runs on changes

**Effort:** Medium (raw terminal input, inline prompt, state management).

#### Feature: Environment-aware output switching

**What:** Auto-detect the execution environment and switch output format
accordingly, with zero configuration.

| Environment | Detection | Default Output |
|-------------|-----------|---------------|
| Interactive terminal | `is_terminal()` | `tree` (full color) |
| GitHub Actions | `$GITHUB_ACTIONS` | `github` (annotations) |
| AI coding agent | `$CLAUDE_CODE` / `$CURSOR_SESSION` | `agent` (minimal) |
| Piped / non-TTY | `!is_terminal()` | `tree` (no color) |

```bash
cargo behave                    # auto-detects, picks the right output
cargo behave --output tree      # explicit override always wins
```

**Why:** Vitest auto-detects CI and switches to `github-actions` reporter
with zero config. This is the gold standard. Every other framework
requires manual `--output` flags or CI-specific config. Auto-detection
eliminates an entire class of "I forgot to set the CI flag" problems.

**Tests:**

Detection:
- `$GITHUB_ACTIONS=true` → github output with annotations
- `$CLAUDE_CODE=1` → agent output
- Interactive TTY → tree output with colors
- Piped to file → tree output without colors
- Multiple env vars: most specific wins (agent > CI > terminal)

Override:
- `--output tree` in GitHub Actions → tree output (override)
- `--output json` always produces JSON regardless of environment
- `behave.toml` `[ci] output = "junit"` respected in CI profile

**Effort:** Small (environment detection, output format selection).

#### Feature: Config-driven defaults in `behave.toml`

**What:** Set default CLI flags in `behave.toml` to avoid repetitive
command-line arguments:

```toml
[defaults]
timeout_ms = 5000
retry = 2
slow_threshold_ms = 500
exclude_tags = ["slow"]
output = "tree"

[ci]
output = "github"
fail_on_focus = true
retry = 3

[watch]
debounce_ms = 300
clear = true
```

`cargo behave` loads `[defaults]`. `cargo behave --profile ci` loads `[ci]`
(overriding defaults). Explicit CLI flags always win.

**Why:** Teams share testing configuration via their repo. Without config
defaults, every developer must remember the right flags. nextest's
`.config/nextest.toml` profiles are the model.

**Effort:** Medium (config parsing, profile merging, CLI integration).

---

### v0.14.0 — "Hermetic Testing"

**Theme:** Eliminate the most common causes of flaky tests at the
structural level. Make test isolation the default, not an afterthought.

#### Fix: `xfail` catches panics (not just `Err`)

**What:** The `xfail` wrapper currently only catches `Result::Err` returns.
If the test body panics (index out of bounds, division by zero, etc.),
the panic propagates past the xfail wrapper and the test fails as a real
failure instead of being caught as an expected failure.

```rust
behave! {
    "known bugs" {
        xfail "panics on empty input" {
            parse(&[]);  // panics with "index out of bounds"
        }
        // Today: REAL FAILURE (panic propagates)
        // Goal: EXPECTED FAILURE (panic caught by xfail)
    }
}
```

**Why:** The DX audit found this is a correctness issue. Users expect
`xfail` to catch any test failure, not just `Err` returns. RSpec's
`pending` catches both exceptions and assertion failures. pytest's
`@pytest.mark.xfail` catches both.

**Implementation:**
- Wrap the xfail test body in `std::panic::catch_unwind`
- Panic → treated as expected failure (test passes)
- No panic + `Ok(())` → unexpected pass (test fails)
- No panic + `Err(e)` → expected failure (test passes) [existing behavior]
- Teardown still runs after catch_unwind

**Tests:**

Panic handling:
- Test that panics with `xfail` → passes (expected failure)
- Test that panics with specific message → passes
- Test that returns `Err` with `xfail` → passes (existing behavior)
- Test that succeeds with `xfail` → fails ("expected failure but test passed")

Teardown:
- Teardown runs after caught panic
- Teardown runs after caught `Err`
- Teardown panic is reported separately

Edge cases:
- `catch_unwind` with non-`UnwindSafe` types: the closure is `AssertUnwindSafe`
- Nested xfail (not allowed, compile error)
- xfail + timeout: timeout triggers before panic → timeout message includes xfail context

**Effort:** Small (wrap body in `catch_unwind`, existing codegen pattern).

#### Feature: Progress bar for long test suites

**What:** Show a nextest-style progress bar with currently-running tests
during execution:

```
    Running [00:01:23] [===========------] 131/297
    ✓ auth::login (2ms)
    ✓ auth::permissions (4ms)
      database::migration (running for 3.2s)
      api::rate_limiting (running for 1.8s)
```

**Why:** For suites with 100+ tests, the current output is either
all-at-once (after completion) or scrolling lines. A progress bar shows
activity, estimates completion, and highlights stuck/slow tests.

**Implementation:**
- Use `crossterm` for cursor control and line overwriting
- Parse cargo test output line by line as it arrives
- Show progress bar + pass/fail/skip counts in real-time
- Show up to 4 currently-running test names below the bar
- When a test exceeds slow threshold, highlight it in progress area
- On completion, replace progress area with final summary

**Tests:**

Progress display:
- Bar shows `N/total` count updating as tests complete
- Elapsed time shown in `[HH:MM:SS]` format
- Currently-running tests shown below bar
- Tests exceeding slow threshold highlighted in yellow

Terminal handling:
- Non-TTY: fallback to simple counter (no cursor control)
- Window resize: progress bar width adjusts
- `Ctrl-C`: clean exit, restore cursor position

Accuracy:
- Total test count from `cargo test -- --list` output
- Pass/fail/skip counts accurate at every point
- Final summary matches actual results

**Effort:** Medium (real-time output parsing, cursor control).

#### Feature: `hermetic;` keyword with test sandboxing

**What:** A group-level keyword that enables per-test isolation:

```rust
behave! {
    "file processing" {
        hermetic;

        "creates output file" |ctx| {
            let dir = ctx.tmp_dir();  // unique, auto-cleaned
            process_file(&dir.join("in.txt"), &dir.join("out.txt"))?;
            expect!(dir.join("out.txt")).to_exist()?;
        }
        // tmp_dir automatically deleted after test
    }
}
```

`hermetic;` enables:
1. **Unique temp directory per test** — automatically created, path injected
   via test context, cleaned up after. No two tests share filesystem state.
2. **Environment variable isolation** — snapshot `env::vars()` before the test,
   restore after. No test can pollute another's environment.
3. **Deterministic RNG seed** — derive from test name + global seed. Same
   seed every run, different per test.

**Why:** Flaky tests erode trust in the entire suite. When developers stop
trusting tests, they stop running them. The top causes of flakiness —
shared filesystem, env var leakage, non-deterministic ordering — are all
preventable. Deno's [resource sanitizer][deno-sanitizer] proves this works.
ExUnit's `:tmp_dir` tag is the model for per-test temp directories.
Bazel's [hermeticity][bazel-hermetic] principle shows that structural
isolation is more effective than after-the-fact detection.

[deno-sanitizer]: https://docs.deno.com/runtime/fundamentals/testing/
[bazel-hermetic]: https://bazel.build/basics/hermeticity

**Implementation:**
- `hermetic;` is a group-level keyword (like `tokio;`) that generates a
  context struct passed to tests
- The context provides `.tmp_dir()`, `.env()`, `.rng()`
- Teardown automatically cleans the temp dir and restores env vars
- Behind `std` feature (filesystem operations)

**Effort:** Medium-Large (context struct, codegen, cleanup logic).

#### Feature: Resource leak detection (post-test assertions)

**What:** Automatically detect common resource leaks after each test:

```
✗ database::query (RESOURCE LEAK)
  Leaked: 2 open file descriptors
    fd 7: /tmp/behave-test-xxxx/data.db (opened at tests/db.rs:15)
    fd 9: /tmp/behave-test-xxxx/wal.db (opened at tests/db.rs:16)
  Hint: ensure all File handles are dropped or explicitly closed
```

**Why:** Resource leaks are the #1 cause of flaky tests in systems code
and the hardest bugs to track down. Deno's test runner detects leaked
resources (file handles, network connections, timers) automatically.
Rust's ownership model means leaked resources are usually accidental
(e.g., a `File` stored in a struct that outlives the test).

**Implementation:**
- On Linux: snapshot `/proc/self/fd` before/after test
- On macOS: use `proc_pidinfo` or similar
- Compare and report any new file descriptors that were not closed
- Behind `std` feature, opt-in via `hermetic;` or `leak_check;`

**Effort:** Medium (platform-specific fd tracking, error formatting).

#### Feature: Serial execution via tags

**What:** `cargo behave` recognizes a `serial` tag as a directive to run those
tests single-threaded, preventing interference from shared global state.

```rust
behave! {
    "config loading" {
        tag "serial";

        setup {
            std::env::set_var("DATABASE_URL", "postgres://test");
        }

        teardown {
            std::env::remove_var("DATABASE_URL");
        }

        "reads DATABASE_URL from environment" {
            let config = Config::from_env()?;
            expect!(config.database_url()).to_equal("postgres://test")?;
        }
    }
}
```

```bash
cargo behave                           # serial-tagged tests run sequentially
cargo behave --tag serial              # run ONLY serial tests
```

**Why:** Test flakiness from shared global state (environment variables, current
working directory, static mutexes, filesystem) is the [#1 developer
complaint][serial-test] across all languages. The `serial_test` crate (4.5M
downloads/mo) exists solely for this. Recognizing `serial` as a built-in tag
behavior means users get isolation without another dependency. Combined with
`hermetic;` for filesystem and env isolation, this handles 80% of flakiness.

[serial-test]: https://crates.io/crates/serial_test

**Implementation:**
- `cargo behave` collects tests tagged `serial` into a separate execution group
- Serial group runs after parallel tests complete (or before, configurable)
- No proc-macro changes needed — uses existing tag infrastructure
- Also available in `behave.toml`: `serial_tags = ["serial", "db"]`

**Tests:**

Tag recognition:
- Tests tagged `serial` are detected via `__TAG_serial__` in test name
- Multiple serial tags in `behave.toml` are all recognized (`serial_tags = ["serial", "db"]`)
- Tests with both `serial` and other tags are correctly partitioned

Execution ordering:
- Serial-tagged tests run sequentially (never in parallel with each other)
- Serial group runs after all parallel tests complete (default order)
- Parallel tests are not blocked by serial group (no unnecessary serialization)
- `--tag serial` runs only serial tests (existing filter behavior)
- `--exclude-tag serial` runs only parallel tests

Isolation guarantees:
- Two serial tests that modify the same env var do not interfere
- Serial tests see a clean environment (no leakage from parallel group)
- Teardown in serial tests runs before the next serial test starts

Configuration:
- `behave.toml` `serial_tags` config is loaded and merged with defaults
- CLI `--serial-tag` flag overrides config file
- Empty `serial_tags` disables serial grouping (all tests run in parallel)

Output:
- Tree output groups serial tests under a `[serial]` section
- Summary shows serial test count: "5 passed (2 serial), 0 failed"
- JUnit/JSON output includes serial execution metadata

Edge cases:
- Zero serial tests: all tests run in parallel, no serial phase
- All tests serial: entire suite runs sequentially
- Serial tag combined with `--retry`: retried serial tests also run sequentially
- Serial tag combined with `--watch`: serial grouping preserved across re-runs

**Effort:** Small-Medium (CLI execution grouping, tag-based partitioning).

---

### v1.0.0 — "Stability Commitment"

**Theme:** API freeze. Production-ready. The commitment.

#### API audit and freeze

Review every public type, trait, method, and macro:
- Is the name correct and will we want it in 5 years?
- Is `#[non_exhaustive]` on every public struct and enum?
- Are all unstable/experimental items behind feature flags?
- Remove anything that should not be committed to

This is the most important task in the release. rstest has had
[24 breaking releases pre-1.0][rstest-breaking]. behave should commit clearly.

[rstest-breaking]: https://github.com/la10736/rstest/issues/197

#### Error message audit

Every matcher tested with intentionally wrong values. Review output for:
- Clarity: can a developer understand the failure in <5 seconds?
- Completeness: actual value, expected value, description all present?
- Formatting: multi-line values diff cleanly with `color` feature?
- Truncation: large values don't cause memory issues or unreadable output?

#### STABILITY.md policy

Document:
- What is covered by SemVer (public API, macro syntax, CLI flags)
- What may change in 1.x (internal codegen structure, error message wording)
- MSRV policy (how often it bumps, how much notice)
- Deprecation policy (minimum 2 minor versions before removal)

#### Performance baseline

Publish compile-time benchmarks as part of the release:
- Overhead per 100 tests vs bare `#[test]`
- Incremental recompilation time
- Binary size impact

#### Adopt libtest JSON output (if stabilized)

The [libtest JSON stabilization][libtest-json] is a 2025 H1 Rust Project
Goal. When the `--format json` flag lands on stable, behave should adopt it
immediately:
- Replace the fragile text-based `pretty` output parser with structured JSON
- Unlock per-test durations, stderr capture, and metadata without heuristics
- Reduce maintenance burden of the CLI parser module

[libtest-json]: https://rust-lang.github.io/rust-project-goals/2025h1/libtest-json.html

---

### v1.1+ — "Innovation" (post-1.0)

**Theme:** Features that push the boundary of what a testing framework
can do. These leverage Rust's unique strengths (type system, ownership,
proc macros) to create capabilities no other language can match.

#### Feature: Compile-failure assertions (`compile_fail` blocks)

**What:** Verify that certain misuses of an API *fail to compile*, co-located
with the rest of the BDD suite instead of in separate trybuild files.

```rust
behave! {
    "type safety" {
        compile_fail "cannot build Pipeline with no stages" {
            let _ = PipelineBuilder::new().build();
            //~^ ERROR: cannot call `build` on `PipelineBuilder<Empty>`
        }

        compile_fail "Send bound prevents non-Send types" {
            send_to_thread(Rc::new(42));
        }
    }
}
```

The proc macro detects `compile_fail` blocks and emits trybuild-style test
harnesses automatically. The expected error pattern is inline, not in a
separate `.stderr` file. Tests appear in the BDD tree for discoverability.

**Why:** Every crate using typestate patterns, builder patterns, or newtype
enforcement needs compile-failure tests. [`trybuild`][trybuild] is functional
but operates as a completely separate workflow: one `.rs` file per test case,
`.stderr` snapshot files, disconnected from the test suite. The alignment with
behave's vision is near-perfect: "leverages Rust's type system" and "parse,
don't validate" are exactly the patterns that need negative compilation tests.
No other BDD framework in any language offers this.

[trybuild]: https://docs.rs/trybuild/latest/trybuild/

**Implementation:** `compile_fail` blocks expand to a function that shells out
to `rustc` (like trybuild) with the block body as source. The expected error
pattern is extracted from `//~^` comments. Runs as a normal `#[test]` function.

**Tests:**

Basic compilation failure:
- `compile_fail` block with code that fails to compile → test passes
- `compile_fail` block with code that compiles successfully → test fails with
  "expected compilation failure but code compiled"
- Failure message includes the code block and the fact that it compiled

Error pattern matching:
- `//~^ ERROR: <pattern>` matches the compiler error message (substring)
- Missing `//~^` comment → test passes on any compilation error (no pattern check)
- `//~^ ERROR: <pattern>` that does not match actual error → test fails showing
  expected pattern vs actual compiler output
- Multiple `//~^` annotations on different lines each match independently

Code generation:
- `compile_fail` block generates a valid `#[test]` function (appears in `cargo test --list`)
- `compile_fail` block inside a group inherits the group's module path
- `compile_fail` block with `tag "type-safety"` → tag is encoded in test name
- `compile_fail` blocks co-exist with normal tests in the same `behave!` invocation

Edge cases:
- `compile_fail` block referencing types from the current crate resolves correctly
- `compile_fail` block with `use` imports at the top works
- `compile_fail` combined with `focus` → focused compile-fail test runs
- `compile_fail` combined with `pending` → compile error (cannot combine)
- Timeout on `rustc` invocation does not hang the test suite
- Parallel execution: multiple `compile_fail` tests do not interfere

Proc macro errors:
- `compile_fail` on a group (not a test) → `compile_error!("compile_fail cannot be applied to groups")`
- `compile_fail` combined with `xfail` → `compile_error!("cannot combine")`
- Empty `compile_fail` body → `compile_error!("compile_fail block must have a body")`

**Effort:** Medium-Large (trybuild-like harness generation, error matching).

#### Feature: Static assertions in the BDD tree (`static_assert` blocks)

**What:** Embed compile-time property checks in the test tree for
discoverability and documentation.

```rust
behave! {
    "type properties" {
        static_assert "Config is Send + Sync" {
            const _: () = {
                fn assert_send_sync<T: Send + Sync>() {}
                assert_send_sync::<Config>();
            };
        }

        static_assert "Packet fits in a cache line" {
            const _: () = assert!(std::mem::size_of::<Packet>() <= 64);
        }
    }
}
```

The proc macro expands `static_assert` blocks into `const _: () = { ... }`
items that are verified at compile time. They appear in the test tree output
for documentation purposes but have **literally zero runtime cost**.

**Why:** Every published crate should verify Send/Sync/Sized/Unpin bounds.
[`static_assertions`][static-assert] handles some cases but exists outside
any test framework. Embedding these in the BDD tree makes them discoverable
and documented. This is the purest expression of behave's vision: zero-runtime,
leverages Rust's type system, compiles to ordinary code.

[static-assert]: https://docs.rs/static_assertions/latest/static_assertions/

**Tests:**

Basic static assertions:
- `static_assert` block with true assertion compiles and test passes
- `static_assert` block with false assertion fails at compile time (not runtime)
- `static_assert` with `assert!(size_of::<T>() <= N)` enforces size bounds
- `static_assert` with Send/Sync/Unpin bound checks compiles for valid types

Code generation:
- `static_assert` emits `const _: () = { ... }` in the generated module
- `static_assert` appears in `cargo test --list` output (as a zero-body test function)
- `static_assert` appears in `cargo behave` tree output with the label
- `static_assert` inside a group inherits the group's module path
- Multiple `static_assert` blocks in the same group do not collide

Integration with DSL:
- `static_assert` with `tag "compile-time"` → tag encoded in test name
- `static_assert` blocks co-exist with normal tests and `compile_fail` blocks
- `static_assert` inherits no setup/teardown (compile-time, no runtime context)
- `static_assert` combined with `focus` → focused static assertion
- `static_assert` combined with `pending` → `#[ignore]` on the placeholder test

Tree output:
- `cargo behave` renders `static_assert` tests with a distinct marker (e.g., `◆`)
- `static_assert` tests show 0ms duration (compile-time, no runtime)

Edge cases:
- `static_assert` on a group → `compile_error!("static_assert is a test-level keyword")`
- `static_assert` with empty body → `compile_error!("static_assert must have a body")`
- `static_assert` referencing generic type `T` from `each_type` → works correctly

**Effort:** Small (proc macro emits `const` items, no runtime code).

#### Feature: Allocation tracking assertions

**What:** Assert heap allocation counts and sizes directly in tests for
performance-sensitive code paths.

```rust
behave! {
    "hot path" {
        "parser allocates at most 5 times" {
            let stats = behave::track_allocs(|| {
                parse(&input);
            });
            expect!(stats.total_allocations()).to_be_at_most(5)?;
            expect!(stats.peak_bytes()).to_be_less_than(4096)?;
        }
    }
}
```

`track_allocs` wraps a closure with a thread-local allocation counter. The
returned `AllocStats` struct plugs into behave's existing ordering matchers.

**Why:** Performance-sensitive Rust code (parsers, serializers, embedded,
game engines) needs inline allocation assertions. [`dhat-rs`][dhat] can track
allocations but requires manual global allocator setup and has no assertion
integration. No testing framework does this well inline. The matchers are
zero-cost; only the counting wrapper has overhead.

[dhat]: https://docs.rs/dhat/latest/dhat/

Behind an `alloc-tracking` feature flag. Requires a global allocator wrapper
(test-only, similar to dhat-rs's approach).

**Tests:**

`track_allocs` closure wrapper:
- `track_allocs(|| {})` returns `AllocStats` with zero allocations
- `track_allocs(|| { vec![1, 2, 3] })` reports at least 1 allocation
- `track_allocs(|| { let v = vec![0u8; 1024]; drop(v); })` tracks bytes allocated
- Nested `track_allocs` calls each track independently (thread-local counter)
- `track_allocs` in multi-threaded context only counts current thread's allocations

`AllocStats` fields:
- `total_allocations()` returns count of `alloc` calls
- `total_deallocations()` returns count of `dealloc` calls
- `total_bytes()` returns cumulative bytes allocated
- `peak_bytes()` returns high-water mark of live bytes
- All fields work with existing ordering matchers (`to_be_less_than`, `to_be_at_most`, etc.)

Matcher integration:
- `expect!(stats.total_allocations()).to_be_at_most(5)?` passes and fails correctly
- `expect!(stats.peak_bytes()).to_be_less_than(4096)?` passes and fails correctly
- Failure messages include the actual allocation count/bytes
- `AllocStats` implements `Debug` for readable failure output

Edge cases:
- `track_allocs` with a closure that panics: counter is still restored (no leak)
- `track_allocs` with zero allocations: all fields are zero
- `track_allocs` with realloc: counts as dealloc + alloc (consistent)
- Feature gate: `alloc-tracking` not enabled → `track_allocs` is not available
- Global allocator wrapper only activates when `track_allocs` is in scope

Compile-time safety:
- `track_allocs` requires `alloc-tracking` feature — missing feature gives clear error
- `AllocStats` is `#[non_exhaustive]` — adding fields in minor version is safe
- No `unsafe` in public API (allocator wrapper is internal)

**Effort:** Medium (global allocator wrapper, AllocStats type, feature flag).

#### Feature: Trait contract testing

**What:** A `contract` block that defines tests against a trait, then
automatically generates test suites for every listed implementation:

```rust
behave! {
    contract Storage for [MemoryStorage, SqliteStorage, RedisStorage] {
        "stores and retrieves" |store: impl Storage| {
            store.put("key", "value")?;
            expect!(store.get("key")?).to_be_some_with("value")?;
        }

        "returns None for missing keys" |store: impl Storage| {
            expect!(store.get("missing")?).to_be_none()?;
        }
    }
}
// Generates:
// storage_contract::memory_storage::stores_and_retrieves
// storage_contract::memory_storage::returns_none_for_missing_keys
// storage_contract::sqlite_storage::stores_and_retrieves
// ... etc.
```

**Why:** When you add a new implementation of a trait, you get automatic
verification. When you change the contract, every impl is tested. This is
the Liskov Substitution Principle enforced by tests. Only possible in Rust
because traits define the contract at the type level.

#### Feature: Test execution traces (on-failure recording)

**What:** On failure (or on retry), emit a `.trace.json` file containing
a complete timeline of what happened during the test:

```
$ cargo behave trace "database::connection::handles_timeout"

Timeline:
  0ms  setup: connect_db() -> Ok(Connection { id: 42 })
  1ms  expect!(conn.is_alive()).to_be_true() -> PASS
  2ms  mock_network_partition()
 15ms  expect!(conn.query("SELECT 1")).to_be_err() -> PASS
 16ms  expect!(conn.reconnect()).to_be_ok() -> FAIL
         actual: Err(Timeout)
         expected: to be ok
 16ms  teardown: conn.close()
```

**Why:** Inspired by [Playwright's Trace Viewer][playwright-trace]. The key
insight: "on-first-retry" tracing — zero overhead on passing tests. Only
when a test fails and is retried does the trace recorder activate. This
eliminates the "add println, recompile, re-run" debugging loop that is
especially painful in Rust due to compile times.

[playwright-trace]: https://playwright.dev/docs/trace-viewer

#### Feature: Mutation-aware quality hints

**What:** After running tests, statically analyze which functions have no
assertions that would catch a return-value replacement. Flag them as
"undertested" without running full mutation testing:

```
$ cargo behave --mutation-hints

Tests: 142 passed, 0 failed
Undertested functions (return value never asserted):
  src/auth.rs:45  fn hash_password() -> String
  src/db.rs:112   fn connection_pool_size() -> usize
```

**Why:** 100% test pass rate creates false confidence. [cargo-mutants][mutants]
exists but is slow (runs the full suite per mutation). Static hints provide
80% of the insight at 1% of the cost by checking whether test assertions
cover function return values.

[mutants]: https://mutants.rs/

#### Feature: `property` keyword for property-based testing

**What:** Integrate proptest/quickcheck into the BDD DSL:

```rust
behave! {
    "sorting" {
        property [proptest] |v: Vec<i32>| {
            let sorted = sort(&v);
            expect!(sorted.len()).to_equal(v.len())?;
            expect!(sorted).to_be_sorted()?;
        }
    }
}
```

**Why:** Property testing is orthogonal to example-based testing. Most
developers never adopt it because it requires learning a separate framework
and writing tests in a different style. Embedding it in the BDD DSL makes
it approachable. The combination of BDD organization + property testing is
unique across all languages.

#### Feature: Compile-time boundary testing from types

**What:** A `#[derive(BehaveBoundary)]` or `exhaustive` keyword that
generates deterministic edge-case tests from type information:

```rust
behave! {
    "user validation" {
        exhaustive User |user| {
            expect!(validate(&user)).to_be_ok()?;
        }
    }
}
```

The proc macro inspects the type and generates tests for:
- Numeric fields: `0`, `MAX`, `MIN`, `-1`, `1`
- `Option` fields: `Some(edge_values)` and `None`
- `String` fields: `""`, very long strings, unicode edge cases
- `Vec` fields: empty, single element, many elements
- Enum fields: one test per variant

**Why:** Developers forget edge cases. Types don't. This leverages Rust's
type system to eliminate the entire category of "I didn't think to test
that boundary." Only possible in a language with compile-time type
reflection via proc macros.

#### Feature: `each_file` for file-driven test generation

**What:** Generate tests from fixture files at compile time:

```rust
behave! {
    "parser" {
        each_file "fixtures/valid/*.json" |path, content| {
            let result = parse(content);
            expect!(result).to_be_ok()?;
        }
    }
}
```

**Why:** `datatest-stable` requires `harness = false` (breaking nextest
and IDE compatibility). behave can provide file-driven testing while
maintaining `#[test]` compatibility by reading files at compile time via
`include_str!`.

#### Feature: Change matchers (RSpec-inspired)

**What:** Assert that an operation changes a value by a specific amount:

```rust
expect_change!(|| db.user_count(), by: 1, when: || db.insert_user(&user))?;
expect_change!(|| account.balance(), from: 100, to: 75, when: || account.withdraw(25))?;
expect_no_change!(|| cache.size(), when: || cache.get("missing"))?;
```

**Why:** One of RSpec's most-loved matchers. In Rust, expressing "this
operation changes that value" requires manually capturing before/after
values — tedious and error-prone. The macro captures both snapshots and
produces clear failure messages.

**Tests:**

`expect_change!`:
- `by: 1` passes when value increases by exactly 1
- `by: 1` fails when value increases by 2 (shows `changed by 2, expected 1`)
- `from: 100, to: 75` passes when value transitions correctly
- `from: 100, to: 75` fails when starting value is wrong (shows `started at 90, expected 100`)
- Works with closures returning any `PartialEq + Debug + Sub` type
- Failure message shows before, after, and expected change

`expect_no_change!`:
- Passes when value unchanged after operation
- Fails when value changes (shows `changed from X to Y, expected no change`)

**Effort:** Small (declarative macro, before/after capture).

#### Feature: `given` / `when` / `then` BDD formatting

**What:** Recognize BDD-style prefixes in test labels and format them
specially in `cargo behave` tree output.

```rust
behave! {
    "checkout" {
        "given a cart with items" {
            let cart = Cart::with_items(3);

            "when the user checks out" {
                let result = cart.checkout();

                "then the order is created" {
                    expect!(result).to_be_ok()?;
                }

                "then the cart is emptied" {
                    expect!(cart.len()).to_equal(0)?;
                }
            }
        }
    }
}
```

Tree output with BDD-aware formatting:
```
  checkout
    Given a cart with items
      When the user checks out
        ✓ Then the order is created (1ms)
        ✓ Then the cart is emptied (0ms)
```

**Why:** Catch2's `SCENARIO / GIVEN / WHEN / THEN` macros are just
aliases for `TEST_CASE / SECTION` with prefixed names. Same principle
here: no DSL changes, just formatting intelligence. BDD vocabulary in
output makes tests readable as specifications by non-developers (PMs,
QA). Kotest offers 10 different spec styles for this reason.

**Implementation:**
- No proc macro changes — `given`, `when`, `then` are just string prefixes
- `cargo behave` renderer detects labels starting with `given `, `when `,
  `then ` (case-insensitive) and capitalizes/indents them
- Optionally bold/italic formatting for the keyword
- JSON output includes a `bdd_keyword` field if detected

**Tests:**

Detection:
- `"given ..."` recognized as Given keyword
- `"when ..."` recognized as When keyword
- `"then ..."` recognized as Then keyword
- `"and ..."` recognized as continuation (same indent as previous)
- Case-insensitive: `"Given ..."` and `"GIVEN ..."` both work
- No prefix: rendered as normal group/test (no change)

Formatting:
- Given/When/Then labels capitalized in output
- Indentation follows BDD hierarchy
- Leaf tests (assertions) show pass/fail marker
- Non-leaf nodes (groups) show no marker

Edge cases:
- Mixed BDD and non-BDD labels in same suite
- Nested Given blocks (unusual but valid)
- Label that starts with "given" but is not BDD (e.g., "given_name validation")
  — only matches if followed by a space

**Effort:** Small (renderer change only, no macro changes).

---

## What We Will NOT Build

These decisions are informed by specific competitor failures. Each links to the
evidence.

### No `let`-style lazy bindings

RSpec's `let` is their [single most debated feature][let-debate]. Thoughtbot's
["Let's Not"][mystery-guest] shows it creates "mystery guests" — setup invisible
at the point of use. `let!` was a [band-aid for a self-inflicted problem][let-bang].
The original RSpec creator now prefers Minitest's explicit setup.

behave's `setup` blocks are eager, visible, and use normal Rust `let` bindings.
This is correct.

[let-debate]: https://dogweather.dev/2016/10/20/why-i-dont-use-letlet-in-my-rspec/
[let-bang]: https://www.codewithjason.com/difference-let-let-instance-variables-rspec/

### No shared examples / shared contexts

Despite being an iconic RSpec feature, shared examples are
[widely considered an anti-pattern][shared-antipattern]:
- "Ghost variables": shared examples depend on undocumented variables that
  includers must define, with no type checking
- "Exponential test growth": m shared examples * n includers = m*n tests
- "Debugging nightmares": failures report bracket coordinates, not source lines
- DRY applied to the wrong thing: tests are documentation, duplication is fine

### No `before_all` / `after_all` / suite-level hooks

This fights the ["ordinary tests" promise][ordinary-tests]. rstest's `#[once]`
[never drops resources][rstest-once] and is
[incompatible with nextest][nextest-once]. pytest's session-scoped fixtures
[break in parallel execution][xdist-session].

For expensive shared resources, the correct pattern in Rust is explicit
`OnceLock`/`LazyLock` in user code, which works with all test runners.

[ordinary-tests]: /docs/ARCHITECTURE.md

### No fixture injection (pytest-style)

pytest's own maintainers [acknowledge][pytest-fixture-debate] that fixtures add
"indirection (harder to see what is happening) and coupling (if I want different
data in a test, that may affect other tests)." Fixture scopes create a
[hierarchy where higher-scoped fixtures cannot use lower-scoped ones][pytest-scopes],
and conftest.py creates [cascading override nightmares][conftest-chaos].

behave's `setup` blocks + normal Rust functions are simpler and more explicit.

[pytest-fixture-debate]: https://github.com/pytest-dev/pytest/discussions/11085
[pytest-scopes]: https://pawamoy.github.io/posts/same-pytest-fixtures-with-different-scopes/

### No mocking / stubbing

mockall is the standard tool. Rust's type system makes mocking fundamentally
different from dynamic languages. This is orthogonal to assertion/DSL concerns.
RSpec's mocking [allowed false positives][rspec-mock-bug] for years because
stubs didn't verify method signatures — a problem Rust's trait system prevents.

[rspec-mock-bug]: https://github.com/rspec/rspec-mocks/issues/227

### No property-based testing

proptest and quickcheck are mature and orthogonal. Absorbing them would add
significant complexity without clear user benefit. They compose alongside
behave naturally.

### No built-in snapshot testing

Jest snapshot testing is [self-defeating at scale][snapshot-trap]: "Engineers
begin blindly updating failed snapshots." The `--update` flag is a "one-way
ratchet of laziness." Where snapshots work is legacy code rescue — a temporary
scaffolding tool, not a testing strategy.

If users want snapshots, insta integrates alongside behave. behave's rich
matcher error messages with colored diffs achieve the same diagnostic goal
without the "blind update" anti-pattern.

[snapshot-trap]: https://medium.com/@sapegin/whats-wrong-with-snapshot-tests-37fbe20dfe8e

### No plugin architecture

pytest's plugin ecosystem is their [biggest maintenance burden][pytest-plugins]:
major API changes break plugins, private APIs are used and then changed, pluggy
itself broke pytest. Establishing a plugin API before 1.0 would constrain
internal evolution.

Post-1.0, if demand exists, consider a small, stable extension API for custom
matchers (already exists via `BehaveMatch<T>`) and custom output formatters.

[pytest-plugins]: https://github.com/pytest-dev/pytest/issues/3744

### No custom test runner / harness

The "ordinary `#[test]` functions" promise is behave's core differentiator.
Tests work with `cargo test`, `cargo-nextest`, IDEs, CI, and every Rust tool
that understands `#[test]`. Breaking this would destroy the value proposition.

---

## Release Timeline Summary

| Version | Theme | Key Features |
|---------|-------|-------------|
| **v0.6.0** | Better Parameterization | `matrix` Cartesian product, named `each` cases, `xfail` |
| **v0.7.0** | Tags & Filtering | `tag` metadata, `skip_when!`, watch mode |
| **v0.8.0** | Type Power | `each_type`, `to_match_pattern!`, partial struct matching |
| **v0.9.0** | Polish & Robustness | failure output audit, compile-time budget, CLI filters, retry |
| **v0.10.0** | Docs & Adoption | matcher gap closure, VS Code snippets, migration guides, `cargo behave init`, `--output github`, **34 DX audit fixes** (error quality, API ergonomics, code dedup, CLI quality, codegen, test coverage) |
| **v0.11.0** | Smart Failures | expression decomposition, `require!`, improved collection errors, `--slow-threshold`, OSC 8 links, `--output agent`, combinator macros, failure capture |
| **v0.12.0** | Async & Integration | `paused_time;`, `to_eventually` retry matcher, stream matchers, `--runner nextest`, tracing |
| **v0.13.0** | Test Intelligence | interactive watch mode, env-aware output, `--affected`, `--failed-last`, `--output markdown`, config profiles |
| **v0.14.0** | Hermetic Testing | `xfail` panic fix, progress bar, `hermetic;`, serial tags, env isolation, temp dirs |
| **v1.0.0** | Stability | API freeze, STABILITY.md, compile-time budget, perf baseline |
| **v1.1+** | Innovation | `compile_fail`, `static_assert`, alloc tracking, trait contracts, `given/when/then`, change matchers, test traces, property DSL |

---

## Competitive Position

```
                    Test Organization
                         ^
                         |
              cucumber-rs |  * behave
           (Gherkin files)|  (in-code BDD DSL)
                         |
    <--------------------+---------------------->
    Minimalist           |           Rich Framework
    (cargo test)         |
                         |
              rstest     |  test-case
           (fixtures)    |  (parameterization)
                         |
                         v
                    Flat Test Functions
```

### Where behave wins today

| Dimension | behave | Nearest Competitor | Gap |
|-----------|--------|-------------------|-----|
| BDD DSL in Rust | Only real option | cucumber-rs (Gherkin files) | Wide |
| Integrated DSL + matchers + CLI | Single crate | rstest + googletest + pretty_assertions (3 crates) | Wide |
| Setup/teardown with inheritance | Built-in, nesting-aware | rstest `#[fixture]` (flat, no teardown) | Medium |
| History-based flaky detection | Across runs, source-aware | nextest (per-run retry only) | Medium |
| Soft assertions | `SoftErrors` explicit API | googletest `expect_that!` (attribute-based) | Parity |

### Where competitors win today (gaps to close)

| Dimension | Competitor | Their Advantage | Closed by |
|-----------|-----------|-----------------|-----------|
| Parameterization breadth | rstest | Cartesian `#[values]` | v0.6 `matrix` ✅ |
| Test filtering | pytest, RSpec | Arbitrary metadata tags | v0.7 `tag` ✅ |
| Runtime skip | pytest | `@pytest.mark.skip` | v0.7 `skip_when!` ✅ |
| Watch mode | Vitest, Jest | Auto-rerun on save | v0.7 `--watch` ✅ |
| Typed tests | Google Test | `TYPED_TEST_SUITE` | v0.8 `each_type` ✅ |
| Pattern matching | (none) | Rust-native patterns | v0.8 `expect_match!` ✅ |
| CI retry | nextest | `--retries` | v0.9 `--retry` ✅ |
| Filter expressions | nextest | Structured queries | v0.9 `--filter` ✅ |
| Migration guides | pytest, Jest | Clear onboarding docs | v0.10 docs overhaul |
| CI annotations | dorny/test-reporter | Inline PR failure view | v0.10 `--output github` |
| Failure context | power-assert, pytest | Sub-expression values | v0.11 enhanced errors |
| Re-run failures | pytest `--lf` | Skip passing tests | v0.13 `--failed-last` |
| Test impact analysis | Gradle Develocity | Run only affected tests | v0.13 `--affected` |
| Hermetic isolation | Deno, Bazel | Structural flake prevention | v0.14 `hermetic;` |
| Deterministic time | tokio `start_paused` | Time-travel in async | v0.12 `paused_time;` |
| Trait contract tests | (none) | Verify all impls automatically | v1.1 `contract` |
| Process output matchers | googletest | `Output` assertions | v0.10 matcher gap closure |
| Iterator matchers | speculoos | Generic `IntoIterator` assertions | v0.10 matcher gap closure |
| Smart pointer deref | (manual) | Auto-deref `Box`/`Arc`/`Rc` | v0.10 matcher gap closure |
| Serde round-trip | (none) | Serialize→deserialize equality | v0.10 matcher gap closure |
| Compile-failure tests | trybuild | Co-located negative compilation | v1.1 `compile_fail` |
| Static type assertions | static_assertions | Send/Sync/Size in BDD tree | v1.1 `static_assert` |
| Allocation tracking | dhat-rs (manual) | Inline alloc count assertions | v1.1 `track_allocs` |
| Serial test execution | serial_test crate | Tag-based sequential grouping | v0.14 serial tags |
| Expression decomposition | Swift `#expect`, Catch2 | Sub-expression values on failure | v0.11 expression decomposition |
| Clickable source links | (none in Rust) | OSC 8 file:line hyperlinks | v0.11 OSC 8 links |
| AI-optimized output | Vitest `agent` reporter | Minimal, token-efficient output | v0.11 `--output agent` |
| Inline failure details | pytest, nextest | Error message in tree output | v0.11 failure capture |
| Retry/eventually matchers | Playwright auto-wait | Poll until condition met | v0.12 `to_eventually` |
| Interactive watch | Vitest, Jest | Keyboard shortcuts in watch mode | v0.13 interactive watch |
| Auto-detect environment | Vitest | Zero-config CI/agent/terminal switch | v0.13 env-aware output |
| Progress bar | nextest | Running test names + ETA | v0.14 progress bar |
| BDD output formatting | Catch2 `SCENARIO` | Given/When/Then in tree output | v1.1 BDD formatting |
| Unwrap-or-fail macro | Swift `#require` | `require!` for Option/Result | v0.11 `require!` |

### Risks

| Risk | Mitigation |
|------|-----------|
| rstest absorbs BDD-like features | Lean into cohesion — rstest will never have a DSL |
| googletest-rust gains traction | Matcher parity already exists; DSL + CLI are the moat |
| Compile time grows with macro complexity | Budget enforcement, codegen optimization in v0.9 |
| Single-maintainer risk (like rstest) | Document everything, keep architecture simple |
| Custom test framework RFC stabilizes | behave already works without it; stabilization would be a tailwind |
| Scope creep from expanded roadmap | Each version has a clear theme; features are behind feature flags |
| libtest JSON format changes during stabilization | Abstract parser behind a trait; support both text and JSON |
| nextest adds BDD-style output | behave's moat is the DSL + matchers, not just the CLI |
| AI test generation reduces manual test writing | behave's DSL is *more* AI-friendly than scattered test files |
| Feature flags accumulate dependency weight | Companion crates for heavy deps; core stays zero-dep |
| Matcher surface area becomes hard to maintain | Each matcher family follows established patterns; audit at v1.0 |
| `compile_fail` / `static_assert` blur test vs compile-time | Clear DSL distinction; `compile_fail` runs `rustc`, `static_assert` is `const` |
| Expression decomposition adds proc macro complexity | Hybrid approach: simple `expect!(x)` stays as `macro_rules!`; decomposition opt-in |
| Interactive watch mode requires raw terminal handling | Use `crossterm` (already a dependency); restore terminal on panic/SIGINT |
| `to_eventually` enables lazy test writing | Default timeout is short (5s); docs emphasize deterministic testing first |
| `--output agent` format must stay stable for AI consumers | Versioned format; `FAIL`/`at`/`expected`/`actual` keywords are contract |

---

## Competitor Vulnerabilities (Intelligence)

### rstest
- No teardown beyond `Drop` ([#94](https://github.com/la10736/rstest/issues/94), 12 upvotes)
- `#[once]` never drops ([#209](https://github.com/la10736/rstest/issues/209))
- `#[once]` incompatible with nextest ([nextest#209](https://github.com/nextest-rs/nextest/issues/209))
- `rstest_reuse` requires fragile macro import ceremony
- rust-analyzer cannot run individual parameterized cases
- 24 breaking releases, no 1.0, single maintainer, no funding
- MSRV jumping to 1.82 (from 1.70) will strand some users

### test-case
- Shadows unstable `#[test_case]` compiler attribute ([#131](https://github.com/frondeus/test-case/issues/131))
- Duplicate test execution with other proc macros ([#146](https://github.com/frondeus/test-case/issues/146))
- Silent error discarding ([#135](https://github.com/frondeus/test-case/issues/135))
- Bad custom names fail silently ([#72](https://github.com/frondeus/test-case/issues/72))

### pretty_assertions
- 96.5 GB memory allocation on ~155KB inputs ([#124](https://github.com/rust-pretty-assertions/rust-pretty-assertions/issues/124))
- "Unbearably slow for big texts" ([#19](https://github.com/rust-pretty-assertions/rust-pretty-assertions/issues/19))
- Only improves `assert_eq!` — no fluent API, no test organization

### nextest
- Cannot run doctests ([#16](https://github.com/nextest-rs/nextest/issues/16))
- Process-per-test makes `lazy_static`/`OnceCell` re-init per test
- Setup/teardown scripts still experimental
- No test organization or BDD-style output

---

## Sources

All claims in this document link to primary sources: GitHub issues, blog posts,
framework documentation, and community discussions. Key references:

### Competitor analysis
- [Thoughtbot: "Let's Not"](https://thoughtbot.com/blog/lets-not) — RSpec `let` problems
- [Solnic: "5 Rules of Simple RSpec Tests"](https://solnic.dev/the-5-rules-of-simple-rspec-tests/) — anti-patterns
- [The Case Against Shared Examples](https://dev.to/epigene/the-case-against-shared-examples-39kh) — RSpec shared examples
- [pytest fixture discussion](https://github.com/pytest-dev/pytest/discussions/11085) — fixture problems
- [Software Testing Anti-patterns](https://blog.codepipes.com/testing/software-testing-antipatterns.html) — framework design
- [What's Wrong with Snapshot Tests](https://medium.com/@sapegin/whats-wrong-with-snapshot-tests-37fbe20dfe8e) — Jest snapshots
- [rstest issue tracker](https://github.com/la10736/rstest/issues) — competitor gaps
- [test-case issue tracker](https://github.com/frondeus/test-case/issues) — competitor fragility
- [nextest documentation](https://nexte.st/) — test runner design

### Rust ecosystem
- [Iterating on Testing in Rust](https://epage.github.io/blog/2023/06/iterating-on-test/) — Rust testing gaps
- [Delete Cargo Integration Tests](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html) — compile time
- [Finish libtest JSON output](https://rust-lang.github.io/rust-project-goals/2025h1/libtest-json.html) — Rust Project Goals 2025 H1
- [Custom test framework RFC 2318](https://rust-lang.github.io/rfcs/2318-custom-test-frameworks.html) — stalled since 2018
- [RFC: setup/teardown/fixture attributes](https://github.com/rust-lang/rust/issues/117668) — proposed, not accepted
- [Rust testing-devex-team](https://github.com/rust-lang/testing-devex-team/issues/2) — upstream improvements
- [2025 Rust Compiler Performance Survey](https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results/) — compile time is #1 pain

### Developer productivity research
- [Google Testing Blog: Flaky Tests](https://testing.googleblog.com/2016/05/flaky-tests-at-google-and-how-we.html) — 84% of failures are flaky
- [Flaky Test Benchmark 2026](https://testdino.com/blog/flaky-test-benchmark/) — cost data
- [Atlassian State of DevEx 2025](https://www.atlassian.com/blog/developer/developer-experience-report-2025) — 50% lose 10+ hrs/week to non-coding
- [Develocity Predictive Test Selection](https://gradle.com/develocity/product/predictive-test-selection/) — 70-90% time reduction
- [How much time debugging?](https://thenewstack.io/how-much-time-do-developers-spend-actually-writing-code/) — 30-50% of dev time
- [Improve test error messages](https://kentcdodds.com/blog/improve-test-error-messages-of-your-abstractions) — error message design

### Innovation inspirations
- [Playwright Trace Viewer](https://playwright.dev/docs/trace-viewer) — time-travel test debugging
- [Deno test sanitizers](https://docs.deno.com/runtime/fundamentals/testing/) — resource leak detection
- [Bazel hermeticity](https://bazel.build/basics/hermeticity) — structural test isolation
- [cargo-mutants](https://mutants.rs/) — mutation testing for Rust
- [Pact contract testing](https://docs.pact.io/) — consumer-driven contracts

### Domain testing & matcher design
- [trybuild](https://docs.rs/trybuild/latest/trybuild/) — compile-failure testing for proc macros
- [static_assertions](https://docs.rs/static_assertions/latest/static_assertions/) — compile-time property checks
- [dhat heap usage testing](https://nnethercote.github.io/perf-book/heap-allocations.html) — allocation counting
- [serial_test](https://crates.io/crates/serial_test) — sequential test execution (4.5M dl/mo)
- [tracing-test](https://docs.rs/tracing-test/latest/tracing_test/) — per-test tracing subscriber
- [tracing-capture](https://crates.io/crates/tracing-capture) — structured span/event inspection
- [speculoos assertion modules](https://docs.rs/speculoos/latest/speculoos/) — path, hashset, iter matchers
- [Structuring and testing proc macros (Ferrous Systems)](https://ferrous-systems.com/blog/testing-proc-macros/) — compile-fail patterns
- [Properly Testing Concurrent Data Structures (matklad)](https://matklad.github.io/2024/07/05/properly-testing-concurrent-data-structures.html) — loom/shuttle

### Developer experience research
- [Swift Testing: #expect macro](https://developer.apple.com/xcode/swift-testing/) — expression decomposition gold standard
- [Catch2 assertion decomposition](https://github.com/catchorg/Catch2/blob/devel/docs/assertions.md) — C++ template-based expression capture
- [Vitest agent reporter PR](https://github.com/vitest-dev/vitest/pull/9779) — AI-optimized test output
- [OSC 8 Hyperlinks specification](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda) — clickable terminal links
- [Leptos DX guide](https://book.leptos.dev/getting_started/leptos_dx.html) — proc macro IDE configuration
- [Kotest assertions (350+ matchers)](https://kotest.io/docs/assertions/assertions.html) — matcher breadth benchmark
- [ExUnit doctests](https://hexdocs.pm/ex_unit/ExUnit.DocTest.html) — docs-as-tests pattern
- [Go synctest](https://go.dev/blog/synctest) — virtual time for concurrent testing
- [Nextest input handling](https://nexte.st/docs/design/architecture/input-handling/) — raw terminal keypresses
- [Nextest configuration reference](https://nexte.st/docs/configuration/reference/) — layered config model
- [CTRF GitHub Test Reporter](https://github.com/ctrf-io/github-test-reporter) — comprehensive CI reporting
- [rust-analyzer proc macro support](https://github.com/rust-lang/rust-analyzer/issues/11014) — IDE limitations
- [Proc macro expansion caching](https://www.coderemote.dev/blog/faster-rust-compiler-macro-expansion-caching/) — compile time optimization
- [Nicholas Nethercote: -Zmacro-stats](https://nnethercote.github.io/2025/06/26/how-much-code-does-that-proc-macro-generate.html) — codegen volume analysis
- [Edition 2027: Stop the tests!](https://internals.rust-lang.org/t/edition-2027-stop-the-tests/23187) — compilation time frustration
- [RFC: setup/teardown/fixture attributes](https://github.com/rust-lang/rust/issues/117668) — 69.6M rstest downloads prove demand
- [pretty_assertions 96.5GB allocation](https://github.com/rust-pretty-assertions/rust-pretty-assertions/issues/124) — why truncation matters
- [Markus Unterwaditzer: Test Parametrization Guide](https://unterwaditzer.net/2023/rust-test-parametrization.html) — boilerplate analysis
- [2025 State of Rust Survey](https://blog.rust-lang.org/2026/03/02/2025-State-Of-Rust-Survey-results/) — 55% wait >10s for rebuilds
