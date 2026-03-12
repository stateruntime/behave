# Feature Competition Analysis

## Part 1: What behave IS

behave is a **BDD-style test authoring library** for Rust. Core value proposition:

> Write test suites as readable scenario trees that compile to ordinary `#[test]` functions.

### Identity pillars

1. **String-label DSL** — tests described in prose, not function names
2. **Zero runtime** — `behave!` is a proc macro; output is standard `#[test]`/`#[tokio::test]`
3. **Setup inheritance** — parent `setup` bindings flow into children automatically
4. **Result-based assertions** — `expect!(x).to_equal(y)?` uses `?`, not panics
5. **Optional CLI** — `cargo behave` for tree/JSON/JUnit output + flaky detection

### Current feature surface (v0.5.0)

| Area | What exists |
|------|-------------|
| DSL keywords | Groups, tests, `setup`, `teardown`, `tokio;`, `timeout`, `pending`, `focus`, `each` |
| Matchers (27+) | equality, boolean, ordering, option, result, collections (Vec + slice), strings, float, HashMap/BTreeMap, regex, predicate, custom trait |
| Negation | `.not()` / `.negate()` on any matcher |
| Combinators | `all_of`, `any_of`, `not_matching` — recursive composition |
| Parameterized | `each [cases] \|params\| { body }` → `case_N` tests |
| Async | `tokio;` group marker → `#[tokio::test]` |
| Soft assertions | `SoftErrors` — collect multiple failures, report together |
| Timeout | `timeout <ms>;` — deadline enforcement with nesting inheritance |
| Color diffs | `color` feature — ANSI diffs via `similar` crate |
| CLI outputs | Tree (colored), JSON, JUnit XML |
| CLI features | Flaky detection with history, workspace-aware package selection |
| Error type | `MatchError` with expression, actual, expected, negated flag |

---

## Part 2: What behave IS NOT

### Deliberately excluded (architectural decisions)

| What | Why excluded |
|------|-------------|
| Custom test runner | Uses `cargo test`/libtest; no harness to maintain |
| before-all / after-all | Fights "ordinary tests" promise; per-test setup suffices |
| Inference-based injection | Breaks the simple "paste setup code" expansion model |
| Full attribute-style API | Dilutes the string-label tree identity |
| Mocking/stubbing | Orthogonal concern (use mockall, wiremock) |
| Property testing | Different paradigm (use proptest, quickcheck) |
| Snapshot testing | Planned integration with insta, not reinventing |
| Benchmarking | Different concern (use criterion) |

### Gaps users might expect

| Gap | Severity | Who has it |
|-----|----------|-----------|
| ~~No colored diff output on failure~~ | ~~High~~ | **DONE v0.3.0** — `color` feature |
| ~~No regex matchers~~ | ~~Medium~~ | **DONE v0.3.0** — `regex` feature |
| ~~No matcher composition~~ | ~~Medium~~ | **DONE v0.4.0** — `all_of`, `any_of`, `not_matching` |
| ~~No soft assertions~~ | ~~Medium~~ | **DONE v0.5.0** — `SoftErrors` |
| ~~No HashMap/BTreeMap matchers~~ | ~~Medium~~ | **DONE v0.4.0** — `std` feature |
| ~~No test timeout~~ | ~~Medium~~ | **DONE v0.5.0** — `timeout <ms>;` DSL keyword |
| No matrix/combinatorial params | Medium | rstest (`#[values]`), test-case (`#[test_matrix]`) — **Planned v0.6** |
| No tag-based filtering | Medium | RSpec, pytest, Google Test — **Planned v0.7** |
| No runtime conditional skip | Medium | pytest (`@pytest.mark.skip`) — **Planned v0.7** |
| No watch mode | Medium | Vitest, Jest — **Planned v0.7** |
| No typed test generation | Medium | Google Test (`TYPED_TEST`) — **Planned v0.8** |
| No pattern matching matcher | Low | (unique opportunity) — **Planned v0.8** |
| No CLI retry | Low | nextest `--retries` — **Planned v0.9** |
| No CLI filter expressions | Low | nextest filter sets — **Planned v0.9** |

---

## Part 3: Competitive Landscape

> **Note (2026-03-12):** Download counts drift quickly and different sites report
> different metrics (total vs monthly). Treat the numbers below as rough
> signals, and prefer `lib.rs` pages for current snapshots.

### Direct competitors (BDD-style DSL)

| Crate | Downloads | Status | vs behave |
|-------|-----------|--------|-----------|
| **spectacular** | (new) | Active | Strong hooks + context injection + async runtime support |
| **rsspec** | (new) | Active | RSpec/Ginkgo-inspired; built-in retries/timeouts/focus controls |
| **rstest-bdd** | (new) | Active | Gherkin BDD + `rstest` fixtures (acceptance-test shape) |
| **fluent-test** | (new) | Early | Jest-style `describe/it/expect` with hooks + roadmap |
| **speculate2** | ~6K | Low activity | behave wins: richer matchers, CLI, async surface |
| **speculate** | ~50K | Low activity | behave wins: active, richer, parameterized |
| **rspec** | ~8K | Dead | behave wins on every dimension |
| **cucumber** | 1.1M | Active | Different paradigm: external Gherkin files + step defs |

**Conclusion (updated):** behave no longer owns the BDD macro niche by default.
There are now multiple actively-maintained "nested tests / RSpec-like" crates,
and at least one "Gherkin + fixtures" option. The differentiator is no longer
"is there a BDD DSL?", but:

- **How it composes with the Rust ecosystem** (`cargo test`, nextest, IDEs)
- **How it handles workflow** (focus/tags/filtering, retries, timeouts, watch)
- **How it handles shared state** (explicit setup vs hooks/fixtures)
- **How good failures are** (matchers, diffs, source locations, ergonomics)

### Competitor deep dives (what they optimize for)

This section is intentionally practical: what a team *actually gets* by choosing
each tool, where it creates lock-in, and what it implies for behave's roadmap.

#### spectacular (RSpec-like suites with hooks and context)

**Positioning:** "RSpec-like testing in Rust", with both a DSL and an
attribute-style API.

**What it does well:**
- Rich hook surface (`before_*`/`after_*`) at multiple levels (suite/group/test)
- Context injection: hooks can "provide" values that tests receive, reducing
  boilerplate in large suites
- Async test support with selectable runtimes (e.g. tokio / async-std)

**Implications for behave:**
- Hook-based shared state is attractive, but it tends to reintroduce "magic".
  behave's differentiation should stay: *explicit setup blocks that paste into
  generated tests*, plus clear docs for `OnceLock`/explicit shared resources.
- Where spectacular can win: teams that want `before_all`/`after_all` semantics
  and dependency injection patterns in tests.

**Links:** docs: <https://spectacular.vercel.app/> (also see docs.rs).

#### rsspec (BDD suites with runner-style workflow knobs)

**Positioning:** Inspired by Ginkgo/RSpec. Offers two modes:
- generate ordinary `#[test]` functions for `cargo test`
- or run as a `harness = false` binary for richer tree output

**What it does well:**
- Workflow knobs users request constantly: retries, timeouts, focus enforcement,
  per-suite repeat-until-pass patterns
- Runtime skipping / filtering (often ergonomic for large suites)
- Optional integration with a large matcher ecosystem via re-exporting
  googletest matchers

**Implications for behave:**
- behave's planned v0.9 "CLI retry" becomes more urgent because a direct BDD
  competitor already ships workflow primitives (even if via its own runner).
- This reinforces that *workflow features are adoption features*, not polish.

**Links:** docs.rs `rsspec` crate page.

#### rstest-bdd (Gherkin BDD plus rstest fixtures)

**Positioning:** BDD tests written in Gherkin (feature files / Given-When-Then),
implemented with step definitions and powered by `rstest` fixtures.

**What it does well:**
- High readability for non-Rust stakeholders (feature files)
- Strong story for fixtures/shared resources through rstest
- Natural fit for "acceptance test" style

**Implications for behave:**
- behave is *code-first BDD*; rstest-bdd/cucumber are *feature-file BDD*.
  behave should not chase feature-file parity; instead, document when behave is
  the better fit (domain tests, fast iteration, no step-def ceremony).

**Links:** crate docs and repo: <https://lib.rs/crates/rstest-bdd>.

#### fluent-test (Jest-style macros: describe/it/expect)

**Positioning:** Brings the Jest mental model (`describe`/`it`/`expect`) into
Rust tests.

**What it does well:**
- Familiar surface for developers coming from JS/TS
- Has hook concepts (`before_all`, `after_all`) as part of the design

**Implications for behave:**
- Highlights demand for "workflow affordances" (watch mode, skip/focus, hooks).
- behave's moat remains: matchers, CLI, and "ordinary `#[test]`" generation.

**Links:** docs.rs `fluent-test` crate page.

#### speculate / speculate2 / rspec (macro DSL predecessors)

These older crates established the "RSpec-like macros" shape in Rust, but
generally have limited feature surfaces and/or low activity. They remain useful
signals for what Rust developers *expect* a nested DSL to look like.

### Indirect competitors (assertion + parameterization)

| Crate | Downloads | What it does | Threat level |
|-------|-----------|-------------|-------------|
| **rstest** | 5.9M | Fixtures, `#[case]`, `#[values]` matrix, `#[timeout]`, async | **High** — most popular parameterized testing |
| **test-case** | 3.7M | `#[test_case]`, `#[test_matrix]` Cartesian product | Medium — params only, no matchers |
| **googletest** | 467K | 40+ matchers, `all!/any!`, soft assertions, field matchers | **High** — richest matcher library |
| **pretty_assertions** | 8.8M | Colored diff for `assert_eq!` | Medium — output quality only |
| **insta** | 5.4M | Snapshot testing with review workflow | Low — complementary, not competitive |

### Indirect competitor deep dives (the "best-of-breed stack")

Most real-world Rust codebases do not adopt a single integrated framework. They
assemble a stack:

- **fixtures/parameterization** (rstest/test-case)
- **assertions/matchers** (googletest/speculoos/assert2/etc.)
- **runner/workflow** (nextest, CI tooling)

behave competes with this stack by reducing mental overhead ("one DSL, one
import"), not by claiming these tools are bad. The question is: how much
cohesion value does behave provide, and where does it still need to match
baseline expectations?

#### rstest (fixtures + parameterization)

**What it optimizes for:** attribute-driven fixtures and powerful
parameterization.

**Why teams adopt it:**
- Handles table-driven tests and fixtures with very little ceremony
- Widely-used defaults and ecosystem familiarity

**Where behave must compete:**
- Parameterization ergonomics (matrix tests, named cases, readable output)
- Compile-time and IDE experience at scale
- Clear guidance on shared resources (especially in parallel runners)

**Useful references:** <https://docs.rs/rstest> / <https://github.com/la10736/rstest>

#### test-case (parameterization-only, minimal surface)

**What it optimizes for:** "drop-in parameterized tests" without a broader test
framework.

**Why teams adopt it:**
- Very small conceptual surface: "write the same test with different inputs"
- Matrix support (`#[test_matrix]`) with a simple story

**Where behave must compete:**
- Provide matrix generation and named cases inside `behave!` (planned v0.6)
- Avoid ecosystem footguns (test-case's `#[test_case]` name collides with a
  nightly built-in attribute; MSRV policy is "latest stable")

**Useful references:** <https://docs.rs/test-case>

#### googletest / speculoos / assert2 (matchers and failure quality)

**What they optimize for:** high-signal failures and expressive matchers.

**Why teams adopt them:**
- Matchers and composition that make failures more diagnosable than `assert_eq!`
- Domain-specific helpers (e.g., JSON/path matchers in some ecosystems)

**Where behave must compete:**
- Error message clarity and diffs for large/complex values
- Breadth of "daily driver" matchers (JSON/object partial matching, paths, etc.)

**Useful references:**
- googletest: <https://docs.rs/googletest>
- speculoos: <https://docs.rs/speculoos>
- assert2: <https://docs.rs/assert2>

#### cargo-nextest (runner + workflow layer)

**What it optimizes for:** running large Rust test suites quickly and
reliably (process isolation, retries, timeouts, reporting, filtering).

**Why teams adopt it:**
- Strong workflow ergonomics for big suites (retries, filters, profiles)
- CI-grade reporting (e.g., JUnit)

**Where behave must compete:**
- Avoid forcing a separate runner/harness; behave's "ordinary `#[test]`" output
  is a strategic advantage.
- Make `cargo-behave` clearly complementary (tree view, suite semantics, flaky
  history) rather than a confusing alternative runner.

**Useful references:** <https://nexte.st/>

### Test runners (complementary)

| Crate | What it does |
|-------|-------------|
| **cargo-nextest** | Process-per-test, retries, JUnit, profiles, timeouts |
| **libtest-mimic** | Custom test harness matching libtest |

### The real competitive threat

The combo of **rstest + googletest + pretty_assertions** gives users:
- Fixtures and parameterization (rstest)
- 40+ matchers with composition (googletest)
- Colored diff output (pretty_assertions)

But requires learning 3 crates and mixing attribute styles.

**behave's moat is cohesion**: one DSL, one import, one mental model.

**Updated threat model:** there is now also an "integrated BDD suite" competitor
cluster (e.g. spectacular/rsspec) that tries to offer the cohesion story too.
behave must win by being:

- the most ecosystem-compatible (cargo test / nextest / IDEs)
- the most explicit (setup visible; no fixture injection)
- the best at failure output (messages + diffs)
- the fastest to compile at scale (measured and enforced)

---

## Part 4: Prioritized Feature Roadmap

### Completed

- [x] **#1 Parameterized tests (`each`)** — **DONE v0.2.0**
- [x] **#2 Colored diff output** — **DONE v0.3.0**
- [x] **#3 Regex string matchers** — **DONE v0.3.0**
- [x] **#4 Matcher composition (`all_of`, `any_of`, `not_matching`)** — **DONE v0.4.0**
- [x] **#5 HashMap/BTreeMap matchers** — **DONE v0.4.0**
- [x] **#6 Soft assertions (`SoftErrors`)** — **DONE v0.5.0**
- [x] **#7 Test timeout (`timeout <ms>;`)** — **DONE v0.5.0**

### Planned — see [ROADMAP.md](ROADMAP.md) for full details

| Version | Theme | Key Features |
|---------|-------|-------------|
| **v0.6.0** | Better Parameterization | `matrix` Cartesian product, named `each` cases, `xfail` |
| **v0.7.0** | Tags & Filtering | `tag` metadata, `skip_when!`, watch mode |
| **v0.8.0** | Type Power | `each_type`, `to_match_pattern!`, partial struct matching |
| **v0.9.0** | Polish & Robustness | failure output audit, compile-time budget, CLI filters, retry |
| **v1.0.0** | Stability | API freeze, migration guides, STABILITY.md, performance baseline |

### Deliberately excluded — see [ROADMAP.md § What We Will NOT Build](ROADMAP.md#what-we-will-not-build)

| Feature | Why Not |
|---------|---------|
| `let`-style lazy bindings | RSpec's biggest regret — mystery guests |
| Shared examples | Anti-pattern: ghost variables, debugging nightmares |
| `before_all` / `after_all` | Fights "ordinary tests" promise, breaks in parallel |
| Fixture injection (pytest-style) | Indirection + coupling at scale |
| Mocking | Orthogonal concern (use mockall) |
| Property-based testing | Orthogonal paradigm (use proptest) |
| Snapshot testing (built-in) | "Blind update" anti-pattern at scale |
| Plugin architecture | Too early, constrains internal evolution |
| Custom test runner | Would destroy the `#[test]` compatibility moat |

---

## Part 5: Matcher Competition & Expansion Opportunities

behave is both a **suite authoring DSL** and an **assertion/matcher library**.
The DSL is the differentiator, but matchers are what users touch every day. In
practice, a "globally used" testing library needs a set of matchers that covers
the common shapes developers assert on:

- collections (ordered + unordered)
- sets and maps
- strings (including normalization)
- paths and filesystem state
- results/options/errors
- JSON (when dealing with APIs)
- floats (including NaN/Inf)

This section captures what other ecosystems treat as baseline, what Rust crates
already provide today, and what behave should expand into.

### Baseline matcher expectations (from other ecosystems)

Rust isn't the only market. Developers arrive with expectations shaped by other
testing frameworks:

- **GoogleTest matchers:** rich container matchers (ordered/unordered),
  string matchers (substring/regex), float "near"/approx matchers, and wrapper
  matchers for `Option`/`Result` that match the inner value. (See the Rust port
  for a concrete matcher inventory.)  
  References: googletest-rust matchers module. <https://docs.rs/googletest/latest/googletest/matchers/index.html>
- **Hamcrest style:** a predictable set of matchers for numeric closeness, file
  paths (exists/file/dir), option/result wrappers, and collection matchers like
  "contains exactly" or "contains in order".  
  References: `hamcrest2`. <https://docs.rs/hamcrest2/latest/hamcrest2/>
- **Jest expect:** matchers like `toMatchObject` are extremely common in API
  and JSON-heavy testing because they enable "assert only the fields you care
  about".  
  References: Jest `expect` docs. <https://jestjs.io/docs/expect>

### What Rust matcher libraries ship today (signals)

Rust already has several matcher/assertion libraries that exist largely because
the standard library's `assert!`/`assert_eq!` surface is intentionally small:

- **speculoos:** fluent, type-directed assertions with dedicated modules for
  `hashset`, `iter`, `path`, `numeric`, and `json` (optional).  
  References: speculoos modules list. <https://docs.rs/speculoos/latest/speculoos/>
- **assert2:** assertion/check macros that parse expressions, support pattern
  matches, and provide `check!` (multiple checks) plus `let_assert!` (pattern
  capture). This validates demand for pattern-based assertions and soft-check
  workflows.  
  References: assert2 crate docs. <https://docs.rs/assert2/latest/assert2/>
- **assert_json_diff:** JSON diffing plus partial inclusion checks (assert that
  "expected is included in actual").  
  References: assert-json-diff docs. <https://docs.rs/assert-json-diff/latest/assert_json_diff/>
- **googletest-json-serde:** dedicated JSON matchers for `serde_json::Value`,
  including path-value matching and ordered/unordered array matchers.  
  References: googletest-json-serde docs. <https://docs.rs/googletest-json-serde/latest/googletest_json_serde/>
- **json_test:** JSONPath-focused assertions for JSON APIs (a strong signal
  that path-based JSON assertions are demanded).  
  References: json_test docs. <https://docs.rs/json_test/latest/json_test/>
- **assert_fs:** filesystem fixtures and assertions (TempDir + file creation +
  validation). This is not a matcher library per se, but it shows how common
  "path exists / file contains / directory layout" checks are in real tests.  
  References: assert_fs docs. <https://docs.rs/assert_fs/latest/assert_fs/>

### behave’s current matcher surface (v0.5.0)

Today behave covers the core types well:

- equality, booleans, ordering
- `Option` / `Result` structure matchers + equality on inner values
- vectors and slices (empty/len/contains/contains-all)
- strings (starts/ends/contains/byte-length)
- floats (epsilon-based approx)
- maps (`HashMap`/`BTreeMap`) behind `std`
- regex matchers behind `regex`
- custom predicates (`to_satisfy`) + custom matcher trait (`BehaveMatch`)

The most important remaining gaps are around *daily-driver* assertions that
appear constantly in large Rust codebases: paths, errors, sets, richer sequence
matchers, and JSON.

### High-leverage matcher expansions (recommended)

These additions aim for maximal DevEx impact with minimal conceptual overhead.

#### 1) Richer sequence matchers (Vec/slice)

Competitors treat these as table stakes:

- ordered matching (`elements_are` / "contains in order")
- unordered matching (`unordered_elements_are` / "contains exactly in any order")
- subset/superset style checks (`subset_of` / `superset_of`)

Proposed families for behave:

- **Ordered:** `to_contain_exactly(&[...])`, `to_start_with_elements(&[...])`,
  `to_end_with_elements(&[...])`
- **Unordered:** `to_contain_exactly_in_any_order(&[...])` (multiset semantics)
- **Sorting:** `to_be_sorted()`, `to_be_sorted_by_key(f)` for slice/vec types

Why it matters: parameterized tests and data-heavy tests frequently produce
collections; users care about **precise failure messages** for "what's missing"
and "what's extra".

Competitor evidence:
- googletest-rust has `elements_are!` and `unordered_elements_are!`. <https://docs.rs/googletest/latest/googletest/matchers/index.html>
- hamcrest2 explicitly calls out "contains exactly" and "contains in order". <https://docs.rs/hamcrest2/latest/hamcrest2/>

#### 2) Set matchers (`HashSet` / `BTreeSet`)

Rust users routinely compare sets when order is irrelevant. speculoos dedicates
an entire module to `hashset`, and googletest matchers include subset/superset
concepts.

Proposed families:

- `to_contain(x)` / `to_contain_all_of(&[...])`
- `to_be_subset_of(&set)` / `to_be_superset_of(&set)`
- `to_have_length(n)` / `to_be_empty()` / `to_not_be_empty()`

Competitor evidence:
- speculoos lists a `hashset` assertion module. <https://docs.rs/speculoos/latest/speculoos/>
- googletest-rust has `subset_of` / `superset_of`. <https://docs.rs/googletest/latest/googletest/matchers/index.html>

#### 3) Path and filesystem matchers (`Path` / `PathBuf`)

Tests touch the filesystem constantly (CLI tools, compilers, generators,
configuration loaders). hamcrest2 and predicate libraries expose path-exists
matchers directly; assert_fs exists as an entire crate for this workflow.

Proposed (metadata-only) matchers (std-only):

- `to_exist()`, `to_not_exist()`
- `to_be_file()`, `to_be_dir()`
- `to_be_absolute()`, `to_be_relative()`
- `to_have_extension("toml")`, `to_have_file_name("Cargo.toml")`

Optional (I/O) matchers (still `std`, but reads files):

- `to_have_contents(str)` / `to_contain_line(str)` (careful with large files)

Competitor evidence:
- hamcrest2 documents `path_exists`, `file_exists`, `dir_exists`. <https://docs.rs/hamcrest2/latest/hamcrest2/>
- assert_fs positions itself around "fixtures and assertions for testing". <https://docs.rs/assert_fs/latest/assert_fs/>

#### 4) Error-chain matchers (std errors + anyhow/eyre optional)

In Rust, errors are data. Tests frequently assert:

- error "kind" or type (downcasting)
- error display string contains something
- error has a specific source/root cause

Proposed families:

- For `Result<T, E>` where `E: std::error::Error`:
  `to_be_err_with_display_containing("...")`, `to_have_source_display_containing("...")`
- For `anyhow::Error` / `eyre::Report` behind optional features:
  `to_be_err_downcast::<MyError>()`, `to_have_any_cause::<MyError>()`

Why it matters: without dedicated error matchers, users write ad-hoc string
contains checks or `matches!` blocks, and failures become hard to read.

References:
- `std::error::Error::source` (error chaining). <https://doc.rust-lang.org/std/error/trait.Error.html>
- anyhow error chain accessors. <https://docs.rs/anyhow/latest/anyhow/struct.Error.html>

#### 5) String normalization matchers

String assertions often fail on whitespace/casing rather than semantics.
Frameworks like Hamcrest ship "contains string" and regex matchers; other
ecosystems also normalize whitespace/case.

Proposed:

- `to_be_empty()` / `to_not_be_empty()` for strings
- `to_have_char_count(n)` (Unicode scalar count) alongside byte-length
- `to_equal_ignoring_case(expected)` and/or `to_equal_ignoring_whitespace(expected)`

Competitor evidence:
- googletest-rust includes `char_count` and multiple string matchers. <https://docs.rs/googletest/latest/googletest/matchers/index.html>

#### 6) Better `Option` / `Result` inner matching (beyond `PartialEq`)

Current behave `to_be_ok_with` / `to_be_some_with` require `PartialEq`. Many
tests instead want:

- "Ok value satisfies predicate"
- "Err value matches shape"
- "Some(inner matcher)"

Proposed:

- `to_be_ok_and(|v| ...)`, `to_be_err_and(|e| ...)`, `to_be_some_and(|v| ...)`
- `to_be_ok_matching(matcher)` / `to_be_some_matching(matcher)` using
  `BehaveMatch<T>` for reusable domain checks

Competitor evidence:
- googletest-rust matchers like `ok(inner)`, `err(inner)`, `some(inner)`. <https://docs.rs/googletest/latest/googletest/matchers/index.html>

#### 7) Float shape matchers (NaN/Inf/finite)

behave already has epsilon comparison. Add the common float shape matchers:

- `to_be_nan()`, `to_be_infinite()`, `to_be_finite()`

Competitor evidence:
- googletest-rust includes `is_nan`, `is_infinite`, `is_finite`. <https://docs.rs/googletest/latest/googletest/matchers/index.html>

### JSON matchers (feature-flagged, high ROI)

The Rust ecosystem has multiple JSON assertion crates because API tests are
common and JSON comparisons are painful.

behave already plans "partial struct matching" behind a `serde` feature. The
research suggests extending that family rather than stopping at a single
matcher:

- `to_contain_fields(expected_json)` — partial object match (planned)
- `to_have_json_pointer("/a/b/0", expected)` — precise path/value assertions
- (optional) JSONPath matchers if users request them strongly (note existing
  crates like `json_test`)

Competitor evidence:
- assert-json-diff provides "partial matching" (`assert_json_include!`). <https://docs.rs/assert-json-diff/latest/assert_json_diff/>
- googletest-json-serde demonstrates path matching + array matchers. <https://docs.rs/googletest-json-serde/latest/googletest_json_serde/>
- json_test focuses on JSONPath assertions. <https://docs.rs/json_test/latest/json_test/>

### Matchers vs DSL scope (what not to build)

Even with matcher expansion, behave should stay disciplined:

- Avoid baking in domain-specific matchers (HTTP, SQL, etc.) unless they are
  feature-flagged integrations with widely used crates.
- Keep dependencies optional (feature flags) to protect compile times and MSRV.
- Prefer a few matchers with excellent failure messages over a huge surface
  with mediocre output.

## Sources

- [lib.rs testing category](https://lib.rs/development-tools/testing)
- [spectacular docs](https://spectacular.vercel.app/)
- [rsspec on docs.rs](https://docs.rs/rsspec)
- [rstest-bdd on lib.rs](https://lib.rs/crates/rstest-bdd)
- [fluent-test on docs.rs](https://docs.rs/fluent-test)
- [rstest docs](https://docs.rs/rstest) / [rstest GitHub](https://github.com/la10736/rstest)
- [googletest-rust matchers](https://docs.rs/googletest/latest/googletest/matchers/index.html) / [GitHub](https://github.com/google/googletest-rust)
- [pretty_assertions](https://crates.io/crates/pretty_assertions)
- [test-case](https://docs.rs/test-case)
- [insta](https://insta.rs/)
- [cargo-nextest](https://nexte.st/)
- [speculate2](https://lib.rs/crates/speculate2)
- [cucumber](https://lib.rs/crates/cucumber)
- [speculoos](https://docs.rs/speculoos/latest/speculoos/)
- [hamcrest2](https://docs.rs/hamcrest2/latest/hamcrest2/)
- [assert2](https://docs.rs/assert2/latest/assert2/)
- [assert-json-diff](https://docs.rs/assert-json-diff/latest/assert_json_diff/)
- [googletest-json-serde](https://docs.rs/googletest-json-serde/latest/googletest_json_serde/)
- [json_test](https://docs.rs/json_test/latest/json_test/)
- [assert_fs](https://docs.rs/assert_fs/latest/assert_fs/)
- [anyhow::Error](https://docs.rs/anyhow/latest/anyhow/struct.Error.html)
- [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html)
- [Jest expect](https://jestjs.io/docs/expect)
