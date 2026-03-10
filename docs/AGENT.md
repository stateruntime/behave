# Code Style Guide

Canonical style reference for this project. Both human contributors and AI agents must follow these conventions.

---

## 1. Philosophy: Boring Rust Wins

Maintainability is about the humans who will read and change the code, not about the computer that runs it. Write for the next developer, who may be yourself in six months.

**Core principles:**

- Optimize for **reading**. Code is read 10x more than written.
- Prefer **explicit data flow** (inputs → outputs) over magic.
- Keep logic **flat and linear**: guard clauses + small functions.
- Prefer **duplication over the wrong abstraction**.
- Every `pub` item is a promise. Think before making something public.
- If a reviewer asks "what does this do?", the code is too clever — rewrite it.

**What boring means in practice:**

| Avoid | Prefer |
|-------|--------|
| Excessive generics / trait gymnastics | Concrete types; generics only with 3+ concrete uses |
| Macro-heavy APIs | Regular functions and types |
| Operator overloading for non-math types | Named methods: `config.merge(other)` |
| Deep trait hierarchies | Flat traits, composition |
| `From`/`Into` proliferation | Explicit conversion methods |
| Builder with 30 methods | Config struct with `Default` + struct update syntax |
| Stringly-typed APIs | Enums and newtypes |

---

## 2. Project Structure

### Crate Layout

```
crate-name/
  VERSION                 # Single source of truth for crate version
  Cargo.toml              # Manifest (version synced from VERSION)
  deny.toml               # Dependency audit config
  justfile                # Task runner
  rustfmt.toml            # Formatter config
  rust-toolchain.toml     # Toolchain pinning
  src/
    lib.rs                # Crate root: top-level docs, re-exports
    error.rs              # Crate error types (always present)
    <module>.rs           # One module per logical domain
    <module>/
      mod.rs              # For multi-file modules
      submodule.rs
  tests/
    integration_test.rs   # Integration tests (public API only)
  examples/
    basic.rs              # Usage examples
  docs/
    AGENT.md              # This file
    ARCHITECTURE.md       # Architecture overview
    RELEASE.md            # Release process
```

### Module Organization

- **`lib.rs`** is the public API facade. It declares modules and re-exports public types. Keep it short.
- **One module per logical concern.** If you can't describe a module in one sentence without "and", split it.
- **`pub` items** are a long-term commitment. Think before making something public.
- **`pub(crate)`** for internal helpers shared across modules.
- Keep private implementation details private (no visibility modifier).

```rust
// lib.rs — the public face
mod parser;
mod config;
mod error;

pub use self::config::Config;
pub use self::error::Error;
pub use self::parser::parse;
```

### Dependency Direction

Dependencies point inward. Core logic depends on nothing external. Adapters depend on core.

```
  Adapters / Integration   → depends on ↓
  Application Logic        → depends on ↓
  Core Types + Traits      → depends on nothing
```

High-level policy must never depend on low-level detail. Both depend on abstractions:

```rust
// Core defines the interface (no external deps)
pub trait Storage {
    fn get(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    fn put(&self, key: &str, value: &[u8]) -> Result<(), StorageError>;
}

// Adapter implements it (depends on core + external crate)
pub struct FileStorage { root: PathBuf }
impl Storage for FileStorage { /* ... */ }
```

### Cohesion Over Convenience

Every module must have **functional cohesion** — all items contribute to a single, well-defined task.

```rust
// BAD: Coincidental cohesion — a junk drawer
mod utils {
    pub fn format_date() { ... }
    pub fn connect_to_db() { ... }
    pub fn send_email() { ... }
    pub fn calculate_tax() { ... }
}

// GOOD: Functional cohesion
mod tax {
    pub fn calculate_vat(amount: Decimal, rate: Rate) -> Decimal { ... }
    pub fn calculate_import_duty(value: Decimal, country: Country) -> Decimal { ... }
}
```

---

## 3. Formatting

- **Formatter:** `rustfmt` with config in `rustfmt.toml`. Non-negotiable.
- **Line width:** 100 characters max.
- **Indentation:** 4 spaces, no tabs.
- **Imports:** Group in this order, separated by blank lines:
  1. `std` / `core` / `alloc`
  2. External crates
  3. Crate-internal (`crate::`, `super::`)
- **Trailing commas:** Always (rustfmt default).

---

## 4. Naming

Poor naming is the #1 source of cognitive overhead. Good naming is not cosmetic — it is design.

### Conventions Table

| Item              | Convention          | Example                |
|-------------------|---------------------|------------------------|
| Crates            | `kebab-case`        | `my-crate`             |
| Modules           | `snake_case`        | `my_module`            |
| Types / Traits    | `PascalCase`        | `MyStruct`, `Readable` |
| Functions         | `snake_case`        | `do_thing`             |
| Constants         | `SCREAMING_SNAKE`   | `MAX_RETRIES`          |
| Type params       | Single uppercase    | `T`, `E`, `K`, `V`    |
| Lifetimes         | Short lowercase     | `'a`, `'de`            |
| Feature flags     | `snake_case`        | `serde`, `async_runtime` |

### Method Naming

| Pattern | Meaning | Example |
|---------|---------|---------|
| `as_*`  | Cheap borrow → borrow | `as_str()`, `as_bytes()` |
| `to_*`  | Expensive, may allocate | `to_string()`, `to_vec()` |
| `into_*`| Consumes self | `into_inner()`, `into_vec()` |
| `is_*` / `has_*` | Returns bool | `is_empty()`, `has_permission()` |
| `try_*` | Fallible variant | `try_from()`, `try_parse()` |
| `iter` / `iter_mut` / `into_iter` | Iterator producers | per convention |

### Naming Rules

**1. Names reveal intent, not implementation.**
```rust
// Bad — reveals implementation
let elapsed_time_in_ms: u64 = 0;
// Good — reveals intent
let request_timeout_ms: u64 = 0;
```

**2. No meaningless words.**
Banned: `data`, `info`, `temp`, `result`, `item`, `thing`, `obj`, `val`, `flag`, `stuff`.
Banned verbs: `process()`, `handle()`, `manage()`, `execute()`, `do_thing()`.
Prefer: `validate()`, `transform()`, `persist()`, `dispatch()`, `render()`, `parse()`.

**3. No abbreviations** except universally understood: `id`, `url`, `io`, `fmt`, `str`, `msg`, `db`, `tx`.

```rust
// BAD
fn proc_msg(m: &Msg) -> Res { ... }
let cb = |x| x + 1;

// GOOD
fn process_message(message: &Message) -> ProcessResult { ... }
let increment = |value| value + 1;
```

**4. Booleans answer yes/no questions.**
```rust
is_visible, has_children, can_edit, was_deleted, should_retry
```

**5. Function names describe what they DO, not how.**
```rust
// Bad — describes mechanism
fn string_to_config_object(s: &str) -> Config { ... }
// Good — describes purpose
fn parse_config(s: &str) -> Config { ... }
```

**6. Names that lie are worse than bad names.**
A function named `get_*` must not mutate state. A function named `is_*` must not have side effects. A `validate_*` must not transform data.

---

## 5. Functions: Write Dumb Code

### Hard Limits

| Metric | Maximum |
|--------|---------|
| Function body | 40 lines (excluding doc comments, blank lines) |
| Nesting depth | 3 levels |
| Parameters | 4 (use a struct beyond that) |
| Cyclomatic complexity | 10 |

If you need a comment to explain what a block does, extract it into a named function.

### Guard Clauses — No Nesting

Prefer early returns. The happy path flows downward. Never nest deeper than 3 levels.

```rust
// BAD: Deep nesting — reader must mentally track 4+ branches
fn process(data: &[u8], config: &Config) -> Result<Output, Error> {
    if let Some(header) = parse_header(data) {
        if header.version == 2 {
            if let Ok(body) = parse_body(&data[header.len..]) {
                if validate(&body, config) {
                    Ok(transform(body))
                } else { Err(Error::Invalid) }
            } else { Err(Error::ParseBody) }
        } else { Err(Error::UnsupportedVersion(header.version)) }
    } else { Err(Error::NoHeader) }
}

// GOOD: Flat, linear, reads like a checklist
fn process(data: &[u8], config: &Config) -> Result<Output, Error> {
    let header = parse_header(data).ok_or(Error::NoHeader)?;

    if header.version != 2 {
        return Err(Error::UnsupportedVersion(header.version));
    }

    let body = parse_body(&data[header.len..]).map_err(|_| Error::ParseBody)?;

    if !validate(&body, config) {
        return Err(Error::Invalid);
    }

    Ok(transform(body))
}
```

### Linear Flow

Write workflows as a straight line of steps:

```rust
fn execute(input: &Input) -> Result<Output, Error> {
    let parsed = parse(input)?;
    let validated = validate(parsed)?;
    let transformed = transform(validated)?;
    Ok(transformed)
}
```

### Single Level of Abstraction

A function should operate at one level. Don't mix high-level orchestration with low-level detail:

```rust
// BAD: Mixes levels
fn checkout(order: &Order) -> Result<(), Error> {
    // High-level step
    let total = order.items.iter().map(|i| i.price * i.qty).sum::<u64>(); // low-level detail
    // Suddenly SQL
    db.execute(&format!("INSERT INTO orders ..."))?; // very low-level
    send_email(&order.user.email, &format!("Order {} confirmed", order.id))?;
    Ok(())
}

// GOOD: One level
fn checkout(order: &Order) -> Result<(), Error> {
    let total = calculate_total(order);
    save_order(order, total)?;
    notify_customer(order)?;
    Ok(())
}
```

### Command/Query Separation

A function should either **do something** (command) or **answer something** (query) — never both.

```rust
// BAD: Named like a query but mutates state
fn get_user(&mut self, id: UserId) -> &User {
    self.access_count += 1;  // hidden mutation!
    &self.users[&id]
}

// GOOD: Separated
fn user(&self, id: UserId) -> &User { &self.users[&id] }      // query
fn record_access(&mut self, id: UserId) { self.access_count += 1; } // command
```

### No Boolean Flag Parameters

```rust
// BAD: What does `true` mean at the call site?
fn render_button(label: &str, is_primary: bool) { ... }
render_button("Submit", true); // unclear

// GOOD: Enum is self-documenting
enum ButtonStyle { Primary, Secondary }
fn render_button(label: &str, style: ButtonStyle) { ... }
render_button("Submit", ButtonStyle::Primary); // obvious
```

---

## 6. Error Handling

### Library Error Contract

- Implement `Display` and `std::error::Error` manually — no derive macros for errors.
- Define one crate-level `Error` enum (or a small set of domain errors).
- Mark all public error enums `#[non_exhaustive]`.
- Never `unwrap()`, `expect()`, or `panic!()` in library code — propagate with `?`.
- Reserve `unwrap()` for tests only.
- Error messages: lowercase, no trailing punctuation (they chain: "failed to parse: invalid header").

```rust
#[derive(Debug)]
#[non_exhaustive]
pub enum ParseError {
    InvalidHeader { expected: String, found: String },
    UnsupportedVersion(u32),
    Io(std::io::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeader { expected, found } =>
                write!(f, "invalid header: expected {expected}, found {found}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported version: {v}"),
            Self::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self { Self::Io(err) }
}
```

### Error Context

Include enough context to diagnose without a debugger. Include the input that caused the failure:

```rust
#[error("failed to read config file at {}", path.display())]
ReadFile {
    path: PathBuf,
    #[source]
    cause: std::io::Error,
},
```

### When to Use What

| Situation | Return type |
|-----------|-------------|
| Operation can fail in expected ways | `Result<T, Error>` |
| Absence is normal, not an error | `Option<T>` |
| Programmer error / violated invariant | `panic!` (tests only) |
| Never in library code | `unwrap()`, `expect()`, `panic!()` |

### Separate Error Handling from Logic

Don't scatter error handling throughout business logic. Handle errors at boundaries:

```rust
// BAD: Error handling interleaved with logic
fn process(input: &Input) -> Result<Output, Error> {
    let user = match db.get_user(input.user_id) {
        Ok(u) => u,
        Err(e) => { log::error!("db error: {e}"); return Err(Error::Database(e)); }
    };
    // ... 50 more lines of mixed logic and error handling
}

// GOOD: Use ? for propagation, handle at the boundary
fn process(input: &Input) -> Result<Output, Error> {
    let user = db.get_user(input.user_id)?;
    let order = create_order(&user, input)?;
    Ok(order)
}
```

---

## 7. Type Safety

### Parse, Don't Validate

The single most important type safety principle. Represent validated data with distinct types. Validate at the boundary once, use typed representations internally.

```rust
// BAD: Stringly typed, validation scattered and unreliable
fn send_notification(email: &str, message: &str) -> Result<(), Error> {
    if !email.contains('@') { return Err(Error::InvalidEmail); }
    // ...
}

// GOOD: Parse once at boundary, type guarantees validity
pub struct Email(String);

impl Email {
    pub fn parse(input: &str) -> Result<Self, EmailError> {
        if !is_valid_email(input) {
            return Err(EmailError { input: input.to_owned() });
        }
        Ok(Self(input.to_owned()))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

// Cannot receive an invalid email — the type guarantees it
fn send_notification(email: &Email, message: &str) -> Result<(), Error> { ... }
```

### Newtype Wrappers

Wrap primitive types to give them domain meaning and prevent mixups:

```rust
pub struct UserId(u64);
pub struct Meters(f64);
pub struct Seconds(f64);

// These can never be confused, unlike bare u64/f64
pub fn speed(distance: Meters, time: Seconds) -> MetersPerSecond { ... }
```

### Enums Over Booleans

```rust
// BAD — what does `true` mean?
fn connect(addr: &str, use_tls: bool) { ... }

// GOOD — self-documenting
pub enum Transport { Plain, Tls }
fn connect(addr: &str, transport: Transport) { ... }
```

### Make Invalid States Unrepresentable

```rust
// BAD: Booleans + options for mutually exclusive states
pub struct Connection {
    is_authenticated: bool,
    username: Option<String>, // None when not auth'd... hopefully
}

// GOOD: Enum enforces the invariant
pub enum Connection {
    Anonymous,
    Authenticated { username: String },
}
```

### Non-Exhaustive for Future-Proofing

```rust
#[non_exhaustive]
pub enum Status {
    Active,
    Inactive,
    Suspended { reason: String },
    // Can add variants in minor version without breaking downstream
}
```

---

## 8. Abstractions

### The Rule of Three

1. **First time:** Just write it.
2. **Second time:** Duplicate (reluctantly). Note the similarity.
3. **Third time:** Now abstract.

Don't abstract until you have 3 concrete cases. Two looks like a pattern. Three usually is.

### Good Abstractions

A good abstraction:
- Hides complexity behind a simpler interface
- Captures a real concept in the domain
- Has a clear, stable contract
- Does not leak implementation details

```rust
// Good: abstracts the concept (persistence of orders), hides the mechanism
pub trait OrderRepository {
    fn save(&self, order: &Order) -> Result<(), Error>;
    fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, Error>;
}
```

### Bad Abstractions

**Leaky** — exposes internals through the interface:
```rust
// Leaky: caller must know about SQL
pub trait Repository {
    fn find_by_sql(&self, sql: &str) -> Result<Vec<Record>, Error>;
}
```

**Premature** — one implementation behind a trait/factory:
```rust
// Premature: only one shipping calculator exists. Just use a function.
trait ShippingCalculator { fn calculate(&self, ...) -> f64; }
struct StandardShipping;
impl ShippingCalculator for StandardShipping { ... }
struct ShippingCalculatorFactory;
```

**Speculative generality** — code for hypothetical future needs:
```rust
// Nobody asked for plugins. No plugins exist. This is dead weight.
pub trait PluginRegistry { ... }
pub trait Plugin { ... }
pub struct PluginLoader { ... }
```

### Abstract the Concept, Not the Mechanism

```rust
// Abstracting mechanism (bad): caller still deals with low-level detail
pub trait Writer {
    fn write_bytes(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;
}

// Abstracting concept (good): caller works with domain objects
pub trait OrderStore {
    fn save(&self, order: &Order) -> Result<(), StoreError>;
    fn find(&self, id: OrderId) -> Result<Option<Order>, StoreError>;
}
```

---

## 9. State Management

Mutable shared state is the primary source of bugs and unpredictability.

### The Dangerousness Spectrum

```
Pure function (no state)          → most predictable
Local mutable state               → predictable within function
Struct instance state              → predictable within lifetime
Shared module-level state          → dangerous
Global mutable state               → very dangerous
```

### Rules

**1. Prefer immutability.** Use `&self` over `&mut self` where possible.

**2. No global mutable state.** No `static mut`, no `lazy_static` with mutation, no hidden singletons.

**3. Pass state explicitly.** All inputs are parameters, all outputs are return values.

```rust
// BAD: Implicit state — must call methods in order, state bleeds between calls
struct Processor {
    current_user: Option<User>,  // must be set before process()
    last_result: Option<Output>, // carries over between calls!
}

// GOOD: Explicit state — each call is self-contained
fn process(user: &User, input: &Input) -> Result<Output, Error> { ... }
```

**4. No hidden inputs.** Don't read environment variables or global config inside library functions. Take them as parameters.

```rust
// BAD: Hidden input
pub fn save_config(config: &Config) -> Result<(), Error> {
    let path = std::env::var("CONFIG_PATH")?; // hidden dependency!
    std::fs::write(path, config.serialize()?)?;
    Ok(())
}

// GOOD: Explicit input
pub fn save_config(config: &Config, writer: &mut impl Write) -> Result<(), Error> {
    let bytes = config.serialize()?;
    writer.write_all(&bytes)?;
    Ok(())
}
```

**5. No hidden outputs.** If a function has side effects, make it visible in the signature (takes `&mut`, returns evidence of the effect).

---

## 10. Coupling & Cohesion

### Minimize Coupling

**Best:** Message coupling — communicates only through well-defined interfaces (traits).
**Worst:** Content coupling — reaching into another module's private state.

```rust
// BAD: Tight coupling — knows about internal fields
fn process(engine: &mut Engine) {
    engine.internal_buffer.clear(); // reaching into internals
    engine.state = EngineState::Ready; // modifying private state
}

// GOOD: Loose coupling — uses public interface
fn process(engine: &mut Engine) {
    engine.reset(); // delegates to the public API
}
```

### Maximize Cohesion

All items in a module should contribute to a single, well-defined purpose. If a struct has methods that don't use most of its fields, it has low cohesion — split it.

### Code Smells That Signal Coupling/Cohesion Problems

**Shotgun surgery:** One conceptual change touches 10+ files. Extract the concept into its own module/type.

**Feature envy:** A function mostly uses data from another struct. Move it there, or make it a method on that struct.

**Message chains:** `order.customer().address().city().name()` — you know too much about internal structure. Provide `order.shipping_city()` instead.

**Middle man:** A struct that only delegates to another struct with no added value. Delete it.

---

## 11. API Design

### Standard Trait Implementations

Every public type should derive as many standard traits as semantically appropriate:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(u64);
```

Rule: If `Debug` is possible, derive it. If `Clone` is cheap, derive it. If equality is meaningful, derive `PartialEq` (and `Eq` if total).

### Accept Generics, Return Concrete Types

```rust
// Good: accept anything string-like
pub fn connect(addr: impl Into<String>) -> Connection { ... }

// Good: accept anything iterable
pub fn process_all(items: impl IntoIterator<Item = Record>) { ... }

// For read-only strings, prefer &str
pub fn lookup(name: &str) -> Option<&Entry> { ... }
```

### Accept Slices Over Owned Collections

```rust
// BAD
pub fn sum(values: &Vec<f64>) -> f64 { ... }

// GOOD
pub fn sum(values: &[f64]) -> f64 { ... }
```

### Use `#[must_use]` on Pure Functions

```rust
#[must_use]
pub fn compute(input: &Data) -> Result<Output, Error> { ... }
```

### Backwards Compatibility

- `#[non_exhaustive]` on public enums and structs that may grow.
- Sealed traits for traits users should not implement.
- Deprecate for at least one minor version before removing.
- `cargo semver-checks` catches accidental breaks.

---

## 12. Documentation

### Requirements

Every `pub` item gets a `///` doc comment. Enforced by `missing_docs` lint.

1. **Summary line** — imperative mood ("Returns", "Creates", not "This function returns").
2. **`# Examples`** — working doc-test for every public function.
3. **`# Errors`** — list error variants and when they occur.
4. **`# Panics`** — if it can panic, document when (should be extremely rare).

```rust
/// Parses the input string into a structured document.
///
/// # Examples
///
/// ```
/// # use crate_name::parse;
/// let doc = parse("hello world")?;
/// assert_eq!(doc.word_count(), 2);
/// # Ok::<(), crate_name::Error>(())
/// ```
///
/// # Errors
///
/// Returns [`ParseError::EmptyInput`] if the input string is empty.
pub fn parse(input: &str) -> Result<Document, ParseError> { ... }
```

### Comments: When They Help and When They Lie

**Write comments that explain WHY, not WHAT.**

```rust
// GOOD: Explains a non-obvious decision
// Using BTreeMap instead of HashMap because keys must be iterated
// in sorted order for the wire protocol.
let entries: BTreeMap<Key, Value> = BTreeMap::new();

// BAD: Restates the code
// increment counter by 1
counter += 1;
```

**Never commit commented-out code.** Use version control. If you need it back, git has it.

**Lying comments are worse than no comments.** If the code changes but the comment doesn't, the comment becomes a trap. When you change code, update or delete nearby comments.

### Intra-Doc Links

Always link to other items using backtick-bracket syntax:

```rust
/// Parses using the rules defined in [`Config`].
/// For streaming, see [`StreamParser`].
```

---

## 13. Safety & Security

### Unsafe

`unsafe` is **forbidden** at the crate level. If approved after discussion:
- Isolate behind a safe abstraction
- Every `unsafe` block gets a `// SAFETY:` comment

### Banned in Library Code

Enforced by Clippy denials in `Cargo.toml`:

| Banned | Reason |
|--------|--------|
| `unwrap()` | Use `?` or return a proper error |
| `expect()` | Same |
| `panic!()` | Libraries must not crash the caller |
| `todo!()` | No stubs in production code |
| `unimplemented!()` | Same |
| `dbg!()` | No debug prints in production |
| `println!()` / `eprintln!()` | Libraries must not print to stdout/stderr |

### Input Validation at Boundaries

- Every `pub fn` accepting user data must validate it.
- Set limits: max string lengths, max collection sizes, max recursion depth.
- Reject unexpected input rather than trying to "fix" it.

### Secure Defaults

- Timeouts must have default values, never infinite.
- The `Default` impl must produce a safe configuration.

---

## 14. Performance (Without Premature Optimization)

- Prefer iterators over indexed loops.
- Use `Cow<'_, str>` when a function may or may not need to allocate.
- Accept `&str` over `String` in parameters where possible.
- Return iterators instead of `Vec` when the caller may not need to collect.
- Pre-allocate with `Vec::with_capacity(n)` when size is known.
- Zero-copy parsing: borrow from the input buffer where possible.
- **Profile before optimizing.** Use `cargo bench` with `criterion`.

---

## 15. Testing

### Structure

- **Unit tests** in `#[cfg(test)] mod tests` inside each module.
- **Integration tests** in `tests/` directory (public API only).
- **Doc-tests** in `///` examples (documentation that is always correct).

### What to Test

- Business logic and edge cases
- Every error variant (must have a test that triggers it)
- Public API contracts
- Behavior, not implementation

### What NOT to Test

- Private implementation details (they change)
- Simple getters with no logic
- Framework/language built-ins

### Test Naming

Test names describe the scenario and expected outcome:

```rust
#[test]
fn parse_valid_header_returns_version_and_length() { ... }

#[test]
fn parse_empty_input_returns_none() { ... }

#[test]
fn validate_rejects_expired_token() { ... }
```

### Test Quality Rules

- Test **behavior**, not implementation. If you refactor internals, tests should still pass.
- Use `assert_eq!` / `assert_ne!` over plain `assert!` for better failure messages.
- Each test tests **one thing**. If it fails, you know exactly what broke.
- Tests must be **independent** — no shared mutable state between tests.
- No `#[ignore]` tests in main branch — fix or delete them.
- Arrange-Act-Assert structure:

```rust
#[test]
fn applying_coupon_reduces_total() {
    // Arrange
    let order = Order::new(vec![Item::new(100)]);
    let coupon = Coupon::percent(20);

    // Act
    let total = order.apply_coupon(&coupon).total();

    // Assert
    assert_eq!(total, 80);
}
```

---

## 16. Code Smells — What to Watch For

These are surface indicators of deeper problems. When you spot them, refactor.

### Bloaters

- **Long function** (>40 lines) — extract smaller functions.
- **Long parameter list** (>4 params) — use a struct.
- **Primitive obsession** — use newtypes instead of bare `String`/`u64` for domain concepts.
- **Data clumps** — groups of values that always travel together should be a struct.

### Change Preventers

- **Shotgun surgery** — one change touches many files. Extract the shared concept.
- **Divergent change** — one module changes for many unrelated reasons. Split it.

### Dispensables

- **Dead code** — delete it. Git remembers.
- **Speculative generality** — traits/factories for one implementation. Delete the abstraction.
- **TODO/FIXME graveyard** — fix them or file issues and remove the comments.

### Couplers

- **Feature envy** — a function mostly uses another struct's data. Move it.
- **Message chains** — `a.b().c().d()`. Provide a direct method instead.
- **Inappropriate intimacy** — reaching into another module's `pub(crate)` internals. Use the public API.

---

## 17. Feature Flags (Modular Design)

This crate is designed to be modular. Users should be able to import only what they need.

### Rules

- **Features must be additive.** Enabling a feature must never remove functionality.
- **Default features cover the common case.**
- **Heavy dependencies go behind features.** Core must compile fast.
- **Use `dep:` syntax** (Rust 1.60+) to prevent implicit feature names.
- **Document every feature** in `lib.rs` top-level docs and in `Cargo.toml`.

### Pattern: Conditional Compilation

```rust
#[cfg(feature = "serde")]
mod serde_support;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    pub name: String,
}
```

---

## 18. Git Conventions

### Branch Naming

```
feat/short-description
fix/short-description
docs/short-description
refactor/short-description
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): short summary in imperative mood

Optional longer description explaining WHY, not what.
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

### Pull Requests

- One logical change per PR.
- CHANGELOG.md updated for user-facing changes.
- All CI checks must pass.

---

## 19. Versioning & Releases

### Single Source of Truth

The `VERSION` file at the repository root is the single source of truth. `Cargo.toml` must match it. CI enforces this.

### SemVer Rules

| Change | Bump |
|--------|------|
| Breaking API change | Major |
| New feature, backwards compatible | Minor |
| Bug fix, no API change | Patch |
| MSRV bump | Minor (minimum) |
| New optional feature flag | Minor |

### Changelog Discipline

- Every PR with user-facing changes **must** update `CHANGELOG.md` under `[Unreleased]`.
- At release time, `[Unreleased]` is renamed to the new version with date.
- Categories: Added, Changed, Deprecated, Removed, Fixed, Security.

---

## 20. CI Expectations

Every PR must pass:

1. `cargo fmt --check` — formatting
2. `cargo clippy --all-features --all-targets -- -D warnings` — lints
3. `cargo test --all-features` — tests
4. `cargo test --no-default-features` — minimal feature tests
5. `cargo doc --no-deps --all-features` with `-D warnings` — docs build
6. MSRV check — builds on minimum supported Rust version
7. VERSION sync — VERSION file matches Cargo.toml
8. Changelog check — warning if CHANGELOG.md not updated
