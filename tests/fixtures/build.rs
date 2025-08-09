fn main() {
    // Use the wat crate from dev-dependencies to generate WASM files
    println!("cargo:rerun-if-changed=build.rs");
}
