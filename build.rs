use std::io;

fn main() -> io::Result<()> {
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
