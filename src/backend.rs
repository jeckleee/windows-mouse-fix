use crate::{
    editor_process::EditorProcess,
    model::Config,
    platform::{Command, Event},
};
use std::{
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};

pub fn run() {
    let (config, error) = match Config::load() {
        Ok(config) => (config, None),
        Err(error) => (
            Config {
                enabled: false,
                ..Default::default()
            },
            Some(Event::Error(format!(
                "配置读取失败，已暂停。原文件未覆盖：{error}"
            ))),
        ),
    };
    let (commands, receiver) = mpsc::channel();
    let (sender, events) = mpsc::channel();
    let initial = config.clone();
    let worker =
        std::thread::spawn(move || crate::platform::run_backend(initial, receiver, sender));
    match std::env::current_exe() {
        Ok(executable) => run_session(&executable, config, error, &commands, events),
        Err(error) => show_error(&format!("无法定位设置程序：{error}")),
    }
    let _ = commands.send(Command::Shutdown);
    let _ = worker.join();
}

fn show_error(error: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW};
    let text: Vec<u16> = error.encode_utf16().chain(Some(0)).collect();
    let title: Vec<u16> = "Windows Mouse Fix".encode_utf16().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_ICONERROR,
        );
    }
}

fn notify(editor: &Option<EditorProcess>, event: Event) {
    if let Some(editor) = editor {
        editor.send(event);
    }
}

fn apply(config: &Config, commands: &mpsc::Sender<Command>, editor: &Option<EditorProcess>) {
    let _ = commands.send(Command::Configure(config.clone()));
    notify(editor, Event::Saved(config.save()));
}

fn restore_tray(
    config: &mut Config,
    commands: &mpsc::Sender<Command>,
    editor: &Option<EditorProcess>,
) {
    let _ = commands.send(Command::Disconnect);
    if !config.show_tray {
        config.show_tray = true;
        apply(config, commands, editor);
        notify(editor, Event::State(config.clone()));
    }
}

fn run_session(
    executable: &Path,
    mut config: Config,
    mut last_error: Option<Event>,
    commands: &mpsc::Sender<Command>,
    events: mpsc::Receiver<Event>,
) {
    let (requests, incoming) = mpsc::channel::<(u32, Command)>();
    let mut editor: Option<EditorProcess> = None;
    let mut ready = false;
    let mut want_open = true;
    let mut hide_pid = None;
    let mut running = true;
    while running {
        // Only the currently owned child can change configuration or stop the backend.
        // Late EOF/commands from an earlier editor must not affect a newly opened one.
        for (pid, command) in incoming.try_iter() {
            if editor.as_ref().is_none_or(|child| child.child.id() != pid) {
                continue;
            }
            match command {
                Command::Configure(next) => {
                    if let Err(error) = next.validate() {
                        notify(&editor, Event::State(config.clone()));
                        notify(&editor, Event::Error(error));
                    } else {
                        config = next;
                        apply(&config, commands, &editor);
                    }
                }
                Command::Capture(value) => {
                    let _ = commands.send(Command::Capture(value));
                }
                Command::Record(value) => {
                    let _ = commands.send(Command::Record(value));
                }
                Command::HideToTray => {
                    hide_pid = Some(pid);
                    restore_tray(&mut config, commands, &editor);
                    let _ = commands.send(Command::HideToTray);
                }
                Command::Disconnect => {
                    editor.as_mut().unwrap().closing = true;
                    editor.as_mut().unwrap().disconnected = true;
                    hide_pid = None;
                    restore_tray(&mut config, commands, &editor);
                }
                Command::CancelHide => {
                    hide_pid = None;
                    editor.as_mut().unwrap().closing = false;
                }
                Command::Shutdown => {
                    running = false;
                    break;
                }
            }
        }
        if !running {
            break;
        }
        if let Some(child) = editor.as_mut()
            && child.disconnected
        {
            match child.child.try_wait() {
                Ok(Some(_)) => {
                    editor = None;
                    hide_pid = None;
                    restore_tray(&mut config, commands, &editor);
                }
                Ok(None) => {}
                Err(error) => {
                    last_error = Some(Event::Error(format!("设置进程状态检查失败：{error}")));
                    editor = None;
                    hide_pid = None;
                    restore_tray(&mut config, commands, &editor);
                }
            }
        }
        // Polling here does not draw anything: the resident process never creates egui/GL.
        match events.recv_timeout(Duration::from_millis(25)) {
            Ok(event) => match event {
                Event::Show => {
                    hide_pid = None;
                    want_open = true;
                }
                Event::Quit => running = false,
                Event::Toggle => {
                    config.enabled = !config.enabled;
                    apply(&config, commands, &editor);
                    notify(&editor, Event::State(config.clone()));
                }
                Event::Ready => {
                    ready = true;
                    notify(&editor, Event::Ready);
                }
                Event::Hide => {
                    if matches!(last_error, Some(Event::TrayError(_))) {
                        last_error = None;
                    }
                    if let Some(pid) = hide_pid.take()
                        && let Some(child) = editor.as_mut()
                        && child.child.id() == pid
                    {
                        child.closing = true;
                        child.send(Event::Hide);
                    }
                }
                Event::TrayError(_) => {
                    hide_pid = None;
                    last_error = Some(event.clone());
                    notify(&editor, event);
                    if editor.as_ref().is_none_or(|child| child.closing) {
                        want_open = true;
                    }
                }
                Event::Error(_) => {
                    last_error = Some(event.clone());
                    notify(&editor, event);
                }
                _ => notify(&editor, event),
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                show_error("后台输入服务已停止，请重新启动程序。");
                break;
            }
        }
        if want_open && running {
            if let Some(child) = &editor {
                if !child.closing {
                    child.send(Event::Show);
                    want_open = false;
                }
            } else {
                match EditorProcess::spawn(executable, requests.clone()) {
                    Ok(child) => {
                        child.send(Event::State(config.clone()));
                        if ready {
                            child.send(Event::Ready);
                        }
                        if let Some(error) = &last_error {
                            child.send(error.clone());
                        }
                        editor = Some(child);
                    }
                    Err(error) => {
                        restore_tray(&mut config, commands, &editor);
                        show_error(&format!("无法打开设置窗口：{error}"));
                    }
                }
                want_open = false;
            }
        }
    }
    let _ = commands.send(Command::Shutdown);
    if let Some(child) = &mut editor {
        child.send(Event::Quit);
        let deadline = Instant::now() + Duration::from_secs(2);
        while matches!(child.child.try_wait(), Ok(None)) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    // EditorProcess::drop reaps the child, forcibly only if graceful shutdown failed.
}
