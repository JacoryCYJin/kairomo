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
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BeanRange {
    #[default]
    Day,
    Week,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub bean_range: BeanRange,
    #[serde(default)]
    pub phrases: crate::Phrases,
    pub focus_minutes: u32,
    pub break_minutes: u32,
    pub locale: String,
    pub notifications: bool,
    #[serde(default)]
    pub focus_mode: FocusMode,
    #[serde(default)]
    pub appearance: Appearance,
    #[serde(default = "default_visual_series")]
    pub visual_series: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            phrases: crate::Phrases::default(),
            bean_range: BeanRange::default(),
            focus_minutes: 50,
            break_minutes: 10,
            locale: "zh-CN".into(),
            notifications: true,
            focus_mode: FocusMode::Manual,
            appearance: Appearance::System,
            visual_series: default_visual_series(),
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
    #[error("invalid_phrases")]
    InvalidPhrases,
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
