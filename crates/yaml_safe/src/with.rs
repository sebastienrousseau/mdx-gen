// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Helper modules for `#[serde(with = "...")]` enum
//! serialization strategies.
//!
//! These modules provide alternative ways to represent
//! Rust enums in YAML, matching the `serde_yml::with` API.

pub mod singleton_map;
pub mod singleton_map_optional;
pub mod singleton_map_recursive;
