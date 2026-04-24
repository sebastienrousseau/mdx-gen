//! This build script checks if the current Rustc version is at least the
//! minimum required version.
//! If the current Rustc version is less than the minimum required version,
//! the build script will exit the build process with a non-zero exit code.
//!
//! The minimum required version is specified in the `min_version` variable.

use std::process;

fn main() {
    let min_version = "1.88";

    match version_check::is_min_version(min_version) {
        Some(true) => {}
        _ => {
            eprintln!(
                "'mdx-gen' requires Rustc version >= {}",
                min_version
            );
            process::exit(1);
        }
    }
}
