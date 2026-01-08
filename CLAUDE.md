# Project Overview

`unwrapped` is a Rust procedural macro library that generates "unwrapped" struct variants by converting `Option<T>` fields to `T` fields. This is useful for form handling, API design patterns, and data transformation pipelines where you need multiple struct variants with different optionality semantics.

**Key Focus Areas:**

- **Type Safety** - Compile-time generation of unwrapped struct variants with proper trait implementations
- **Flexibility** - Configurable naming, field skipping, and both fallible/infallible conversions
- **Extensibility** - Core logic exposed for proc-macro authors to build upon

## Architecture Documentation Index

| Crate | Architecture Doc | Purpose |
|-------|------------------|---------|
| `unwrapped` | [docs/ARCHITECTURE.md](crates/unwrapped/docs/ARCHITECTURE.md) | Public facade exposing derive macro and error types |
| `unwrapped-derive` | [docs/ARCHITECTURE.md](crates/unwrapped-derive/docs/ARCHITECTURE.md) | Procedural macro entry point |
| `unwrapped-core` | [docs/ARCHITECTURE.md](crates/unwrapped-core/docs/ARCHITECTURE.md) | Reusable core logic for macro authors |

## Crate Descriptions

### Core Library

- **unwrapped** - Public-facing crate providing `#[derive(Unwrapped)]`, the `Unwrapped` trait, and `UnwrappedError` type

### Macro Implementation

- **unwrapped-derive** - Proc-macro crate that parses derive input and delegates to core
- **unwrapped-core** - Standalone core logic with no proc-macro dependencies, reusable by other macro authors
