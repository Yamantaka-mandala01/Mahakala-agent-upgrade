#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_language(0x0804)
       .set("FileVersion", "1.0.0.0")
       .set("ProductName", "Mahakala Agent")
       .set("FileDescription", "Mahakala Agent - AI Agent with WebUI")
       .set("LegalCopyright", "Copyright (C) 2026");
    
    if let Err(e) = res.compile() {
        eprintln!("Warning: Failed to compile Windows resources: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {}
