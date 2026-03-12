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

#### Migration guides

Lower the adoption barrier:
- **From `assert_eq!`:** side-by-side comparison, incremental adoption path
- **From rstest:** how to convert `#[fixture]` + `#[case]` to `setup` + `each`
- **From test-case:** how to convert `#[test_case]` to `each` / `matrix`
- **From googletest:** matcher equivalence table

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
| **v1.0.0** | Stability | API freeze, migration guides, STABILITY.md, performance baseline |

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
| Parameterization breadth | rstest | Cartesian `#[values]` | v0.6 `matrix` |
| Test filtering | pytest, RSpec | Arbitrary metadata tags | v0.7 `tag` |
| Runtime skip | pytest | `@pytest.mark.skip` | v0.7 `skip_when!` |
| Watch mode | Vitest, Jest | Auto-rerun on save | v0.7 `--watch` |
| Typed tests | Google Test | `TYPED_TEST_SUITE` | v0.8 `each_type` |
| Pattern matching | (none) | Rust-native patterns | v0.8 `to_match_pattern!` |
| CI retry | nextest | `--retries` | v0.9 `--retry` |
| Filter expressions | nextest | Structured queries | v0.9 `--filter` |

### Risks

| Risk | Mitigation |
|------|-----------|
| rstest absorbs BDD-like features | Lean into cohesion — rstest will never have a DSL |
| googletest-rust gains traction | Matcher parity already exists; DSL + CLI are the moat |
| Compile time grows with macro complexity | Budget enforcement, codegen optimization in v0.9 |
| Single-maintainer risk (like rstest) | Document everything, keep architecture simple |
| Custom test framework RFC stabilizes | behave already works without it; stabilization would be a tailwind |

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

- [Thoughtbot: "Let's Not"](https://thoughtbot.com/blog/lets-not) — RSpec `let` problems
- [Solnic: "5 Rules of Simple RSpec Tests"](https://solnic.dev/the-5-rules-of-simple-rspec-tests/) — anti-patterns
- [The Case Against Shared Examples](https://dev.to/epigene/the-case-against-shared-examples-39kh) — RSpec shared examples
- [pytest fixture discussion](https://github.com/pytest-dev/pytest/discussions/11085) — fixture problems
- [Software Testing Anti-patterns](https://blog.codepipes.com/testing/software-testing-antipatterns.html) — framework design
- [What's Wrong with Snapshot Tests](https://medium.com/@sapegin/whats-wrong-with-snapshot-tests-37fbe20dfe8e) — Jest snapshots
- [Iterating on Testing in Rust](https://epage.github.io/blog/2023/06/iterating-on-test/) — Rust testing gaps
- [Delete Cargo Integration Tests](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html) — compile time
- [rstest issue tracker](https://github.com/la10736/rstest/issues) — competitor gaps
- [test-case issue tracker](https://github.com/frondeus/test-case/issues) — competitor fragility
- [nextest documentation](https://nexte.st/) — test runner design
