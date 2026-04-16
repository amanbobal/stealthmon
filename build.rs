fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/tray_icon.ico");
        // Set subsystem to windows so no console window appears
        res.set("InternalName", "stealthmon");
        res.set("ProductName", "StealthMon Activity Monitor");
        res.set("FileDescription", "Silent background activity monitor");
        res.compile().expect("Failed to compile Windows resources");
    }
}
