use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Focus,
    Break,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Running,
    Paused,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

fn default_visual_series() -> String {
    crate::themes::catalog()["default"]
        .as_str()
        .expect("default theme")
        .into()
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BeanRange {
    #[default]
    Day,
    Week,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPickerStyle {
    #[default]
    Wheel,
    Select,
}

fn default_retain_task() -> bool {
    true
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutKind {
    Toggle,
    Pause,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutModifier {
    Control,
    Alt,
    Shift,
    Meta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShortcutBinding {
    #[serde(default)]
    pub modifiers: Vec<ShortcutModifier>,
    pub key: String,
}

impl ShortcutBinding {
    pub fn toggle_default() -> Self {
        Self {
            modifiers: vec![ShortcutModifier::Control, ShortcutModifier::Alt],
            key: "KeyK".into(),
        }
    }

    pub fn pause_default() -> Self {
        Self {
            modifiers: vec![ShortcutModifier::Control, ShortcutModifier::Alt],
            key: "Enter".into(),
        }
    }

    pub fn canonical(&self) -> Self {
        let mut modifiers = self.modifiers.clone();
        modifiers.sort();
        Self {
            modifiers,
            key: self.key.clone(),
        }
    }

    pub fn same(&self, other: &Self) -> bool {
        self.canonical() == other.canonical()
    }

    pub fn has_required_modifier(&self) -> bool {
        self.modifiers.iter().any(|modifier| {
            matches!(
                modifier,
                ShortcutModifier::Control | ShortcutModifier::Alt | ShortcutModifier::Meta
            )
        })
    }

    pub fn validate(&self) -> Result<(), TimerError> {
        if !self.has_required_modifier() {
            return Err(TimerError::InvalidShortcut);
        }
        let mut seen = std::collections::HashSet::new();
        for modifier in &self.modifiers {
            if !seen.insert(*modifier) {
                return Err(TimerError::InvalidShortcut);
            }
        }
        if !supported_shortcut_key(&self.key) {
            return Err(TimerError::InvalidShortcut);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShortcutBindings {
    #[serde(default = "ShortcutBinding::toggle_default")]
    pub toggle: ShortcutBinding,
    #[serde(default = "ShortcutBinding::pause_default")]
    pub pause: ShortcutBinding,
    #[serde(default = "default_true")]
    pub toggle_enabled: bool,
    #[serde(default = "default_true")]
    pub pause_enabled: bool,
}

impl Default for ShortcutBindings {
    fn default() -> Self {
        Self {
            toggle: ShortcutBinding::toggle_default(),
            pause: ShortcutBinding::pause_default(),
            toggle_enabled: true,
            pause_enabled: true,
        }
    }
}

impl ShortcutBindings {
    pub fn validate(&self) -> Result<(), TimerError> {
        self.toggle.validate()?;
        self.pause.validate()?;
        if self.toggle.same(&self.pause) {
            return Err(TimerError::InvalidShortcut);
        }
        Ok(())
    }

    /// macOS reserves Control-Option-Space for switching input sources.
    pub fn migrate_legacy_toggle_default(&mut self) {
        let legacy = ShortcutBinding {
            modifiers: vec![ShortcutModifier::Control, ShortcutModifier::Alt],
            key: "Space".into(),
        };
        if self.toggle.same(&legacy) {
            self.toggle = ShortcutBinding::toggle_default();
        }
    }

    pub fn for_binding(&self, kind: ShortcutKind) -> &ShortcutBinding {
        match kind {
            ShortcutKind::Toggle => &self.toggle,
            ShortcutKind::Pause => &self.pause,
        }
    }

    pub fn enabled(&self, kind: ShortcutKind) -> bool {
        match kind {
            ShortcutKind::Toggle => self.toggle_enabled,
            ShortcutKind::Pause => self.pause_enabled,
        }
    }

    pub fn set_enabled(&mut self, kind: ShortcutKind, enabled: bool) {
        match kind {
            ShortcutKind::Toggle => self.toggle_enabled = enabled,
            ShortcutKind::Pause => self.pause_enabled = enabled,
        }
    }

    pub fn set_binding(&mut self, kind: ShortcutKind, binding: ShortcutBinding) {
        match kind {
            ShortcutKind::Toggle => self.toggle = binding,
            ShortcutKind::Pause => self.pause = binding,
        }
    }
}

fn supported_shortcut_key(key: &str) -> bool {
    if key == "Escape" {
        return false;
    }
    matches!(
        key,
        "Space"
            | "Enter"
            | "Tab"
            | "Backspace"
            | "Delete"
            | "Insert"
            | "Home"
            | "End"
            | "PageUp"
            | "PageDown"
            | "ArrowUp"
            | "ArrowDown"
            | "ArrowLeft"
            | "ArrowRight"
            | "Backquote"
            | "Minus"
            | "Equal"
            | "BracketLeft"
            | "BracketRight"
            | "Backslash"
            | "Semicolon"
            | "Quote"
            | "Comma"
            | "Period"
            | "Slash"
            | "Numpad0"
            | "Numpad1"
            | "Numpad2"
            | "Numpad3"
            | "Numpad4"
            | "Numpad5"
            | "Numpad6"
            | "Numpad7"
            | "Numpad8"
            | "Numpad9"
            | "NumpadAdd"
            | "NumpadSubtract"
            | "NumpadMultiply"
            | "NumpadDivide"
            | "NumpadDecimal"
            | "NumpadEnter"
            | "NumpadEqual"
    ) || key
        .strip_prefix("Key")
        .is_some_and(|rest| rest.len() == 1 && rest.as_bytes()[0].is_ascii_uppercase())
        || key
            .strip_prefix("Digit")
            .is_some_and(|rest| rest.len() == 1 && rest.as_bytes()[0].is_ascii_digit())
        || key
            .strip_prefix('F')
            .is_some_and(|rest| matches!(rest.parse::<u8>(), Ok(1..=24)))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub bean_range: BeanRange,
    #[serde(default)]
    pub theme_options:
        std::collections::BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    pub phrases: crate::Phrases,
    pub focus_minutes: u32,
    pub break_minutes: u32,
    pub locale: String,
    pub notifications: bool,
    #[serde(default = "default_retain_task")]
    pub retain_task: bool,
    #[serde(default)]
    pub focus_mode: FocusMode,
    #[serde(default)]
    pub appearance: Appearance,
    #[serde(default = "default_visual_series")]
    pub visual_series: String,
    #[serde(default)]
    pub task_colors: Vec<String>,
    #[serde(default)]
    pub task_picker: TaskPickerStyle,
    #[serde(default)]
    pub global_shortcuts: bool,
    #[serde(default)]
    pub shortcut_bindings: ShortcutBindings,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            phrases: crate::Phrases::default(),
            bean_range: BeanRange::default(),
            theme_options: Default::default(),
            focus_minutes: 50,
            break_minutes: 10,
            locale: "zh-CN".into(),
            notifications: true,
            retain_task: true,
            focus_mode: FocusMode::Manual,
            appearance: Appearance::System,
            visual_series: default_visual_series(),
            task_colors: Vec::new(),
            task_picker: TaskPickerStyle::Wheel,
            global_shortcuts: false,
            shortcut_bindings: ShortcutBindings::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FocusMode {
    #[default]
    Manual,
    Timed,
}
impl Settings {
    pub fn validate(&self) -> Result<(), TimerError> {
        if !self.phrases.valid() {
            return Err(TimerError::InvalidPhrases);
        }
        if !(1..=240).contains(&self.focus_minutes) || !(1..=120).contains(&self.break_minutes) {
            return Err(TimerError::InvalidDuration);
        }
        if !["zh-CN", "zh-TW", "en-US", "ja-JP"].contains(&self.locale.as_str()) {
            return Err(TimerError::InvalidLocale);
        }
        if !crate::themes::registered(&self.visual_series) {
            return Err(TimerError::InvalidSeries);
        }
        self.shortcut_bindings.validate()?;
        crate::themes::validate_saved_options(&self.theme_options)?;
        if self.task_colors.len() > 24 {
            return Err(TimerError::InvalidTaskColor);
        }
        let mut seen = std::collections::HashSet::new();
        for color in &self.task_colors {
            let Some(normalized) = crate::tasks::normalize_color(Some(color))? else {
                return Err(TimerError::InvalidTaskColor);
            };
            if !seen.insert(normalized) {
                return Err(TimerError::InvalidTaskColor);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Interval {
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub duration_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub kind: Kind,
    pub title: String,
    #[serde(default)]
    pub task_id: Option<String>,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub target_ms: u64,
    /// Capture the mode for this round; later settings changes affect the next round.
    #[serde(default)]
    pub focus_mode: FocusMode,
    pub duration_ms: u64,
    pub intervals: Vec<Interval>,
    pub corrected: bool,
}

impl Session {
    /// Eligibility for a newly completed record; historical reads and corrections keep their data.
    pub fn is_recordable_focus(&self) -> bool {
        self.kind == Kind::Focus
            && self.ended_at_ms.is_some()
            && !self.intervals.is_empty()
            && self.duration_ms >= 60_000
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default)]
    pub tasks: Vec<crate::Task>,
    #[serde(default)]
    pub selected_task_id: Option<String>,
    /// Internal visual seed; persisted independently of editable preferences.
    #[serde(default)]
    pub bean_random_seed: Option<String>,
    #[serde(default)]
    pub phrase_selection: Option<crate::PhraseSelection>,
    #[serde(default)]
    pub revision: u64,
    pub phase: Phase,
    pub current: Option<Session>,
    pub settings: Settings,
    pub needs_recovery: bool,
    pub pause_reason: Option<String>,
    pub checkpoint_at_ms: i64,
    pub target_notified: bool,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            selected_task_id: None,
            bean_random_seed: None,
            phrase_selection: None,
            revision: 0,
            phase: Phase::Idle,
            current: None,
            settings: Settings::default(),
            needs_recovery: false,
            pause_reason: None,
            checkpoint_at_ms: 0,
            target_notified: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TimerError {
    #[error("invalid_task")]
    InvalidTask,
    #[error("task_in_use")]
    TaskInUse,
    #[error("invalid_task_order")]
    InvalidTaskOrder,
    #[error("invalid_task_name")]
    InvalidTaskName,
    #[error("invalid_task_color")]
    InvalidTaskColor,
    #[error("duplicate_task")]
    DuplicateTask,
    #[error("invalid_phrases")]
    InvalidPhrases,
    #[error("invalid_transition")]
    InvalidTransition,
    #[error("invalid_duration")]
    InvalidDuration,
    #[error("invalid_series")]
    InvalidSeries,
    #[error("invalid_theme_options")]
    InvalidThemeOptions,
    #[error("invalid_locale")]
    InvalidLocale,
    #[error("invalid_shortcut")]
    InvalidShortcut,
    #[error("title_too_long")]
    TitleTooLong,
}

#[cfg(test)]
mod task_color_settings_tests {
    use super::*;
    #[test]
    fn custom_task_colors_are_optional_and_validated() {
        let mut legacy = serde_json::to_value(Settings::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("task_colors");
        let legacy: Settings = serde_json::from_value(legacy).unwrap();
        assert!(legacy.task_colors.is_empty());
        let mut settings = Settings {
            task_colors: vec!["#3E63DD".into(), "#e2a336".into()],
            ..Settings::default()
        };
        assert!(settings.validate().is_ok());
        settings.task_colors = vec!["#3e63dd".into(), "#3E63DD".into()];
        assert!(matches!(
            settings.validate(),
            Err(TimerError::InvalidTaskColor)
        ));
        settings.task_colors = vec!["red".into()];
        assert!(matches!(
            settings.validate(),
            Err(TimerError::InvalidTaskColor)
        ));
        settings.task_colors = vec!["#3e63dd".into(); 25];
        assert!(matches!(
            settings.validate(),
            Err(TimerError::InvalidTaskColor)
        ));
    }
    #[test]
    fn global_shortcuts_default_to_off_for_legacy_settings_and_round_trip() {
        let mut legacy = serde_json::to_value(Settings::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("global_shortcuts");
        let legacy: Settings = serde_json::from_value(legacy).unwrap();
        assert!(!legacy.global_shortcuts);
        let settings = Settings {
            global_shortcuts: true,
            ..Settings::default()
        };
        let encoded = serde_json::to_string(&settings).unwrap();
        assert!(
            serde_json::from_str::<Settings>(&encoded)
                .unwrap()
                .global_shortcuts
        );
    }
    #[test]
    fn shortcut_bindings_default_for_legacy_settings_and_validate_custom_values() {
        let mut legacy = serde_json::to_value(Settings::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("shortcut_bindings");
        let legacy: Settings = serde_json::from_value(legacy).unwrap();
        assert_eq!(legacy.shortcut_bindings, ShortcutBindings::default());

        let custom = ShortcutBinding {
            modifiers: vec![
                ShortcutModifier::Shift,
                ShortcutModifier::Meta,
                ShortcutModifier::Control,
            ],
            key: "KeyK".into(),
        };
        let settings = Settings {
            shortcut_bindings: ShortcutBindings {
                toggle: custom.clone(),
                pause: ShortcutBinding {
                    modifiers: vec![ShortcutModifier::Control, ShortcutModifier::Alt],
                    key: "Enter".into(),
                },
                ..ShortcutBindings::default()
            },
            ..Settings::default()
        };
        assert!(settings.validate().is_ok());
        let encoded = serde_json::to_string(&settings).unwrap();
        assert_eq!(
            serde_json::from_str::<Settings>(&encoded)
                .unwrap()
                .shortcut_bindings
                .toggle,
            custom
        );

        let mut invalid = settings.clone();
        invalid.shortcut_bindings.toggle = ShortcutBinding {
            modifiers: vec![ShortcutModifier::Shift],
            key: "KeyK".into(),
        };
        assert!(matches!(
            invalid.validate(),
            Err(TimerError::InvalidShortcut)
        ));
        invalid.shortcut_bindings.toggle = ShortcutBinding {
            modifiers: vec![ShortcutModifier::Control, ShortcutModifier::Control],
            key: "KeyK".into(),
        };
        assert!(matches!(
            invalid.validate(),
            Err(TimerError::InvalidShortcut)
        ));
        invalid.shortcut_bindings.toggle = ShortcutBinding {
            modifiers: vec![ShortcutModifier::Control],
            key: "Escape".into(),
        };
        assert!(matches!(
            invalid.validate(),
            Err(TimerError::InvalidShortcut)
        ));
        invalid.shortcut_bindings.toggle = ShortcutBinding {
            modifiers: vec![ShortcutModifier::Control],
            key: "NotAKey".into(),
        };
        assert!(matches!(
            invalid.validate(),
            Err(TimerError::InvalidShortcut)
        ));
        invalid.shortcut_bindings = ShortcutBindings {
            toggle: ShortcutBinding::toggle_default(),
            pause: ShortcutBinding::toggle_default(),
            ..ShortcutBindings::default()
        };
        assert!(matches!(
            invalid.validate(),
            Err(TimerError::InvalidShortcut)
        ));
        assert!(ShortcutBinding {
            modifiers: vec![ShortcutModifier::Alt, ShortcutModifier::Control],
            key: "KeyK".into(),
        }
        .same(&ShortcutBinding {
            modifiers: vec![ShortcutModifier::Control, ShortcutModifier::Alt],
            key: "KeyK".into(),
        }));

        let mut legacy_default = ShortcutBindings {
            toggle: ShortcutBinding {
                modifiers: vec![ShortcutModifier::Control, ShortcutModifier::Alt],
                key: "Space".into(),
            },
            pause: ShortcutBinding::pause_default(),
            ..ShortcutBindings::default()
        };
        legacy_default.migrate_legacy_toggle_default();
        assert_eq!(legacy_default.toggle, ShortcutBinding::toggle_default());
    }
    #[test]
    fn task_picker_style_defaults_to_wheel_and_round_trips() {
        let mut legacy = serde_json::to_value(Settings::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("task_picker");
        let legacy: Settings = serde_json::from_value(legacy).unwrap();
        assert_eq!(legacy.task_picker, TaskPickerStyle::Wheel);
        let settings = Settings {
            task_picker: TaskPickerStyle::Select,
            ..Settings::default()
        };
        let encoded = serde_json::to_string(&settings).unwrap();
        assert_eq!(
            serde_json::from_str::<Settings>(&encoded)
                .unwrap()
                .task_picker,
            TaskPickerStyle::Select
        );
    }
}

#[cfg(test)]
mod bean_range_tests {
    use super::*;
    #[test]
    fn only_day_and_week_are_accepted_and_legacy_settings_default_to_day() {
        assert_eq!(
            serde_json::from_str::<BeanRange>("\"day\"").unwrap(),
            BeanRange::Day
        );
        assert_eq!(
            serde_json::from_str::<BeanRange>("\"week\"").unwrap(),
            BeanRange::Week
        );
        assert!(serde_json::from_str::<BeanRange>("\"month\"").is_err());
        let settings: Settings = serde_json::from_str(
            r#"{"focus_minutes":50,"break_minutes":10,"locale":"zh-CN","notifications":true}"#,
        )
        .unwrap();
        assert_eq!(settings.bean_range, BeanRange::Day);
    }
}
