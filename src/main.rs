#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(windows)]
mod backend;
#[cfg(windows)]
mod editor_process;
#[cfg(windows)]
mod engine;
mod fonts;
#[cfg(windows)]
mod ipc;
mod model;
mod platform;
#[cfg(windows)]
mod startup;
mod ui;
mod updates;

fn main() -> eframe::Result {
    #[cfg(windows)]
    if !std::env::args_os().any(|arg| arg == "--settings") {
        if let Some(_instance) = platform::single_instance() {
            backend::run(std::env::args_os().any(|arg| arg == "--startup"));
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/mouse.png"))
                    .expect("embedded application icon must be valid"),
            )
            .with_inner_size([780.0, 680.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        &format!("Windows Mouse Fix {}", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc)))),
    )
}
