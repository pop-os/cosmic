fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=target/wrapper/libwrapper.a");
    println!("cargo:rustc-link-search=target/wrapper/");
}
