# Architecture

## Overview

The `unwrapped` crate is the public-facing facade for the unwrapped macro ecosystem. It provides the derive macros, error types, and trait definitions that users interact with directly.

## Design

This crate follows the facade pattern, re-exporting functionality from internal crates while providing a clean public API. Users only need to depend on this single crate to use `#[derive(Unwrapped)]` and `#[derive(Wrapped)]`.

### Key Components

- **`UnwrappedError`** - Error type returned by fallible conversions. Contains the field name that failed to unwrap.
- **`Unwrapped` trait** - Associates an original struct with its unwrapped variant via `type Unwrapped`.
- **`Wrapped` trait** - Associates an original struct with its wrapped variant via `type Wrapped`.
- **Derive macros** - `Unwrapped` and `Wrapped` are re-exported from `unwrapped-derive` when the `derive` feature is enabled (on by default).
