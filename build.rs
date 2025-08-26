//! Build script for Caxton project.
//!
//! This build script ensures that the project is recompiled when migration files
//! are modified, which is necessary for the `SQLx` `migrate!()` macro to detect
//! changes in the migrations directory.

/// Main build script entry point.
///
/// Tells Cargo to rerun the build when files in the migrations directory change,
/// which is required for the `SQLx` migration system to work properly.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
