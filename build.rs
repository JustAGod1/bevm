use std::io;

fn main() -> io::Result<()> {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    #[cfg(windows)]
    {
        use winres::WindowsResource;
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("ussr.ico")
            .compile()?;
    }
    Ok(())
}
