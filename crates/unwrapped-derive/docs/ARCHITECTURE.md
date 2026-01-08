# Architecture

## Overview

The `unwrapped-derive` crate is the procedural macro entry point. It handles macro registration and input parsing, delegating the actual code generation to `unwrapped-core`.

## Design

This crate is intentionally minimal. It exists solely because Rust requires proc-macro crates to be separate from regular library crates. All logic lives in `unwrapped-core` to enable reuse by other macro authors.
