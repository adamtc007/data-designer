fn main() {
    // No build steps needed for HTTP approach
    println!("cargo:rerun-if-changed=src/");
}