fn main() {
    println!("cargo:rerun-if-changed=assets/mouse.ico");
    println!("cargo:rerun-if-changed=assets/windows-mouse-fix.manifest");
    println!("cargo:rerun-if-env-changed=RC_PATH");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/mouse.ico")
            .set_manifest_file("assets/windows-mouse-fix.manifest")
            .set("ProductName", "Windows Mouse Fix")
            .set("FileDescription", "Windows Mouse Fix")
            .compile()
            .expect("failed to compile Windows icon resource");
    }
}
