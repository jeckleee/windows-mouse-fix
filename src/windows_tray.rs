use super::*;
use std::cell::Cell;
use windows_sys::Win32::{Security::*, System::Threading::*};

// Read only after registration fails; never change another process's permissions.
unsafe fn token_value(token: HANDLE, class: TOKEN_INFORMATION_CLASS) -> String {
    let mut value = 0u32;
    let mut length = 0;
    if GetTokenInformation(
        token,
        class,
        (&mut value as *mut u32).cast(),
        4,
        &mut length,
    ) == 0
    {
        return format!("读取失败：{}", std::io::Error::last_os_error());
    }
    value.to_string()
}

unsafe fn integrity_level(token: HANDLE) -> String {
    let mut length = 0;
    GetTokenInformation(token, TokenIntegrityLevel, null_mut(), 0, &mut length);
    if length == 0 {
        return format!("读取失败：{}", std::io::Error::last_os_error());
    }
    // Word-aligned storage for TOKEN_MANDATORY_LABEL and the SID it points into.
    let mut buffer = vec![0usize; (length as usize).div_ceil(size_of::<usize>())];
    if GetTokenInformation(
        token,
        TokenIntegrityLevel,
        buffer.as_mut_ptr().cast(),
        length,
        &mut length,
    ) == 0
    {
        return format!("读取失败：{}", std::io::Error::last_os_error());
    }
    let label = &*buffer.as_ptr().cast::<TOKEN_MANDATORY_LABEL>();
    if IsValidSid(label.Label.Sid) == 0 {
        return "无效完整性 SID".into();
    }
    let count = *GetSidSubAuthorityCount(label.Label.Sid);
    if count == 0 {
        return "完整性 SID 缺少级别".into();
    }
    let rid = *GetSidSubAuthority(label.Label.Sid, u32::from(count - 1));
    let name = match rid {
        0x0000..=0x0fff => "不可信",
        0x1000..=0x1fff => "低",
        0x2000..=0x2fff => "中",
        0x3000..=0x3fff => "高",
        _ => "系统或更高",
    };
    format!("{name}（0x{rid:04X}）")
}

unsafe fn process_security(pid: u32) -> String {
    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if process.is_null() {
        return format!("PID {pid}，读取失败：{}", std::io::Error::last_os_error());
    }
    let mut token = null_mut();
    let result = if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
        format!(
            "PID {pid}，令牌读取失败：{}",
            std::io::Error::last_os_error()
        )
    } else {
        let result = format!(
            "PID {pid}，完整性：{}\n会话：{}，已提升：{}，UIAccess：{}，AppContainer：{}",
            integrity_level(token),
            token_value(token, TokenSessionId),
            token_value(token, TokenElevation),
            token_value(token, TokenUIAccess),
            token_value(token, TokenIsAppContainer),
        );
        CloseHandle(token);
        result
    };
    CloseHandle(process);
    result
}

pub enum TrayCommand {
    Configure { show: bool, enabled: bool },
    HideToTray,
    Shutdown,
}

thread_local! {
    static UI: RefCell<Option<UiSender>> = const { RefCell::new(None) };
    static TASKBAR_CREATED: Cell<u32> = const { Cell::new(0) };
    static REFRESH: Cell<bool> = const { Cell::new(false) };
}

unsafe extern "system" fn window_proc(window: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // Clone before calling Windows: Shell/menu APIs can re-enter this window procedure.
    let ui = UI.with(|s| s.borrow().clone());
    if message == SHOW {
        if let Some(ui) = ui {
            ui.notify(Event::Show);
        }
        return 0;
    }
    if message == TRAY {
        if let Some(ui) = ui {
            match l as u32 & 0xffff {
                WM_LBUTTONDBLCLK => ui.notify(Event::Show),
                WM_CONTEXTMENU | WM_RBUTTONUP => {
                    let menu = CreatePopupMenu();
                    if menu.is_null() {
                        ui.notify(Event::TrayError(
                            "无法打开托盘菜单，请重新打开 exe 进入设置。".into(),
                        ));
                        return 0;
                    }
                    AppendMenuW(menu, MF_STRING, 1, wide("打开窗口").as_ptr());
                    AppendMenuW(menu, MF_STRING, 2, wide("启用 / 暂停").as_ptr());
                    AppendMenuW(menu, MF_SEPARATOR, 0, null());
                    AppendMenuW(menu, MF_STRING, 3, wide("退出").as_ptr());
                    let mut point = zeroed();
                    GetCursorPos(&mut point);
                    SetForegroundWindow(window);
                    let choice = TrackPopupMenu(
                        menu,
                        TPM_RETURNCMD | TPM_RIGHTBUTTON,
                        point.x,
                        point.y,
                        0,
                        window,
                        null(),
                    );
                    DestroyMenu(menu);
                    PostMessageW(window, WM_NULL, 0, 0);
                    match choice {
                        1 => ui.notify(Event::Show),
                        2 => ui.notify(Event::Toggle),
                        3 => ui.notify(Event::Quit),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        return 0;
    }
    if message != 0 && TASKBAR_CREATED.with(|id| id.get() == message) {
        REFRESH.with(|refresh| refresh.set(true));
    }
    DefWindowProcW(window, message, w, l)
}

unsafe fn allow_shell_messages(window: HWND, taskbar_created: u32) -> Result<(), String> {
    // Explorer can have lower integrity than this app when it is run as administrator.
    // Allow tray callbacks, reopen requests and taskbar recreation on our window.
    for message in [TRAY, SHOW, taskbar_created]
        .into_iter()
        .filter(|id| *id != 0)
    {
        if ChangeWindowMessageFilterEx(window, message, MSGFLT_ALLOW, null_mut()) == 0 {
            return Err(format!(
                "无法接收任务栏消息（0x{message:04X}）：{}。托盘操作可能不可用。",
                std::io::Error::last_os_error()
            ));
        }
    }
    Ok(())
}

struct Tray {
    data: NOTIFYICONDATAW,
    present: bool,
    show: bool,
    enabled: bool,
}

impl Tray {
    unsafe fn refresh(&mut self) -> Result<(), String> {
        if !self.show {
            if self.present {
                Shell_NotifyIconW(NIM_DELETE, &self.data);
            }
            self.present = false;
            return Ok(());
        }
        if self.data.hWnd.is_null() || self.data.hIcon.is_null() {
            return Err("托盘窗口或图标初始化失败，设置窗口将保持打开。".into());
        }
        self.data.szTip.fill(0);
        let tip = wide(if self.enabled {
            "Windows Mouse Fix · 已启用"
        } else {
            "Windows Mouse Fix · 已暂停"
        });
        self.data.szTip[..tip.len()].copy_from_slice(&tip);
        // Explorer may have lost an icon, or still have it after a failed previous request.
        let operations = if self.present {
            [NIM_MODIFY, NIM_ADD]
        } else {
            [NIM_ADD, NIM_MODIFY]
        };
        let mut failures = Vec::new();
        let mut access_denied = false;
        for operation in operations {
            // Shell_NotifyIcon only documents its BOOL result. Clear stale thread errors;
            // any last-error value below is supplemental diagnostic information.
            SetLastError(0);
            let started = Instant::now();
            let result = Shell_NotifyIconW(operation, &self.data);
            let error = GetLastError();
            if result != 0 {
                self.present = true;
                self.data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
                Shell_NotifyIconW(NIM_SETVERSION, &self.data);
                return Ok(());
            }
            access_denied |= error == ERROR_ACCESS_DENIED;
            let operation = if operation == NIM_ADD {
                "NIM_ADD"
            } else {
                "NIM_MODIFY"
            };
            failures.push(format!(
                "{operation}: FALSE，附加错误 0x{error:08X}，耗时 {} ms",
                started.elapsed().as_millis()
            ));
        }
        self.present = false;
        let taskbar = FindWindowW(wide("Shell_TrayWnd").as_ptr(), null());
        let mut shell_pid = 0;
        GetWindowThreadProcessId(taskbar, &mut shell_pid);
        let shell_security = if shell_pid == 0 {
            "未找到任务栏所属进程".into()
        } else {
            process_security(shell_pid)
        };
        let executable = std::env::current_exe()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|error| format!("无法读取：{error}"));
        let guidance = if access_denied {
            "附加错误为访问被拒绝，尚不能确定原因。请复制下方权限诊断信息；管理员运行仅是临时绕过办法。"
        } else {
            "请复制以下诊断信息，以便进一步排查。"
        };
        Err(format!(
            "Windows Shell 未能注册托盘图标，设置窗口将保持打开。\n{guidance}\n\n版本：{}\n{}\n托盘消息窗口有效：{}\n系统任务栏窗口存在：{}\n结构体大小：{}\n程序路径：{}\n\n本程序：{}\n任务栏进程：{}\n\n附加错误仅供参考；权限字段 0 表示否、1 表示是，读取失败不代表否。",
            env!("CARGO_PKG_VERSION"),
            failures.join("\n"),
            IsWindow(self.data.hWnd) != 0,
            !taskbar.is_null(),
            self.data.cbSize,
            executable,
            process_security(GetCurrentProcessId()),
            shell_security,
        ))
    }
}

// The shell must not run on the low-level input-hook thread.
pub fn run(show: bool, enabled: bool, ui: UiSender, commands: mpsc::Receiver<TrayCommand>) {
    unsafe {
        run_inner(show, enabled, ui, commands);
    }
}

unsafe fn run_inner(
    show: bool,
    enabled: bool,
    ui: UiSender,
    commands: mpsc::Receiver<TrayCommand>,
) {
    UI.with(|s| *s.borrow_mut() = Some(ui.clone()));
    TASKBAR_CREATED.with(|id| id.set(RegisterWindowMessageW(wide("TaskbarCreated").as_ptr())));
    let module = GetModuleHandleW(null());
    let class_name = wide(CLASS);
    let class = WNDCLASSW {
        style: CS_DBLCLKS,
        lpfnWndProc: Some(window_proc),
        hInstance: module,
        lpszClassName: class_name.as_ptr(),
        ..zeroed()
    };
    let registered = RegisterClassW(&class);
    let window = if registered == 0 {
        null_mut()
    } else {
        CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            class_name.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            module,
            null(),
        )
    };
    let init_error = std::io::Error::last_os_error();
    let mut data: NOTIFYICONDATAW = zeroed();
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = window;
    data.uID = 1;
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    data.uCallbackMessage = TRAY;
    data.hIcon = LoadImageW(
        module,
        std::ptr::without_provenance(1), // MAKEINTRESOURCEW: icon group ID from build.rs.
        IMAGE_ICON,
        GetSystemMetrics(SM_CXSMICON),
        GetSystemMetrics(SM_CYSMICON),
        LR_DEFAULTCOLOR,
    ) as HICON;
    let mut tray = Tray {
        data,
        present: false,
        show,
        enabled,
    };
    if window.is_null() {
        ui.notify(Event::TrayError(format!(
            "无法创建托盘消息窗口：{init_error}"
        )));
    } else {
        if let Err(error) = allow_shell_messages(window, TASKBAR_CREATED.with(|id| id.get())) {
            ui.notify(Event::TrayError(error));
        }
        if let Err(error) = tray.refresh() {
            ui.notify(Event::TrayError(error));
        }
    }
    let mut running = true;
    while running {
        while let Ok(command) = commands.try_recv() {
            let hide = matches!(command, TrayCommand::HideToTray);
            match command {
                TrayCommand::Configure { show, enabled } => {
                    tray.show = show;
                    tray.enabled = enabled;
                }
                TrayCommand::HideToTray => tray.show = true,
                TrayCommand::Shutdown => {
                    running = false;
                    break;
                }
            }
            match tray.refresh() {
                Ok(()) if hide => ui.notify(Event::Hide),
                Ok(()) => {}
                Err(error) => ui.notify(Event::TrayError(error)),
            }
        }
        let mut message = zeroed();
        for _ in 0..64 {
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
        if REFRESH.with(|refresh| refresh.replace(false)) {
            tray.present = false;
            if let Err(error) = tray.refresh() {
                ui.notify(Event::TrayError(error));
            }
        }
        MsgWaitForMultipleObjectsEx(0, null(), 50, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    }
    tray.show = false;
    let _ = tray.refresh();
    if !window.is_null() {
        DestroyWindow(window);
    }
    if !tray.data.hIcon.is_null() {
        DestroyIcon(tray.data.hIcon);
    }
    if registered != 0 {
        UnregisterClassW(class_name.as_ptr(), module);
    }
    UI.with(|s| *s.borrow_mut() = None);
}
