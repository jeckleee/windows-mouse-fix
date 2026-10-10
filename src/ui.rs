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
    #[cfg(windows)]
    startup_enabled: bool,
    #[cfg(windows)]
    startup_error: Option<String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        configure_style(ctx);
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
        #[cfg(windows)]
        let startup = crate::startup::enabled();
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
            #[cfg(windows)]
            startup_enabled: startup.as_ref().copied().unwrap_or(false),
            #[cfg(windows)]
            startup_error: startup.err(),
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
        page_heading(ui, "通用设置", "管理运行方式与版本更新");
        ui.strong("运行设置");
        let mut changed = false;
        card(ui).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            changed |= setting(
                ui,
                &mut self.config.enabled,
                "启用按钮映射",
                "关闭窗口后，映射继续在后台运行",
            );
            ui.separator();
            changed |= setting(
                ui,
                &mut self.config.show_tray,
                "在系统托盘中显示",
                "双击托盘图标打开设置；右键可暂停或退出",
            );
            #[cfg(windows)]
            {
                ui.separator();
                let previous = self.startup_enabled;
                if setting(
                    ui,
                    &mut self.startup_enabled,
                    "开机自启",
                    "登录 Windows 后自动驻留托盘；移动程序后请重新勾选",
                ) {
                    match crate::startup::set_enabled(self.startup_enabled) {
                        Ok(()) => self.startup_error = None,
                        Err(error) => {
                            self.startup_enabled = previous;
                            self.startup_error = Some(error);
                        }
                    }
                }
                if let Some(error) = &self.startup_error {
                    ui.colored_label(Color32::LIGHT_RED, format!("自启设置失败：{error}"));
                }
            }
        });
        ui.add_space(6.0);
        ui.strong("版本更新");
        card(ui).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label("更新来源");
            changed |= ui
                .add(
                    egui::TextEdit::singleline(&mut self.config.update_repository)
                        .desired_width(ui.available_width())
                        .hint_text("GitHub 仓库：所有者/仓库名"),
                )
                .changed();
            ui.weak("此 Windows 项目的 GitHub 仓库，未配置时不检查更新。");
            ui.separator();
            let configured = !self.config.update_repository.trim().is_empty();
            ui.add_enabled_ui(configured, |ui| {
                changed |= setting(
                    ui,
                    &mut self.config.check_updates,
                    "打开设置时检查更新",
                    "仅检查版本，不会自动下载或安装",
                );
            });
            ui.add_enabled_ui(configured && self.config.check_updates, |ui| {
                changed |= setting(
                    ui,
                    &mut self.config.beta_updates,
                    "包含 Beta 版本",
                    "同时检查预发布版本",
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        configured && self.update_rx.is_none(),
                        egui::Button::new("立即检查"),
                    )
                    .clicked()
                {
                    self.check_updates();
                }
            });
            if !self.update_status.is_empty() {
                ui.label(&self.update_status);
            }
            if let Some((version, url)) = &self.release {
                ui.hyperlink_to(format!("下载 {version}"), url);
            }
        });
        ui.add_space(6.0);
        ui.strong("应用信息");
        card(ui).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label("当前版本");
                ui.label(env!("CARGO_PKG_VERSION"));
            });
            ui.separator();
            ui.label("配置位置");
            ui.label(
                RichText::new(Config::path().display().to_string())
                    .small()
                    .color(ui.visuals().weak_text_color()),
            );
        });
        ui.horizontal_wrapped(|ui| {
            ui.weak("关闭设置窗口后，程序继续在后台运行。");
            if ui.button("退出程序").clicked() {
                self.quit(ui.ctx());
            }
        });
        if changed {
            self.save();
        }
    }

    fn buttons(&mut self, ui: &mut egui::Ui) -> bool {
        page_heading(ui, "按钮映射", "为中键和侧键设置动作");
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 158.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(
            rect,
            10.0,
            if response.hovered() {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                ui.visuals().faint_bg_color
            },
        );
        painter.rect_stroke(
            rect.shrink(1.0),
            10.0,
            egui::Stroke::new(1.5_f32, accent(ui)),
            egui::StrokeKind::Inside,
        );
        let center = rect.center();
        painter.circle_stroke(
            egui::pos2(center.x, rect.top() + 33.0),
            16.0,
            egui::Stroke::new(1.5_f32, accent(ui)),
        );
        painter.text(
            egui::pos2(center.x, rect.top() + 32.0),
            egui::Align2::CENTER_CENTER,
            "+",
            egui::FontId::proportional(28.0),
            accent(ui),
        );
        painter.text(
            egui::pos2(center.x, rect.top() + 72.0),
            egui::Align2::CENTER_CENTER,
            "将指针移入此区域，操作鼠标按钮以录入",
            egui::FontId::proportional(16.0),
            ui.visuals().text_color(),
        );
        painter.text(
            egui::pos2(center.x, rect.top() + 103.0),
            egui::Align2::CENTER_CENTER,
            "支持单击、双击、长按和四向拖动",
            egui::FontId::proportional(14.0),
            ui.visuals().weak_text_color(),
        );
        painter.text(
            egui::pos2(center.x, rect.top() + 133.0),
            egui::Align2::CENTER_CENTER,
            "可配合 Ctrl / Shift / Alt / Win · 单击需等待双击判定",
            egui::FontId::proportional(13.0),
            ui.visuals().weak_text_color(),
        );
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("＋ 手动添加映射").color(Color32::WHITE))
                        .fill(accent(ui)),
                )
                .clicked()
            {
                self.open_editor(None, Rule::default());
            }
            if ui.button("恢复默认值…").clicked() {
                self.restore = true;
            }
        });
        let mut edit = None;
        let mut remove = None;
        if self.config.rules.is_empty() {
            card(ui).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.strong("尚未添加映射");
                ui.weak("在上方录入操作，或手动添加映射。未绑定的按键保留原始行为。");
            });
        }
        for button in 3..=5 {
            let count = self
                .config
                .rules
                .iter()
                .filter(|r| r.trigger.button == button)
                .count();
            if count == 0 {
                continue;
            }
            card(ui).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.strong(button_label(button));
                    ui.weak(format!("{count} 条映射"));
                });
                ui.separator();
                egui::ScrollArea::horizontal()
                    .id_salt(("rules", button))
                    .show(ui, |ui| {
                        let action_width = (ui.available_width() - 360.0).max(120.0);
                        egui::Grid::new(("rules", button))
                            .num_columns(5)
                            .min_col_width(30.0)
                            .spacing([18.0, 8.0])
                            .show(ui, |ui| {
                                ui.weak("触发操作");
                                ui.label("");
                                ui.weak("执行动作");
                                ui.label("");
                                ui.label("");
                                ui.end_row();
                                for (index, rule) in self
                                    .config
                                    .rules
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, r)| r.trigger.button == button)
                                {
                                    ui.label(format!(
                                        "{}{}",
                                        rule.trigger.modifiers.label(),
                                        rule.trigger.gesture.label()
                                    ));
                                    ui.weak("→");
                                    ui.add_sized(
                                        [action_width, 24.0],
                                        egui::Label::new(rule.action.label())
                                            .halign(egui::Align::Min),
                                    );
                                    if ui
                                        .small_button(RichText::new("编辑").color(accent(ui)))
                                        .clicked()
                                    {
                                        edit = Some(index);
                                    }
                                    if ui.small_button("删除").clicked() {
                                        remove = Some(index);
                                    }
                                    ui.end_row();
                                }
                            });
                    });
            });
        }
        card(ui).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            if setting(
                ui,
                &mut self.config.lock_pointer,
                "执行拖动映射时锁定指针",
                "拖动操作期间保持指针位置",
            ) {
                self.save();
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
        let title = if index.is_some() {
            "编辑映射"
        } else {
            "添加映射"
        };
        egui::Window::new(title)
            .id(egui::Id::new("mapping_editor"))
            .title_bar(false)
            .resizable(false)
            .default_width(460.0)
            .max_width(460.0)
            .frame(
                egui::Frame::window(&ctx.style())
                    .inner_margin(24)
                    .corner_radius(12),
            )
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).size(24.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        cancel |= ui.button("×").on_hover_text("取消并关闭").clicked();
                    });
                });
                ui.weak("选择触发操作，并设置执行动作");
                egui::ScrollArea::vertical()
                    .id_salt("mapping_editor_content")
                    .max_height((ctx.content_rect().height() - 220.0).max(140.0))
                    .show(ui, |ui| {
                        card(ui).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.strong("触发操作");
                            ui.add_enabled_ui(!self.recording, |ui| {
                                ui.columns(2, |columns| {
                                    columns[0].label("鼠标按钮");
                                    egui::ComboBox::from_id_salt("button")
                                        .width(columns[0].available_width())
                                        .selected_text(button_label(rule.trigger.button))
                                        .show_ui(&mut columns[0], |ui| {
                                            for button in 3..=5 {
                                                ui.selectable_value(
                                                    &mut rule.trigger.button,
                                                    button,
                                                    button_label(button),
                                                );
                                            }
                                        });
                                    columns[1].label("操作方式");
                                    egui::ComboBox::from_id_salt("gesture")
                                        .width(columns[1].available_width())
                                        .selected_text(rule.trigger.gesture.label())
                                        .show_ui(&mut columns[1], |ui| {
                                            for gesture in Gesture::ALL {
                                                ui.selectable_value(
                                                    &mut rule.trigger.gesture,
                                                    gesture,
                                                    gesture.label(),
                                                );
                                            }
                                        });
                                });
                                ui.label("修饰键（可选）");
                                modifier_controls(ui, &mut rule.trigger.modifiers);
                                ui.label(
                                    RichText::new("不选择修饰键时，仅使用鼠标按钮触发")
                                        .small()
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            if rule.trigger.gesture.is_drag() {
                                ui.weak("超过 24 像素后触发一次；松开按钮结束本次操作。");
                            }
                            if rule.trigger.gesture == Gesture::Hold {
                                ui.weak("按住 400 毫秒触发；已识别的拖动优先。");
                            }
                        });
                        card(ui).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.strong("执行动作");
                            ui.add_enabled_ui(!self.recording, |ui| {
                                egui::ComboBox::from_id_salt("action")
                                    .width(ui.available_width())
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
                            ui.label(
                                RichText::new("可选择系统动作，或录制自定义快捷键")
                                    .small()
                                    .color(ui.visuals().weak_text_color()),
                            );
                            if let Action::Shortcut(shortcut) = &mut rule.action {
                                ui.separator();
                                ui.label(if shortcut.key == 0 {
                                    "尚未录制快捷键".into()
                                } else {
                                    shortcut.label()
                                });
                                if self.recording {
                                    ui.colored_label(
                                        accent(ui),
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
                        });
                        card(ui).inner_margin(12).show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new(format!(
                                    "{} · {}{}  →  {}",
                                    button_label(rule.trigger.button),
                                    rule.trigger.modifiers.label(),
                                    rule.trigger.gesture.label(),
                                    rule.action.label()
                                ))
                                .color(ui.visuals().weak_text_color()),
                            );
                        });
                        if !self.draft_error.is_empty() {
                            ui.colored_label(Color32::LIGHT_RED, &self.draft_error);
                        }
                    });
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    commit = ui
                        .add_enabled(
                            !self.recording,
                            egui::Button::new(RichText::new("保存映射").color(Color32::WHITE))
                                .fill(accent(ui)),
                        )
                        .clicked();
                    cancel |= ui.button("取消").clicked();
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

fn configure_style(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::System);
    ctx.all_styles_mut(|style| {
        let dark = style.visuals.dark_mode;
        let blue = if dark {
            Color32::from_rgb(91, 155, 255)
        } else {
            Color32::from_rgb(35, 108, 220)
        };
        style
            .text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(26.0));
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        style.spacing.item_spacing = egui::vec2(10.0, 12.0);
        style.spacing.button_padding = egui::vec2(14.0, 7.0);
        style.visuals.panel_fill = if dark {
            Color32::from_rgb(23, 27, 34)
        } else {
            Color32::from_rgb(245, 247, 250)
        };
        style.visuals.faint_bg_color = if dark {
            Color32::from_rgb(32, 38, 48)
        } else {
            Color32::WHITE
        };
        style.visuals.window_fill = style.visuals.panel_fill;
        style.visuals.text_edit_bg_color = Some(style.visuals.panel_fill);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(
            1.0_f32,
            if dark {
                Color32::from_rgb(65, 77, 94)
            } else {
                Color32::from_rgb(190, 200, 214)
            },
        );
        style.visuals.selection.bg_fill = blue;
        style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, Color32::WHITE);
        style.visuals.hyperlink_color = blue;
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(
            1.0_f32,
            if dark {
                Color32::from_rgb(48, 55, 68)
            } else {
                Color32::from_rgb(220, 225, 232)
            },
        );
        for widget in [
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
        ] {
            widget.corner_radius = egui::CornerRadius::same(6);
        }
    });
}

fn accent(ui: &egui::Ui) -> Color32 {
    ui.visuals().hyperlink_color
}

fn card(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(10)
        .inner_margin(16)
}

fn page_heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.heading(title);
    ui.weak(subtitle);
    ui.add_space(4.0);
}

fn setting(ui: &mut egui::Ui, value: &mut bool, title: &str, subtitle: &str) -> bool {
    ui.horizontal(|ui| {
        let text_width = (ui.available_width() - 62.0).max(0.0);
        ui.allocate_ui_with_layout(
            egui::vec2(text_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_min_width(text_width);
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.label(title);
                ui.label(
                    RichText::new(subtitle)
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
            },
        );
        let (rect, mut response) =
            ui.allocate_exact_size(egui::vec2(42.0, 24.0), egui::Sense::click());
        if response.clicked() {
            *value = !*value;
            response.mark_changed();
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *value, title)
        });
        let fill = if *value {
            accent(ui)
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        ui.painter().rect(
            rect,
            12.0,
            fill,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let x = if *value {
            rect.right() - 12.0
        } else {
            rect.left() + 12.0
        };
        ui.painter()
            .circle_filled(egui::pos2(x, rect.center().y), 9.0, Color32::WHITE);
        response.on_hover_text(title).changed()
    })
    .inner
}

fn modifier_controls(ui: &mut egui::Ui, modifiers: &mut Modifiers) {
    ui.columns(4, |columns| {
        for (column, (value, label)) in columns.iter_mut().zip([
            (&mut modifiers.ctrl, "Ctrl"),
            (&mut modifiers.shift, "Shift"),
            (&mut modifiers.alt, "Alt"),
            (&mut modifiers.win, "Win"),
        ]) {
            egui::Frame::new()
                .fill(column.visuals().panel_fill)
                .stroke(column.visuals().widgets.inactive.bg_stroke)
                .corner_radius(6)
                .inner_margin(6)
                .show(column, |ui| {
                    ui.checkbox(value, label);
                });
        }
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
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| ui.label(&error));
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
        egui::TopBottomPanel::top("tabs")
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(20, 12)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("Windows Mouse Fix");
                    ui.add_space(12.0);
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
                        ui.colored_label(
                            if self.ready && self.config.enabled {
                                Color32::from_rgb(83, 200, 125)
                            } else {
                                ui.visuals().weak_text_color()
                            },
                            "●",
                        );
                    });
                });
            });
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(20, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.weak(format!("v{}", env!("CARGO_PKG_VERSION")));
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add(egui::Label::new(RichText::new(&self.status).small()).wrap());
                        });
                    });
                });
            });
        let mut capture = false;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(20),
            )
            .show(ctx, |ui| {
                ui.add_enabled_ui(self.draft.is_none() && !self.restore, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt(("page", self.tab))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            if self.tab == 0 {
                                self.general(ui);
                            } else {
                                capture = self.buttons(ui);
                            }
                        });
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
