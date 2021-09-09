fn main() {

    std::env::set_var("RUST_FLAGS", "-C link-args=-Wl,-rpath,.");
    println!("cargo:rustc-link-search=all=.");
    println!("cargo:rustc-link-lib=dylib=SDL2");
}