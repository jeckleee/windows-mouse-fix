use crate::{
    model::*,
    platform::{Command, Event, Service},
    updates,
};
use eframe::egui::{self, Color32, RichText};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

type UpdateResult = Result<Option<(String, String)>, String>;

pub struct App {
    config: Config,
    service: Service,
    tab: u8,
    status: String,
    ready: bool,
    capture: bool,
    recording: bool,
    draft: Option<(Option<usize>, Rule)>,
    draft_error: String,
    restore: bool,
    quitting: bool,
    hide_pending: Option<Instant>,
    tray_error: Option<String>,
    update_rx: Option<mpsc::Receiver<UpdateResult>>,
    update_status: String,
    release: Option<(String, String)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        ctx.set_visuals(egui::Visuals::dark());
        let mut fonts = egui::FontDefinitions::default();
        if let Some(data) = crate::fonts::chinese_font() {
            fonts
                .font_data
                .insert("Chinese".into(), egui::FontData::from_static(data).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("Chinese".into());
            ctx.set_fonts(fonts);
        }
        let mut style = (*ctx.style()).clone();
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));
        style.spacing.item_spacing = egui::vec2(10.0, 12.0);
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        ctx.set_style(style);
        let (config, status) = match Config::load() {
            Ok(config) => (config, "正在启动输入引擎…".into()),
            Err(error) => (
                Config {
                    enabled: false,
                    ..Default::default()
                },
                format!("配置读取失败，已暂停。原文件未覆盖：{error}"),
            ),
        };
        let service = Service::start(config.clone(), cc);
        let mut app = Self {
            config,
            service,
            tab: 0,
            status,
            ready: false,
            capture: false,
            recording: false,
            draft: None,
            draft_error: String::new(),
            restore: false,
            quitting: false,
            hide_pending: None,
            tray_error: None,
            update_rx: None,
            update_status: String::new(),
            release: None,
        };
        if app.config.check_updates && !app.config.update_repository.is_empty() {
            app.check_updates();
        }
        app
    }

    fn save(&mut self) {
        self.service.send(Command::Configure(self.config.clone()));
        self.capture = false;
        #[cfg(windows)]
        {
            self.status = "正在保存…".into();
        }
        #[cfg(not(windows))]
        self.saved(self.config.save());
    }

    fn saved(&mut self, result: Result<(), String>) {
        self.status = match result {
            Ok(()) => if self.ready {
                "已保存并生效"
            } else {
                "已保存；输入引擎尚未就绪"
            }
            .into(),
            Err(error) => format!("当前会话已应用，但保存失败：{error}"),
        };
    }

    fn quit(&mut self, ctx: &egui::Context) {
        self.service.send(Command::Shutdown);
        self.quitting = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn check_updates(&mut self) {
        if self.update_rx.is_some() {
            return;
        }
        let repo = self.config.update_repository.trim().to_owned();
        let beta = self.config.beta_updates;
        let (sender, receiver) = mpsc::channel();
        self.update_rx = Some(receiver);
        self.update_status = "正在检查…".into();
        self.release = None;
        std::thread::spawn(move || {
            let _ = sender.send(updates::check(&repo, beta));
        });
    }

    fn open_editor(&mut self, index: Option<usize>, rule: Rule) {
        self.service.send(Command::Capture(false));
        self.capture = false;
        self.draft_error.clear();
        self.draft = Some((index, rule));
    }

    fn general(&mut self, ui: &mut egui::Ui) {
        ui.heading("通用");
        ui.add_space(8.0);
        let mut changed = ui
            .checkbox(&mut self.config.enabled, "启用按钮映射")
            .changed();
        ui.weak("关闭设置窗口后继续在后台运行；使用下方或托盘中的“退出程序”结束运行。");
        ui.separator();
        changed |= ui
            .checkbox(&mut self.config.show_tray, "在系统托盘中显示")
            .changed();
        ui.weak("双击托盘图标打开窗口；右键可打开窗口、启用 / 暂停或退出。关闭窗口时会自动显示托盘图标。");
        ui.separator();
        ui.label("更新来源（此 Windows 项目的 GitHub 仓库）");
        let edit = ui.add(
            egui::TextEdit::singleline(&mut self.config.update_repository)
                .hint_text("所有者/仓库名，发布前可留空"),
        );
        changed |= edit.changed();
        let configured = !self.config.update_repository.trim().is_empty();
        changed |= ui
            .add_enabled(
                configured,
                egui::Checkbox::new(&mut self.config.check_updates, "打开设置时检查更新"),
            )
            .changed();
        changed |= ui
            .add_enabled(
                configured && self.config.check_updates,
                egui::Checkbox::new(&mut self.config.beta_updates, "包含 Beta 版本"),
            )
            .changed();
        if !configured {
            ui.weak("尚未配置发布源，不会请求原 Mac 项目的更新。");
        }
        if ui
            .add_enabled(
                configured && self.update_rx.is_none(),
                egui::Button::new("立即检查"),
            )
            .clicked()
        {
            self.check_updates();
        }
        if !self.update_status.is_empty() {
            ui.label(&self.update_status);
        }
        if let Some((version, url)) = &self.release {
            ui.hyperlink_to(format!("下载 {version}"), url);
        }
        ui.separator();
        ui.weak(format!("版本 {} · 配置保存在：", env!("CARGO_PKG_VERSION")));
        ui.label(Config::path().display().to_string());
        ui.add_space(8.0);
        if ui.button("退出程序").clicked() {
            self.quit(ui.ctx());
        }
        if changed {
            self.save();
        }
    }

    fn buttons(&mut self, ui: &mut egui::Ui) -> bool {
        ui.heading("按钮");
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 105.0),
            egui::Sense::hover(),
        );
        ui.painter().rect_filled(
            rect,
            8.0,
            if response.hovered() {
                Color32::from_gray(65)
            } else {
                Color32::from_gray(43)
            },
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "+",
            egui::FontId::proportional(52.0),
            Color32::LIGHT_GRAY,
        );
        ui.label("将指针移入 + 区域，操作中键或侧键以录入。支持单击、双击、长按和四向拖动。");
        ui.weak("可同时按住 Ctrl / Shift / Alt / Win。单击录入需等待系统双击时间结束。");
        ui.horizontal(|ui| {
            if ui.button("手动添加映射").clicked() {
                self.open_editor(None, Rule::default());
            }
            if ui.button("恢复默认值…").clicked() {
                self.restore = true;
            }
        });
        if ui
            .checkbox(&mut self.config.lock_pointer, "执行拖动映射时锁定指针")
            .changed()
        {
            self.save();
        }
        ui.separator();
        let mut edit = None;
        let mut remove = None;
        egui::ScrollArea::both().id_salt("rules").show(ui, |ui| {
            if self.config.rules.is_empty() {
                ui.weak("尚未添加映射。默认保留所有鼠标按键的原始行为。");
            }
            for button in 3..=5 {
                if !self.config.rules.iter().any(|r| r.trigger.button == button) {
                    continue;
                }
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.strong(button_label(button));
                    egui::Grid::new(("rules", button))
                        .num_columns(4)
                        .striped(true)
                        .show(ui, |ui| {
                            for (index, rule) in self
                                .config
                                .rules
                                .iter()
                                .enumerate()
                                .filter(|(_, r)| r.trigger.button == button)
                            {
                                if ui.small_button("删除").clicked() {
                                    remove = Some(index);
                                }
                                ui.label(format!(
                                    "{}{}",
                                    rule.trigger.modifiers.label(),
                                    rule.trigger.gesture.label()
                                ));
                                ui.label(rule.action.label());
                                if ui.small_button("编辑").clicked() {
                                    edit = Some(index);
                                }
                                ui.end_row();
                            }
                        });
                });
            }
        });
        if let Some(index) = edit {
            self.open_editor(Some(index), self.config.rules[index].clone());
        }
        if let Some(index) = remove {
            self.config.rules.remove(index);
            self.save();
        }
        response.hovered() && self.ready && self.draft.is_none() && !self.restore
    }

    fn editor(&mut self, ctx: &egui::Context) {
        let Some((index, mut rule)) = self.draft.clone() else {
            return;
        };
        let mut cancel = false;
        let mut commit = false;
        egui::Window::new(if index.is_some() {
            "编辑映射"
        } else {
            "添加映射"
        })
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.add_enabled_ui(!self.recording, |ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("button")
                        .selected_text(button_label(rule.trigger.button))
                        .show_ui(ui, |ui| {
                            for button in 3..=5 {
                                ui.selectable_value(
                                    &mut rule.trigger.button,
                                    button,
                                    button_label(button),
                                );
                            }
                        });
                    egui::ComboBox::from_id_salt("gesture")
                        .selected_text(rule.trigger.gesture.label())
                        .show_ui(ui, |ui| {
                            for gesture in Gesture::ALL {
                                ui.selectable_value(
                                    &mut rule.trigger.gesture,
                                    gesture,
                                    gesture.label(),
                                );
                            }
                        });
                });
                ui.label("触发时需要按住的修饰键");
                modifier_controls(ui, &mut rule.trigger.modifiers);
                ui.separator();
                ui.label("执行动作");
                egui::ComboBox::from_id_salt("action")
                    .width(270.0)
                    .selected_text(rule.action.label())
                    .show_ui(ui, |ui| {
                        for action in Action::builtins() {
                            let label = action.label();
                            ui.selectable_value(&mut rule.action, action, label);
                        }
                        ui.selectable_value(
                            &mut rule.action,
                            Action::Shortcut(Shortcut {
                                modifiers: Modifiers::default(),
                                key: 0,
                            }),
                            "自定义快捷键…",
                        );
                    });
            });
            if let Action::Shortcut(shortcut) = &mut rule.action {
                ui.label(if shortcut.key == 0 {
                    "尚未录制快捷键".into()
                } else {
                    shortcut.label()
                });
                if self.recording {
                    ui.colored_label(
                        Color32::LIGHT_BLUE,
                        "请按快捷键（一个主键 + 修饰键），Esc 取消录制",
                    );
                    if ui.button("取消录制").clicked() {
                        self.recording = false;
                        self.service.send(Command::Record(false));
                    }
                } else if ui
                    .add_enabled(self.ready, egui::Button::new("录制快捷键"))
                    .clicked()
                {
                    self.recording = true;
                    self.service.send(Command::Record(true));
                }
            }
            if rule.trigger.gesture.is_drag() {
                ui.weak("超过 24 像素后触发一次；松开按钮结束本次操作。");
            }
            if rule.trigger.gesture == Gesture::Hold {
                ui.weak("按住 400 毫秒触发；已识别的拖动优先。");
            }
            if !self.draft_error.is_empty() {
                ui.colored_label(Color32::LIGHT_RED, &self.draft_error);
            }
            ui.horizontal(|ui| {
                commit = ui
                    .add_enabled(!self.recording, egui::Button::new("保存"))
                    .clicked();
                cancel = ui.button("取消").clicked();
            });
        });
        self.draft = Some((index, rule.clone()));
        if commit {
            let mut config = self.config.clone();
            if let Some(index) = index {
                config.rules[index] = rule;
            } else {
                config.rules.push(rule);
            }
            match config.validate() {
                Ok(()) => {
                    self.config = config;
                    self.save();
                    self.draft = None;
                }
                Err(error) => self.draft_error = error,
            }
        }
        if cancel {
            self.draft = None;
            self.recording = false;
            self.service.send(Command::Record(false));
        }
    }
}

fn modifier_controls(ui: &mut egui::Ui, modifiers: &mut Modifiers) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut modifiers.ctrl, "Ctrl");
        ui.checkbox(&mut modifiers.shift, "Shift");
        ui.checkbox(&mut modifiers.alt, "Alt");
        ui.checkbox(&mut modifiers.win, "Win");
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        while let Ok(event) = self.service.events.try_recv() {
            match event {
                Event::State(config) => self.config = config,
                Event::Saved(result) => self.saved(result),
                Event::Ready => {
                    self.ready = true;
                    if self.status == "正在启动输入引擎…" {
                        self.status = "输入引擎已就绪".into();
                    }
                }
                Event::Captured(trigger) => {
                    if self.draft.is_none() {
                        let index = self.config.rules.iter().position(|r| r.trigger == trigger);
                        let rule = index.map(|i| self.config.rules[i].clone()).unwrap_or(Rule {
                            trigger,
                            action: Action::Nothing,
                        });
                        self.open_editor(index, rule);
                    }
                }
                Event::Shortcut(shortcut) => {
                    if let Some((_, rule)) = &mut self.draft {
                        rule.action = Action::Shortcut(shortcut);
                    }
                    self.recording = false;
                }
                Event::RecordingCancelled => self.recording = false,
                Event::Show => {
                    self.hide_pending = None;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    if self.config.check_updates && !self.config.update_repository.is_empty() {
                        self.check_updates();
                    }
                }
                Event::Hide => {
                    if self.hide_pending.take().is_some() && !self.quitting {
                        // Exit only this settings process; the backend retains mappings/tray.
                        self.quitting = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    } else if !self.quitting {
                        self.service.send(Command::CancelHide);
                    }
                }
                Event::Toggle => {
                    self.config.enabled = !self.config.enabled;
                    self.save();
                }
                Event::Quit => {
                    self.quitting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Event::TrayError(error) => {
                    self.hide_pending = None;
                    self.status = error.clone();
                    self.tray_error = Some(error);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                }
                Event::Error(error) => self.status = error,
            }
        }
        if let Some(receiver) = &self.update_rx {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(Some(release)) => {
                        self.update_status = "发现新版本".into();
                        self.release = Some(release);
                    }
                    Ok(None) => self.update_status = "未发现更高版本".into(),
                    Err(error) => self.update_status = format!("检查失败：{error}"),
                }
                self.update_rx = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(200));
            }
        }
        let closing = ctx.input(|i| i.viewport().close_requested());
        if closing && !self.quitting && cfg!(windows) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if self.hide_pending.is_none() {
                self.hide_pending = Some(Instant::now());
                self.tray_error = None;
                self.status = "正在收起到系统托盘…".into();
                self.recording = false;
                self.capture = false;
                self.service.send(Command::Record(false));
                self.service.send(Command::Capture(false));
                self.service.send(Command::HideToTray);
            }
        }
        if let Some(start) = self.hide_pending {
            if start.elapsed() >= Duration::from_secs(5) {
                self.hide_pending = None;
                self.service.send(Command::CancelHide);
                let error =
                    "托盘未响应，窗口已保持打开。请重试，或使用“退出程序”后重新启动。".to_owned();
                self.status = error.clone();
                self.tray_error = Some(error);
            } else {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
        if let Some(error) = self.tray_error.clone() {
            egui::Window::new("托盘提示")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(&error);
                    if ui.button("复制诊断信息").clicked() {
                        ui.ctx().copy_text(error.clone());
                    }
                    ui.horizontal(|ui| {
                        if ui.button("重试收起").clicked() {
                            self.tray_error = None;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("保留窗口").clicked() {
                            self.tray_error = None;
                        }
                        if ui.button("退出程序").clicked() {
                            self.quit(ctx);
                        }
                    });
                });
        }
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Windows Mouse Fix");
                ui.add_enabled_ui(self.draft.is_none() && !self.restore, |ui| {
                    ui.selectable_value(&mut self.tab, 0, "通用");
                    ui.selectable_value(&mut self.tab, 1, "按钮");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(if !self.ready {
                        "未就绪"
                    } else if self.config.enabled {
                        "已启用"
                    } else {
                        "已暂停"
                    });
                });
            });
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(RichText::new(&self.status).small());
        });
        let mut capture = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(self.draft.is_none() && !self.restore, |ui| {
                if self.tab == 0 {
                    egui::ScrollArea::vertical().show(ui, |ui| self.general(ui));
                } else {
                    capture = self.buttons(ui);
                }
            });
        });
        capture &= !closing && ctx.input(|i| i.focused);
        if capture != self.capture {
            self.capture = capture;
            self.service.send(Command::Capture(capture));
        }
        if self.recording && !ctx.input(|i| i.focused) {
            self.recording = false;
            self.service.send(Command::Record(false));
        }
        self.editor(ctx);
        if self.restore {
            egui::Window::new("恢复默认映射")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("默认没有任何映射。此操作会删除全部按钮规则，恢复鼠标原始行为。");
                    ui.horizontal(|ui| {
                        if ui.button("恢复默认").clicked() {
                            self.config.rules.clear();
                            self.config.lock_pointer = false;
                            self.save();
                            self.restore = false;
                        }
                        if ui.button("取消").clicked() {
                            self.restore = false;
                        }
                    });
                });
        }
    }
}
