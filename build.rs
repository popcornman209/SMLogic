fn main() {
    // Embed the icon into the .exe on Windows builds
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
