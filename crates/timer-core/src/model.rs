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
