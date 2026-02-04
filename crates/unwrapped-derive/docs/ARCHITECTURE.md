# Architecture

## Overview

The `unwrapped-derive` crate is the procedural macro entry point. It registers `#[derive(Unwrapped)]` and `#[derive(Wrapped)]`, parses input with `syn`, and delegates the actual code generation to `unwrapped-core`.
