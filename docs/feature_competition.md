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

### Current feature surface (v0.2.0)

| Area | What exists |
|------|-------------|
| DSL keywords | Groups, tests, `setup`, `teardown`, `tokio;`, `pending`, `focus`, `each` |
| Matchers (21) | equality, boolean, ordering, option, result, collections (Vec + slice), strings, float, predicate, custom trait |
| Negation | `.not()` / `.negate()` on any matcher |
| Parameterized | `each [cases] \|params\| { body }` → `case_N` tests |
| Async | `tokio;` group marker → `#[tokio::test]` |
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
| No colored diff output on failure | High | pretty_assertions (8.8M downloads) |
| No regex matchers | Medium | googletest (`matches_regex`, `contains_regex`) |
| No matcher composition (`all!`, `any!`) | Medium | googletest (`all!`, `any!`, `not!`) |
| No soft assertions (collect all failures) | Medium | googletest (`expect_that!`) |
| No HashMap/BTreeMap matchers | Medium | googletest (`has_entry`) |
| No test timeout | Medium | rstest (`#[timeout]`), nextest config |
| No matrix/combinatorial params | Medium | rstest (`#[values]`), test-case (`#[test_matrix]`) |
| No fixture reuse across files | Low | rstest (`#[fixture]` importable) |
| No field/property matchers | Low | googletest (`field!`, `property!`) |
| No watch mode | Low | External tools (bacon, cargo-watch) |
| No CLI retry | Low | nextest `--retries` |

---

## Part 3: Competitive Landscape

### Direct competitors (BDD-style DSL)

| Crate | Downloads | Status | vs behave |
|-------|-----------|--------|-----------|
| **speculate** | ~50K | Dead (2018) | behave wins: active, richer, parameterized |
| **speculate2** | ~6K | Low activity | behave wins: more matchers, CLI, async |
| **rspec** | ~8K | Dead | behave wins on every dimension |
| **cucumber** | 1.1M | Active | Different paradigm: external Gherkin files + step defs |

**Conclusion**: behave owns the "BDD macro" niche. No active competition.

### Indirect competitors (assertion + parameterization)

| Crate | Downloads | What it does | Threat level |
|-------|-----------|-------------|-------------|
| **rstest** | 5.9M | Fixtures, `#[case]`, `#[values]` matrix, `#[timeout]`, async | **High** — most popular parameterized testing |
| **test-case** | 3.7M | `#[test_case]`, `#[test_matrix]` Cartesian product | Medium — params only, no matchers |
| **googletest** | 467K | 40+ matchers, `all!/any!`, soft assertions, field matchers | **High** — richest matcher library |
| **pretty_assertions** | 8.8M | Colored diff for `assert_eq!` | Medium — output quality only |
| **insta** | 5.4M | Snapshot testing with review workflow | Low — complementary, not competitive |

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

---

## Part 4: Prioritized Feature Roadmap

### P0 — High impact, addresses direct competitive gaps

- [x] **#1 Parameterized tests (`each`)** — closes gap against rstest `#[case]` — **DONE v0.2.0**
- [x] **#2 Colored diff output** — closes gap against pretty_assertions — **DONE v0.3.0**
- [x] **#3 Regex string matchers** — closes gap against googletest `matches_regex` — **DONE v0.3.0**
- [ ] **#4 Matcher composition (`all!`, `any!`)** — closes gap against googletest combinators — Medium effort (new macros or methods)
- [ ] **#5 Soft assertions mode** — closes gap against googletest `expect_that!` — Large effort (collect errors, report all)

### P1 — Medium impact, differentiators

- [ ] **#6 Test timeout** — prevents hanging tests — Medium effort (DSL + codegen)
- [ ] **#7 HashMap/BTreeMap matchers** — common data structures — Small effort (3-4 matchers)
- [ ] **#8 Snapshot testing** (`to_match_snapshot`) — wrap insta integration — Medium effort
- [ ] **#9 Matrix `each`** (cross-product) — rstest `#[values]` parity — Medium effort (parser + codegen)
- [ ] **#10 Type/field matchers** — googletest `field!`, `property!` — Medium effort

### P2 — Future exploration

- [ ] **#11 CLI retry on failure**
- [ ] **#12 CLI watch mode**
- [ ] **#13 Reusable fixture functions**
- [ ] **#14 Suite-level shared setup**
- [ ] **#15 HTML report output**

---

## Part 5: Suggested Release Plan

| Version | Theme | Features |
|---------|-------|----------|
| **v0.3.0** | "Better failures" | P0 #2 colored diffs, P0 #3 regex matchers |
| **v0.4.0** | "Composition" | P0 #4 `all!/any!` combinators, P1 #7 HashMap matchers |
| **v0.5.0** | "Resilience" | P0 #5 soft assertions, P1 #6 test timeout |
| **v0.6.0** | "Power user" | P1 #9 matrix each, P1 #8 snapshot integration |

---

## Sources

- [lib.rs testing category](https://lib.rs/development-tools/testing)
- [rstest docs](https://docs.rs/rstest) / [rstest GitHub](https://github.com/la10736/rstest)
- [googletest-rust matchers](https://docs.rs/googletest/latest/googletest/matchers/index.html) / [GitHub](https://github.com/google/googletest-rust)
- [pretty_assertions](https://crates.io/crates/pretty_assertions)
- [test-case](https://docs.rs/test-case)
- [insta](https://insta.rs/)
- [cargo-nextest](https://nexte.st/)
- [speculate2](https://lib.rs/crates/speculate2)
- [cucumber](https://lib.rs/crates/cucumber)
