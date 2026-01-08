# Architecture

## Overview

The `unwrapped` crate is the public-facing facade for the unwrapped macro ecosystem. It provides the derive macro, error types, and trait definitions that users interact with directly.

## Design

This crate follows the facade pattern, re-exporting functionality from internal crates while providing a clean public API. Users only need to depend on this single crate to use `#[derive(Unwrapped)]`.

### Key Components

- **`UnwrappedError`** - Error type returned by `try_from()` when an `Option` field is `None`. Contains the field name for debugging.

- **`Unwrapped` trait** - Marker trait that associates an original struct with its unwrapped variant via `type Unwrapped`.

- **Derive macro** (feature-gated) - Re-exported from `unwrapped-derive` when the `derive` feature is enabled (on by default).
