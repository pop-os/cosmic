fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=target/c/libwrapper.a");
    println!("cargo:rustc-link-search=target/c/");
}
