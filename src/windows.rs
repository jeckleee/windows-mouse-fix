#![allow(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::{
    engine::{Effect, Engine},
    model::{Modifiers, is_modifier},
};
use std::{
    cell::RefCell,
    collections::HashSet,
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    Storage::FileSystem::*,
    System::{LibraryLoader::GetModuleHandleW, Threading::CreateMutexW},
    UI::{Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
};

#[path = "windows_tray.rs"]
mod tray;

const CLASS: &str = "WindowsMouseFix.InputWindow";
const SHOW: u32 = WM_APP + 1;
const TRAY: u32 = WM_APP + 2;
const MARKER: usize = 0x574d4649;
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

pub struct Instance(HANDLE);
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub fn single_instance() -> Option<Instance> {
    unsafe {
        let existing = FindWindowW(wide(CLASS).as_ptr(), null());
        if !existing.is_null() && PostMessageW(existing, SHOW, 0, 0) != 0 {
            return None;
        }
        let mutex = CreateMutexW(null(), 0, wide("Local\\WindowsMouseFix.Instance").as_ptr());
        if mutex.is_null() {
            MessageBoxW(
                null_mut(),
                wide("无法创建单实例锁，请重新启动程序。").as_ptr(),
                wide("Windows Mouse Fix").as_ptr(),
                MB_ICONERROR,
            );
            return None;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            for _ in 0..20 {
                let window = FindWindowW(wide(CLASS).as_ptr(), null());
                if !window.is_null() {
                    PostMessageW(window, SHOW, 0, 0);
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            CloseHandle(mutex);
            return None;
        }
        Some(Instance(mutex))
    }
}

pub fn replace_file(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    let from: Vec<_> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<_> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe {
        MoveFileExW(
            from.as_ptr(),
            to.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    } == 0
    {
        Err(std::io::Error::last_os_error().to_string())
    } else {
        Ok(())
    }
}

pub fn main_window(cc: &eframe::CreationContext<'_>) -> isize {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match cc.window_handle().map(|handle| handle.as_raw()) {
        Ok(RawWindowHandle::Win32(window)) => window.hwnd.get(),
        _ => 0,
    }
}

#[derive(Clone)]
struct UiSender {
    events: mpsc::Sender<Event>,
}

impl UiSender {
    fn notify(&self, event: Event) {
        let _ = self.events.send(event);
    }
}

struct Host {
    engine: Engine,
    ui: UiSender,
    start: Instant,
    pointer: POINT,
    recording: bool,
    pressed: [bool; 256],
    swallowed: HashSet<u16>,
}

impl Host {
    fn notify(&self, event: Event) {
        self.ui.notify(event);
    }
    fn now(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
    fn modifiers(&self) -> Modifiers {
        Modifiers {
            ctrl: self.pressed[0xa2] || self.pressed[0xa3] || self.pressed[0x11],
            shift: self.pressed[0xa0] || self.pressed[0xa1] || self.pressed[0x10],
            alt: self.pressed[0xa4] || self.pressed[0xa5] || self.pressed[0x12],
            win: self.pressed[0x5b] || self.pressed[0x5c],
        }
    }
}

thread_local! { static HOST: RefCell<Option<Host>> = const { RefCell::new(None) }; }

unsafe extern "system" fn mouse_hook(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let input = &*(data as *const MSLLHOOKSTRUCT);
    if input.dwExtraInfo == MARKER {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let blocked = HOST.with(|cell| {
        let mut state = cell.borrow_mut();
        let Some(host) = state.as_mut() else {
            return false;
        };
        let message = message as u32;
        if message == WM_MOUSEMOVE {
            let freeze = host.engine.freeze_pointer();
            host.engine
                .movement(input.pt.x - host.pointer.x, input.pt.y - host.pointer.y);
            if !freeze {
                host.pointer = input.pt;
            }
            return freeze;
        }
        let (button, down) = match message {
            WM_MBUTTONDOWN => (3, true),
            WM_MBUTTONUP => (3, false),
            WM_XBUTTONDOWN | WM_XBUTTONUP => {
                let button = match input.mouseData >> 16 {
                    1 => 4,
                    2 => 5,
                    _ => return false,
                };
                (button, message == WM_XBUTTONDOWN)
            }
            _ => return false,
        };
        if down && !host.engine.freeze_pointer() {
            host.pointer = input.pt;
        }
        host.engine
            .button(button, down, host.modifiers(), host.now())
    });
    if blocked {
        1
    } else {
        CallNextHookEx(null_mut(), code, message, data)
    }
}

unsafe extern "system" fn keyboard_hook(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let input = &*(data as *const KBDLLHOOKSTRUCT);
    if input.dwExtraInfo == MARKER || input.vkCode > 255 {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let blocked = HOST.with(|cell| {
        let mut state = cell.borrow_mut();
        let Some(host) = state.as_mut() else {
            return false;
        };
        let key = input.vkCode as u16;
        let down = matches!(message as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
        let was_down = host.pressed[key as usize];
        host.pressed[key as usize] = down;
        if !down && host.swallowed.remove(&key) {
            return true;
        }
        if down && host.swallowed.contains(&key) {
            return true;
        }
        if !host.recording {
            return false;
        }
        if down {
            // Never swallow an up whose physical down preceded recording.
            if !was_down || host.swallowed.contains(&key) {
                host.swallowed.insert(key);
            }
            if !is_modifier(key) && !was_down {
                host.recording = false;
                if key == 27 {
                    host.notify(Event::RecordingCancelled);
                } else {
                    host.notify(Event::Shortcut(Shortcut {
                        key,
                        modifiers: host.modifiers(),
                    }));
                }
            }
            return host.swallowed.contains(&key);
        }
        false
    });
    if blocked {
        1
    } else {
        CallNextHookEx(null_mut(), code, message, data)
    }
}

fn keyboard_input(key: u16, up: bool) -> INPUT {
    let extended =
        matches!(key, 0x21..=0x28 | 0x2d..=0x2e | 0x5b..=0x5c | 0x6f | 0xa3 | 0xa5 | 0xa6..=0xb7);
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: (if up { KEYEVENTF_KEYUP } else { 0 })
                    | (if extended { KEYEVENTF_EXTENDEDKEY } else { 0 }),
                time: 0,
                dwExtraInfo: MARKER,
            },
        },
    }
}

unsafe fn with_modifiers(modifiers: Modifiers, body: Vec<INPUT>) -> Result<(), String> {
    // Release only physically held modifiers, send the requested chord, then restore them.
    let physical: Vec<u16> = [0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0x5b, 0x5c]
        .into_iter()
        .filter(|&key| GetAsyncKeyState(key as i32) < 0)
        .collect();
    let desired: Vec<u16> = [
        (modifiers.ctrl, 0xa2),
        (modifiers.shift, 0xa0),
        (modifiers.alt, 0xa4),
        (modifiers.win, 0x5b),
    ]
    .into_iter()
    .filter_map(|(set, key)| set.then_some(key))
    .collect();
    let body_keys: Vec<_> = body
        .iter()
        .filter(|i| i.r#type == INPUT_KEYBOARD)
        .map(|i| i.Anonymous.ki.wVk)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let held_body_keys: Vec<_> = body_keys
        .iter()
        .copied()
        .filter(|&k| GetAsyncKeyState(k as i32) < 0)
        .collect();
    let mut inputs: Vec<_> = physical
        .iter()
        .rev()
        .map(|&k| keyboard_input(k, true))
        .collect();
    inputs.extend(held_body_keys.iter().map(|&k| keyboard_input(k, true)));
    inputs.extend(desired.iter().map(|&k| keyboard_input(k, false)));
    inputs.extend(body);
    inputs.extend(desired.iter().rev().map(|&k| keyboard_input(k, true)));
    inputs.extend(physical.iter().map(|&k| keyboard_input(k, false)));
    inputs.extend(held_body_keys.iter().map(|&k| keyboard_input(k, false)));
    let sent = SendInput(
        inputs.len() as u32,
        inputs.as_ptr(),
        size_of::<INPUT>() as i32,
    );
    if sent as usize != inputs.len() {
        // A partially inserted sequence must not leave one of our keys held down.
        if sent > 0 {
            let mut cleanup: Vec<_> = desired.iter().map(|&k| keyboard_input(k, true)).collect();
            cleanup.extend(body_keys.iter().map(|&k| keyboard_input(k, true)));
            cleanup.extend(physical.iter().map(|&k| keyboard_input(k, false)));
            cleanup.extend(held_body_keys.iter().map(|&k| keyboard_input(k, false)));
            SendInput(
                cleanup.len() as u32,
                cleanup.as_ptr(),
                size_of::<INPUT>() as i32,
            );
        }
        return Err(
            "快捷键发送失败。目标窗口可能以管理员权限运行；也请检查是否有其他输入拦截软件。".into(),
        );
    }
    Ok(())
}

unsafe fn output(effect: Effect) -> Result<(), String> {
    match effect {
        Effect::Action(action) => {
            if let Some(s) = action.shortcut() {
                with_modifiers(
                    s.modifiers,
                    vec![keyboard_input(s.key, false), keyboard_input(s.key, true)],
                )?;
            }
        }
        Effect::Replay {
            button,
            modifiers,
            count,
        } => {
            let mut inputs = vec![];
            for _ in 0..count {
                for up in [false, true] {
                    let flag = match (button, up) {
                        (3, false) => MOUSEEVENTF_MIDDLEDOWN,
                        (3, true) => MOUSEEVENTF_MIDDLEUP,
                        (_, false) => MOUSEEVENTF_XDOWN,
                        (_, true) => MOUSEEVENTF_XUP,
                    };
                    inputs.push(INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: if button == 3 { 0 } else { (button - 3) as u32 },
                                dwFlags: flag,
                                time: 0,
                                dwExtraInfo: MARKER,
                            },
                        },
                    });
                }
            }
            with_modifiers(modifiers, inputs)?;
        }
        Effect::Captured(trigger) => HOST.with(|s| {
            if let Some(h) = s.borrow().as_ref() {
                h.notify(Event::Captured(trigger));
            }
        }),
    }
    Ok(())
}

pub fn wake_window(window: isize) {
    unsafe {
        PostMessageW(window as HWND, WM_PAINT, 0, 0);
    }
}

pub fn run_backend(config: Config, commands: mpsc::Receiver<Command>, events: mpsc::Sender<Event>) {
    unsafe {
        run_inner(config, commands, UiSender { events });
    }
}

unsafe fn run_inner(config: Config, commands: mpsc::Receiver<Command>, ui: UiSender) {
    let module = GetModuleHandleW(null());
    let (tray_commands, tray_receiver) = mpsc::channel();
    let tray_ui = ui.clone();
    let (show, enabled) = (config.show_tray, config.enabled);
    let tray_worker = std::thread::spawn(move || tray::run(show, enabled, tray_ui, tray_receiver));
    let mut point = zeroed();
    GetCursorPos(&mut point);
    let mut pressed = [false; 256];
    for key in [0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0x5b, 0x5c] {
        pressed[key] = GetAsyncKeyState(key as i32) < 0;
    }
    HOST.with(|s| {
        *s.borrow_mut() = Some(Host {
            engine: Engine::new(config, GetDoubleClickTime() as u64),
            ui,
            start: Instant::now(),
            pointer: point,
            recording: false,
            pressed,
            swallowed: HashSet::new(),
        })
    });
    let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), module, 0);
    let keyboard = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), module, 0);
    let ready = !mouse.is_null() && !keyboard.is_null();
    HOST.with(|s| {
        let mut state = s.borrow_mut();
        let h = state.as_mut().unwrap();
        if ready {
            h.notify(Event::Ready);
        } else {
            h.notify(Event::Error(format!(
                "Windows 输入引擎启动失败：{}",
                std::io::Error::last_os_error()
            )));
        }
    });
    if !ready {
        if !mouse.is_null() {
            UnhookWindowsHookEx(mouse);
        }
        if !keyboard.is_null() {
            UnhookWindowsHookEx(keyboard);
        }
        // Keep the hidden window alive so reopening the application still works.
    }
    let mut running = true;
    while running {
        while let Ok(command) = commands.try_recv() {
            HOST.with(|s| {
                let mut state = s.borrow_mut();
                let h = state.as_mut().unwrap();
                match command {
                    Command::Configure(config) => {
                        let _ = tray_commands.send(tray::TrayCommand::Configure {
                            show: config.show_tray,
                            enabled: config.enabled,
                        });
                        h.engine.configure(config);
                    }
                    Command::Capture(capture) => {
                        h.engine.capture = capture && !h.recording && ready
                    }
                    Command::Record(record) => {
                        h.recording = record && ready;
                        h.engine.cancel();
                    }
                    Command::HideToTray => {
                        h.engine.config.show_tray = true;
                        let _ = tray_commands.send(tray::TrayCommand::HideToTray);
                    }
                    Command::Disconnect => {
                        h.recording = false;
                        h.engine.cancel();
                    }
                    Command::CancelHide => {}
                    Command::Shutdown => running = false,
                }
            });
        }
        let mut message: MSG = zeroed();
        for _ in 0..256 {
            if PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) == 0 {
                break;
            }
            if message.message == WM_QUIT {
                running = false;
                break;
            }
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        let effects = HOST.with(|s| {
            let mut state = s.borrow_mut();
            let h = state.as_mut().unwrap();
            h.engine.tick(h.now());
            h.engine.drain()
        });
        // SendInput can re-enter hooks: never hold the thread-local borrow here.
        for effect in effects {
            if let Err(error) = output(effect) {
                HOST.with(|s| s.borrow().as_ref().unwrap().notify(Event::Error(error)));
            }
        }
        let active = HOST.with(|s| s.borrow().as_ref().unwrap().engine.has_active_input());
        MsgWaitForMultipleObjectsEx(
            0,
            null(),
            if active { 8 } else { 50 },
            QS_ALLINPUT,
            MWMO_INPUTAVAILABLE,
        );
    }
    if ready {
        UnhookWindowsHookEx(mouse);
        UnhookWindowsHookEx(keyboard);
    }
    HOST.with(|s| *s.borrow_mut() = None);
    let _ = tray_commands.send(tray::TrayCommand::Shutdown);
    let _ = tray_worker.join();
}
