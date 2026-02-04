// build.rs
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    pkg_config::Config::new()
        .probe("libffi")
        .expect("Failed to find libffi via pkg-config");
}
