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
    "coffee".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub focus_minutes: u32,
    pub break_minutes: u32,
    pub locale: String,
    pub notifications: bool,
    #[serde(default)]
    pub appearance: Appearance,
    #[serde(default = "default_visual_series")]
    pub visual_series: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            focus_minutes: 50,
            break_minutes: 10,
            locale: "zh-CN".into(),
            notifications: true,
            appearance: Appearance::System,
            visual_series: default_visual_series(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), TimerError> {
        if !(1..=240).contains(&self.focus_minutes) || !(1..=120).contains(&self.break_minutes) {
            return Err(TimerError::InvalidDuration);
        }
        if !["zh-CN", "en-US"].contains(&self.locale.as_str()) {
            return Err(TimerError::InvalidLocale);
        }
        if self.visual_series != "coffee" {
            return Err(TimerError::InvalidSeries);
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
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub target_ms: u64,
    pub duration_ms: u64,
    pub intervals: Vec<Interval>,
    pub corrected: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
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
    #[error("invalid_transition")]
    InvalidTransition,
    #[error("invalid_duration")]
    InvalidDuration,
    #[error("invalid_series")]
    InvalidSeries,
    #[error("invalid_locale")]
    InvalidLocale,
    #[error("title_too_long")]
    TitleTooLong,
}
