Here is the TigerStyle guide, adapted for the Rust ecosystem.

Adapted from https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md (circa 05/12/2025)

***

# TigerStyle (Rust Edition)

## The Essence Of Style
“There are three things extremely hard: steel, a diamond, and to know one's self.” — Benjamin Franklin

Our coding style is evolving. A collective give-and-take at the intersection of engineering and art. Numbers and human intuition. Reason and experience. First principles and knowledge. Precision and poetry. Just like music. A tight beat. A rare groove. Words that rhyme and rhymes that break. Biodigital jazz. This is what we've learned along the way. The best is yet to come.

## Why Have Style?
Another word for style is design.

“The design is not just what it looks like and feels like. The design is how it works.” — Steve Jobs

Our design goals are safety, performance, and developer experience. In that order. All three are important. Good style advances these goals. Does the code make for more or less safety, performance or developer experience? That is why we need style.

Put this way, style is more than readability, and readability is table stakes, a means to an end rather than an end in itself.

“...in programming, style is not something to pursue directly. Style is necessary only where understanding is missing.” ─ Let Over Lambda

This document explores how we apply these design goals to coding style in Rust.

## On Simplicity And Elegance
Simplicity is not a free pass. It's not in conflict with our design goals. It need not be a concession or a compromise.

Rather, simplicity is how we bring our design goals together, how we identify the “super idea” that solves the axes simultaneously, to achieve something elegant.

“Simplicity and elegance are unpopular because they require hard work and discipline to achieve” — Edsger Dijkstra

Contrary to popular belief, simplicity is also not the first attempt but the hardest revision. It's easy to say “let's do something simple”, but to do that in practice takes thought, multiple passes, many sketches, and still we may have to “throw one away”.

The hardest part, then, is how much thought goes into everything. We spend this mental energy upfront, proactively rather than reactively.

An hour or day of design is worth weeks or months in production:

“the simple and elegant systems tend to be easier and faster to design and get right, more efficient in execution, and much more reliable” — Edsger Dijkstra

## Technical Debt
What could go wrong? What's wrong? Which question would we rather ask? The former, because code, like steel, is less expensive to change while it's hot. A problem solved in production is many times more expensive than a problem solved in implementation, or a problem solved in design.

Since it's hard enough to discover showstoppers, when we do find them, we solve them. We don't allow potential clone-latency spikes, or exponential complexity algorithms to slip through.

“You shall not pass!” — Gandalf

We have a “zero technical debt” policy. We do it right the first time. This is important because the second time may not transpire, and because doing good work, that we can be proud of, builds momentum.

## Safety
“The rules act like the seat-belt in your car: initially they are perhaps a little uncomfortable, but after a while their use becomes second-nature and not using them becomes unimaginable.” — Gerard J. Holzmann

NASA's Power of Ten — Rules for Developing Safety Critical Code will change the way you code forever. To expand:

**Use only very simple, explicit control flow.** Do not use recursion to ensure that all executions that should be bounded are bounded (and to avoid stack overflows). Use only a minimum of excellent abstractions but only if they make the best sense of the domain. Abstractions are never zero cost. Every abstraction introduces the risk of a leaky abstraction.

**Put a limit on everything.** All loops and all queues must have a fixed upper bound to prevent infinite loops or tail latency spikes. This follows the “fail-fast” principle. Where a loop cannot terminate (e.g. an event loop), this must be asserted.

**Use explicitly-sized types like `u32` for data.** While Rust encourages `usize` for indexing, use fixed-size types ( `u32`, `u64`) for data structures on disk or over the wire to ensure portability and predictable memory layout. Assert conversions between `usize` and fixed types.

**Assertions detect programmer errors.** Operating errors (File Not Found) return `Result`. Programmer errors (Index Out of Bounds) must panic. The only correct way to handle corrupt code is to crash. Assertions downgrade catastrophic correctness bugs into liveness bugs. Assertions are a force multiplier for discovering bugs by fuzzing.

**Assert all function arguments and invariants.** A function must not operate blindly on data it has not checked. The assertion density of the code must average a minimum of two assertions per function. Use `debug_assert!` for hot paths, but prefer `assert!` by default.

**Pair assertions.** For every property you want to enforce, try to find at least two different code paths where an assertion can be added. Assert validity of data right before writing it to disk, and also immediately after reading from disk.

**Split compound assertions.** Prefer `assert!(a); assert!(b);` over `assert!(a && b);`. The former is simpler to read and provides more precise panic messages.

**Use single-line `if` to assert an implication:** `if a { assert!(b); }`.

**Assert compile-time constants.** Use `const` assertions (or crates like `static_assertions`) to enforce struct sizes, alignments, and invariant relationships before the program even runs.

**The golden rule of assertions** is to assert the positive space that you do expect AND to assert the negative space that you do not expect.

Assertions are a safety net, not a substitute for human understanding. A fuzzer can prove only the presence of bugs, not their absence. Therefore:
1. Build a precise mental model of the code first.
2. Encode your understanding in the form of assertions.
3. Write the code and comments to explain and justify the mental model to your reviewer.

**Minimize dynamic allocation.** Ideally, all memory is allocated at startup. Use `Vec::with_capacity` or pre-allocated arenas. Avoid hidden allocations in hot loops (e.g., `to_string()`, `collect()` or `clone()` on large structures). This avoids unpredictable behavior that can significantly affect performance.

**Declare variables at the smallest possible scope.** Shadowing is permitted in Rust, but use it judiciously.

**Limit function length.** We enforce a hard limit of 70 lines per function. Art is born of constraints.
*   Good function shape is often the inverse of an hourglass: a few parameters, a simple return type, and a lot of meaty logic between the braces.
*   **Centralize control flow.** Push `if`s up and `for`s down.
*   **Centralize state manipulation.** Keep leaf functions pure.

**Respect the compiler.** Treat warnings as errors (`#![deny(warnings)]`). Listen to `clippy`; it is often right.

**External interactions.** When interacting with external entities, do not run in reaction to external events. Batch work. This keeps control flow under your control and improves performance.

**Logic flow:**
*   **Simplify conditions.** Split compound conditions into nested `if`/`else` branches.
*   **State invariants positively.** Negations are mental hurdles.
    *   *Good:* `if index < length { ... } else { ... }`
    *   *Bad:* `if index >= length { ... }`

**Handle all errors.** `unwrap()` is forbidden in production code. Use `expect()` only when you have asserted the condition immediately prior or when the invariant is effectively impossible to violate (and document why). Propagate errors with `?` and handle them at the architectural boundary.

**Always motivate.** Never forget to say *why*. Explain the rationale for a decision in comments.

**Explicit Options.** Avoid relying on implicit defaults for complex behaviors. Use the Builder pattern or Struct Update Syntax with explicit overrides for critical fields.
*   *Good:* `PrefetchOptions { cache: Cache::Data, ..Default::default() }`
*   *Better:* Explicitly passing a configuration struct where defaults might be dangerous.

## Performance
“The lack of back-of-the-envelope performance sketches is the root of all evil.” — Rivacindela Hudsoni

**Think about performance from the outset.** The best time to get 1000x wins is in the design phase. You have to have mechanical sympathy. Work with the grain.

**Perform back-of-the-envelope sketches.** Calculate bandwidth and latency for network, disk, memory, and CPU. Be “roughly right” and land within 90% of the global maximum.

**Optimize for the slowest resources first.** (Network > Disk > Memory > CPU).

**Control Plane vs Data Plane.** Distinguish between them. Batching enables assertion safety in the control plane without killing data plane performance.

**Be predictable.** Don't force the CPU to zig zag. Give the CPU large chunks of work (batching).

**Be explicit.** Extract hot loops into stand-alone functions. This helps the optimizer (LLVM) and helps the human reader spot redundant computations.

## Developer Experience
“There are only two hard things in Computer Science: cache invalidation, naming things, and off-by-one errors.” — Phil Karlton

### Naming Things
Get the nouns and verbs just right.

*   **Casing:** Follow Rust standards. `snake_case` for functions, variables, and modules. `UpperCamelCase` for types and traits. `SCREAMING_SNAKE_CASE` for constants.
*   **No abbreviations:** Use `force`, not `f`. (Exception: `i`, `j` for generic loop counters, `x`, `y` for coordinates).
*   **Acronyms:** `VsrState`, not `VSRState`.
*   **Units last:** `latency_ms_max` rather than `max_latency_ms`. This sorts variables by domain (latency) rather than property (max).
*   **Infuse meaning:** `arena: Allocator` is better than `allocator: Allocator`. It tells the reader about the lifetime strategy.
*   **Symmetry:** Use `source` and `target` (same length) instead of `src` and `dest`. It makes blocks of assignments align visually.
*   **Call history:** If a function calls a helper/callback, prefix the helper with the caller's name. `read_sector()` calls `read_sector_callback()`.
*   **Order matters:** Public fields/methods first. Order structs: Fields -> `impl Type` -> `impl Trait`. Put `new()`/`init()` at the top of the `impl`.

### Explicit Types
Use the "NewType" pattern (tuple structs) to enforce safety.
*   *Bad:* `fn transfer(amount: u64, from: u64, to: u64)`
*   *Good:* `fn transfer(amount: Amount, from: AccountId, to: AccountId)`

### Comments and Commits
**Write descriptive commit messages.** They are being read.

**Say Why and How.** Code isn't documentation. Comments explain *why* the code exists and *how* the methodology works.

**Formatting:** Comments are sentences. Capitalize the first letter. End with a period.

### Cache Invalidation & State
**Minimize aliasing.** Rust's borrow checker prevents memory aliasing, but *logical* aliasing is still possible. Don't keep indices to a `Vec` that might change.

**Pass large types by reference.** Use `&LargeStruct` to prevent accidental stack copies.

**In-place construction.** Return Value Optimization (RVO) is good, but passing a mutable reference to a buffer (`out: &mut LargeStruct`) is often more explicit and safer against stack overflows in unoptimized builds.

**Scope:** Shrink the scope. Calculate variables close to where they are used. Avoid "Place-of-Check to Place-of-Use" (POCPOU) bugs.

**Simpler Signatures:** Dimensionality is viral.
*   `()` trumps `bool`.
*   `bool` trumps `u64`.
*   `u64` trumps `Option<u64>`.
*   `Option<u64>` trumps `Result<u64, E>`.

**Buffer Bleeds:** Ensure padding bytes are zeroed. This is critical for deterministic hashing and preventing information leaks.

**Resource Grouping:** Use scope blocks `{ ... }` to explicitly limit the lifetime of RAII guards (like `MutexGuard` or `RefCell` borrows).

### Off-By-One Errors
Index, count, and size are distinct concepts.
*   **Index:** 0-based.
*   **Count:** 1-based.
*   **Size:** Count * Unit.

**Division:** Show intent. Use `div_euclid`, `div_ceil`, or `div_floor` (via std or crate) rather than `/`.

## Style By The Numbers
*   **Run `cargo fmt`.**
*   **Line Length:** 100 columns. No exceptions. If it doesn't fit, it's too complex.
*   **Braces:** Rust mandates braces for control flow. This prevents "goto fail" bugs.

## Dependencies
**Minimal Dependencies.** Every crate is a supply chain risk, a compilation time cost, and a maintenance burden.
*   Do we really need a crate for a 5-line function?
*   Does this crate panic? Does it use `unsafe`?
*   Audit dependencies vigorously.

## Tooling
“The right tool for the job is often the tool you are already using” — John Carmack

**Standardize on Rust.** Write scripts in Rust (using `cargo-script` or a workspace binary) instead of Bash or Python. This ensures type safety, cross-platform compatibility, and leverage of the team's existing expertise.

## The Last Stage
At the end of the day, keep trying things out, have fun, and remember—it's called TigerBeetle, not only because it's fast, but because it's small!

“You don’t really suppose, do you, that all your adventures and escapes were managed by mere luck, just for your sole benefit? You are a very fine person, Mr. Baggins, and I am very fond of you; but you are only quite a little fellow in a wide world after all!”

“Thank goodness!” said Bilbo laughing, and handed him the tobacco-jar.
