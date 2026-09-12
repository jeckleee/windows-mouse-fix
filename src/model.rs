use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub win: bool,
}

impl Modifiers {
    pub fn label(self) -> String {
        [
            (self.ctrl, "Ctrl+"),
            (self.shift, "Shift+"),
            (self.alt, "Alt+"),
            (self.win, "Win+"),
        ]
        .into_iter()
        .filter_map(|(set, label)| set.then_some(label))
        .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gesture {
    Click,
    DoubleClick,
    Hold,
    DragLeft,
    DragRight,
    DragUp,
    DragDown,
}

impl Gesture {
    pub const ALL: [Self; 7] = [
        Self::Click,
        Self::DoubleClick,
        Self::Hold,
        Self::DragLeft,
        Self::DragRight,
        Self::DragUp,
        Self::DragDown,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Click => "单击",
            Self::DoubleClick => "双击",
            Self::Hold => "长按",
            Self::DragLeft => "按住向左拖动",
            Self::DragRight => "按住向右拖动",
            Self::DragUp => "按住向上拖动",
            Self::DragDown => "按住向下拖动",
        }
    }
    pub fn is_drag(self) -> bool {
        matches!(
            self,
            Self::DragLeft | Self::DragRight | Self::DragUp | Self::DragDown
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trigger {
    pub button: u8,
    pub modifiers: Modifiers,
    pub gesture: Gesture,
}

impl Default for Trigger {
    fn default() -> Self {
        Self {
            button: 3,
            modifiers: Modifiers::default(),
            gesture: Gesture::Click,
        }
    }
}

pub fn button_label(button: u8) -> &'static str {
    match button {
        3 => "中键",
        4 => "按钮 4",
        5 => "按钮 5",
        _ => "未知按钮",
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: u16,
}

pub fn key_label(key: u16) -> String {
    match key {
        0x30..=0x39 | 0x41..=0x5a => char::from_u32(key as u32).unwrap().to_string(),
        0x70..=0x87 => format!("F{}", key - 0x70 + 1),
        0x60..=0x69 => format!("Num{}", key - 0x60),
        8 => "Backspace".into(),
        9 => "Tab".into(),
        13 => "Enter".into(),
        27 => "Esc".into(),
        32 => "Space".into(),
        33 => "PageUp".into(),
        34 => "PageDown".into(),
        35 => "End".into(),
        36 => "Home".into(),
        37 => "Left".into(),
        38 => "Up".into(),
        39 => "Right".into(),
        40 => "Down".into(),
        45 => "Insert".into(),
        46 => "Delete".into(),
        _ => format!("VK {key:#04X}"),
    }
}

impl Shortcut {
    pub fn label(&self) -> String {
        format!("{}{}", self.modifiers.label(), key_label(self.key))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    #[default]
    Nothing,
    Shortcut(Shortcut),
    Back,
    Forward,
    TaskView,
    DesktopLeft,
    DesktopRight,
    ShowDesktop,
    VolumeUp,
    VolumeDown,
    Mute,
    PlayPause,
}

impl Action {
    pub fn builtins() -> Vec<Self> {
        vec![
            Self::Nothing,
            Self::Back,
            Self::Forward,
            Self::TaskView,
            Self::DesktopLeft,
            Self::DesktopRight,
            Self::ShowDesktop,
            Self::VolumeUp,
            Self::VolumeDown,
            Self::Mute,
            Self::PlayPause,
        ]
    }
    pub fn label(&self) -> String {
        match self {
            Self::Shortcut(shortcut) if shortcut.key == 0 => "自定义快捷键（未录制）".into(),
            Self::Shortcut(shortcut) => shortcut.label(),
            Self::Nothing => "不执行任何动作".into(),
            Self::Back => "返回".into(),
            Self::Forward => "前进".into(),
            Self::TaskView => "任务视图".into(),
            Self::DesktopLeft => "切换到左侧桌面".into(),
            Self::DesktopRight => "切换到右侧桌面".into(),
            Self::ShowDesktop => "显示桌面".into(),
            Self::VolumeUp => "增大音量".into(),
            Self::VolumeDown => "减小音量".into(),
            Self::Mute => "静音".into(),
            Self::PlayPause => "播放 / 暂停".into(),
        }
    }
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn shortcut(&self) -> Option<Shortcut> {
        let (key, ctrl, win) = match self {
            Self::Nothing => return None,
            Self::Shortcut(s) => return Some(s.clone()),
            Self::Back => (0xa6, false, false),
            Self::Forward => (0xa7, false, false),
            Self::TaskView => (9, false, true),
            Self::DesktopLeft => (37, true, true),
            Self::DesktopRight => (39, true, true),
            Self::ShowDesktop => (0x44, false, true),
            Self::VolumeUp => (0xaf, false, false),
            Self::VolumeDown => (0xae, false, false),
            Self::Mute => (0xad, false, false),
            Self::PlayPause => (0xb3, false, false),
        };
        Some(Shortcut {
            key,
            modifiers: Modifiers {
                ctrl,
                win,
                ..Default::default()
            },
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub trigger: Trigger,
    pub action: Action,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub enabled: bool,
    pub show_tray: bool,
    pub lock_pointer: bool,
    pub check_updates: bool,
    pub beta_updates: bool,
    pub update_repository: String,
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: true,
            show_tray: true,
            lock_pointer: false,
            check_updates: false,
            beta_updates: false,
            update_repository: String::new(),
            rules: vec![],
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("不支持此配置版本".into());
        }
        for (i, rule) in self.rules.iter().enumerate() {
            if !(3..=5).contains(&rule.trigger.button) {
                return Err("只支持中键及按钮 4、5".into());
            }
            if self.rules[..i].iter().any(|r| r.trigger == rule.trigger) {
                return Err("同一触发操作不能重复绑定".into());
            }
            if let Action::Shortcut(s) = &rule.action
                && (!(1..=254).contains(&s.key) || is_modifier(s.key))
            {
                return Err("快捷键必须包含一个非修饰键".into());
            }
        }
        Ok(())
    }
    pub fn path() -> PathBuf {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("Library/Application Support")
            });
        base.join("WindowsMouseFix/config.json")
    }
    pub fn load() -> Result<Self, String> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let config: Self = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        config.validate()?;
        Ok(config)
    }
    pub fn save(&self) -> Result<(), String> {
        self.validate()?;
        let path = Self::path();
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let temp = path.with_extension("tmp");
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        use std::io::Write;
        let mut file = std::fs::File::create(&temp).map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        crate::platform::replace_file(&temp, &path)
    }
}

pub fn is_modifier(key: u16) -> bool {
    matches!(key, 0x10..=0x12 | 0x5b..=0x5c | 0xa0..=0xa5)
}
