// Copyright © 2024 MDX Gen. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
// See LICENSE-APACHE.md and LICENSE-MIT.md in the repository root for full license information.

//! # MDX Gen
//!
//! This file serves as an entry point for running all the MDX Gen examples, demonstrating logging levels, formats, macros, and library functionality.

#![allow(missing_docs)]

mod basic_conversion_examples;
mod error_examples;
mod extensions_examples;
mod lib_examples;
mod markdown_examples;

/// Entry point to run all MDX Gen examples.
fn main() {
    println!("\n🦀 Running MDX Gen Examples 🦀");

    let _ = basic_conversion_examples::main();
    let _ = error_examples::main();
    let _ = extensions_examples::main();
    let _ = lib_examples::main();
    let _ = markdown_examples::main();

    println!("\n🎉 All MDX Gen examples completed successfully!\n");
}
