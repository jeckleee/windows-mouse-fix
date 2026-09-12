use crate::model::{Config, Shortcut, Trigger};
#[cfg(not(windows))]
use eframe::egui;
use std::{path::Path, sync::mpsc, thread::JoinHandle};

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Ready,
    State(Config),
    Saved(Result<(), String>),
    Captured(Trigger),
    Shortcut(Shortcut),
    RecordingCancelled,
    Show,
    Hide,
    TrayError(String),
    Toggle,
    Quit,
    Error(String),
}

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Command {
    Configure(Config),
    Capture(bool),
    Record(bool),
    HideToTray,
    CancelHide,
    Shutdown,
    Disconnect,
}

pub struct Service {
    pub events: mpsc::Receiver<Event>,
    commands: mpsc::Sender<Command>,
    worker: Option<JoinHandle<()>>,
}

impl Service {
    pub fn start(config: Config, cc: &eframe::CreationContext<'_>) -> Self {
        let (commands, receiver) = mpsc::channel();
        let (sender, events) = mpsc::channel();
        #[cfg(windows)]
        let worker = {
            let _ = config; // The backend owns the authoritative configuration.
            let window = native::main_window(cc);
            std::thread::spawn(move || settings_connection(receiver, sender, window))
        };
        #[cfg(not(windows))]
        let worker = {
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || native::run(config, ctx, receiver, sender))
        };
        Self {
            events,
            commands,
            worker: Some(worker),
        }
    }
    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.send(Command::Disconnect);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub use native::replace_file;
#[cfg(windows)]
pub use native::{run_backend, single_instance};

#[cfg(windows)]
fn settings_connection(
    commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<Event>,
    window: isize,
) {
    let reader_events = events.clone();
    std::thread::spawn(move || {
        let mut reader = std::io::stdin().lock();
        while let Ok(Some(event)) = crate::ipc::receive::<Event>(&mut reader) {
            let _ = reader_events.send(event);
            native::wake_window(window);
        }
        // The owner exited or the pipe failed. Never keep a nonfunctional editor alive.
        let _ = reader_events.send(Event::Quit);
        native::wake_window(window);
    });
    let mut writer = std::io::stdout().lock();
    while let Ok(command) = commands.recv() {
        let disconnect = matches!(command, Command::Disconnect);
        if crate::ipc::send(&mut writer, &command).is_err() {
            let _ = events.send(Event::Quit);
            native::wake_window(window);
            break;
        }
        if disconnect {
            break;
        }
    }
}

#[cfg(not(windows))]
mod native {
    use super::*;
    pub fn replace_file(from: &Path, to: &Path) -> Result<(), String> {
        std::fs::rename(from, to).map_err(|e| e.to_string())
    }
    pub fn run(
        _: Config,
        ctx: egui::Context,
        commands: mpsc::Receiver<Command>,
        events: mpsc::Sender<Event>,
    ) {
        let _ = events.send(Event::Error(
            "当前为 macOS 界面预览；全局鼠标映射与托盘仅在 Windows 上运行。".into(),
        ));
        ctx.request_repaint();
        while let Ok(command) = commands.recv() {
            if matches!(command, Command::Shutdown | Command::Disconnect) {
                break;
            }
        }
    }
}

#[cfg(windows)]
#[path = "windows.rs"]
mod native;
