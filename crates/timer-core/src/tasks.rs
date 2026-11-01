use crate::{Engine, Kind, TimerError};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub archived: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum TaskChange {
    Create { name: String },
    Rename { id: String, name: String },
    Archive { id: String, archived: bool },
    Delete { id: String },
    Reorder { ids: Vec<String> },
}
impl Engine {
    pub fn validate_task(&self, id: Option<&str>) -> Result<(), TimerError> {
        if id.is_some_and(|id| !self.state.tasks.iter().any(|task| task.id == id)) {
            return Err(TimerError::InvalidTask);
        }
        Ok(())
    }
    pub fn select_task(&mut self, id: Option<String>) -> Result<(), TimerError> {
        self.validate_task(id.as_deref())?;
        if id
            .as_ref()
            .is_some_and(|id| self.state.tasks.iter().any(|t| &t.id == id && t.archived))
        {
            return Err(TimerError::InvalidTask);
        }
        self.state.selected_task_id = id.clone();
        if let Some(current) = self
            .state
            .current
            .as_mut()
            .filter(|s| s.kind == Kind::Focus)
        {
            current.task_id = id;
        }
        Ok(())
    }
    pub fn create_task(&mut self, id: String, name: &str) -> Result<(), TimerError> {
        self.add_task(id, name, true)
    }
    pub fn add_task(&mut self, id: String, name: &str, select: bool) -> Result<(), TimerError> {
        let name = name.trim();
        if id.is_empty() || name.is_empty() || name.chars().count() > 40 {
            return Err(TimerError::InvalidTaskName);
        }
        if self
            .state
            .tasks
            .iter()
            .any(|task| task.name.to_lowercase() == name.to_lowercase() || task.id == id)
        {
            return Err(TimerError::DuplicateTask);
        }
        self.state.tasks.push(Task {
            id: id.clone(),
            name: name.into(),
            archived: false,
        });
        if select {
            self.select_task(Some(id))
        } else {
            Ok(())
        }
    }
    fn task_in_use(&self, id: &str) -> bool {
        self.state
            .current
            .as_ref()
            .is_some_and(|s| s.kind == Kind::Focus && s.task_id.as_deref() == Some(id))
    }
    pub fn rename_task(&mut self, id: &str, name: &str) -> Result<(), TimerError> {
        self.validate_task(Some(id))?;
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 40 {
            return Err(TimerError::InvalidTaskName);
        }
        if self
            .state
            .tasks
            .iter()
            .any(|t| t.id != id && t.name.to_lowercase() == name.to_lowercase())
        {
            return Err(TimerError::DuplicateTask);
        }
        self.state
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .unwrap()
            .name = name.into();
        Ok(())
    }
    pub fn archive_task(&mut self, id: &str, archived: bool) -> Result<(), TimerError> {
        self.validate_task(Some(id))?;
        if archived && self.task_in_use(id) {
            return Err(TimerError::TaskInUse);
        }
        self.state
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .unwrap()
            .archived = archived;
        if archived && self.state.selected_task_id.as_deref() == Some(id) {
            self.state.selected_task_id = None;
        }
        Ok(())
    }
    pub fn delete_task(&mut self, id: &str, has_records: bool) -> Result<(), TimerError> {
        self.validate_task(Some(id))?;
        if self.task_in_use(id) {
            return Err(TimerError::TaskInUse);
        }
        if has_records {
            return Err(TimerError::TaskHasRecords);
        }
        self.state.tasks.retain(|t| t.id != id);
        if self.state.selected_task_id.as_deref() == Some(id) {
            self.state.selected_task_id = None;
        }
        Ok(())
    }
    pub fn reorder_tasks(&mut self, ids: &[String]) -> Result<(), TimerError> {
        let unique: std::collections::HashSet<&String> = ids.iter().collect();
        if ids.len() != self.state.tasks.len()
            || unique.len() != ids.len()
            || ids
                .iter()
                .any(|id| !self.state.tasks.iter().any(|t| &t.id == id))
        {
            return Err(TimerError::InvalidTaskOrder);
        }
        self.state.tasks = ids
            .iter()
            .map(|id| {
                self.state
                    .tasks
                    .iter()
                    .find(|t| &t.id == id)
                    .unwrap()
                    .clone()
            })
            .collect();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Phase;

    #[test]
    fn catalog_management_preserves_sessions_and_guards_current_and_historical_tasks() {
        let mut e = Engine::default();
        e.create_task("math".into(), "数学").unwrap();
        e.start("focus".into(), "习题".into(), 1000).unwrap();
        e.advance(61_501, 62_501);
        let before = serde_json::to_value(&e.state.current).unwrap();
        e.add_task("physics".into(), "物理", false).unwrap();
        e.rename_task("math", " 数学复习 ").unwrap();
        e.reorder_tasks(&["physics".into(), "math".into()]).unwrap();
        assert_eq!(serde_json::to_value(&e.state.current).unwrap(), before);
        assert_eq!(e.state.selected_task_id.as_deref(), Some("math"));
        assert!(matches!(
            e.archive_task("math", true),
            Err(TimerError::TaskInUse)
        ));
        assert!(matches!(
            e.delete_task("math", false),
            Err(TimerError::TaskInUse)
        ));
        e.pause(62_501, "manual").unwrap();
        assert!(e.archive_task("math", true).is_err());
        let done = e.finish(62_501).unwrap();
        e.archive_task("math", true).unwrap();
        assert!(e.state.selected_task_id.is_none());
        assert!(e.select_task(Some("math".into())).is_err());
        assert!(e.validate_task(Some("math")).is_ok());
        assert!(matches!(
            e.delete_task("math", true),
            Err(TimerError::TaskHasRecords)
        ));
        assert_eq!(done.task_id.as_deref(), Some("math"));
        e.archive_task("math", false).unwrap();
        assert!(e.state.selected_task_id.is_none());
        e.delete_task("physics", false).unwrap();
        assert_eq!(e.state.tasks[0].name, "数学复习");
    }
    #[test]
    fn invalid_catalog_edits_are_rejected_without_partial_mutation() {
        let mut e = Engine::default();
        e.add_task("a".into(), "Math", false).unwrap();
        e.add_task("b".into(), "Physics", false).unwrap();
        let before = serde_json::to_value(&e.state).unwrap();
        assert!(e.rename_task("b", " math ").is_err());
        assert!(e.rename_task("a", " ").is_err());
        assert!(e.rename_task("a", &"字".repeat(41)).is_err());
        assert!(e.reorder_tasks(&["a".into(), "a".into()]).is_err());
        assert!(e.reorder_tasks(&["a".into()]).is_err());
        assert!(e.reorder_tasks(&["a".into(), "missing".into()]).is_err());
        assert_eq!(serde_json::to_value(&e.state).unwrap(), before);
        let old: Task = serde_json::from_str(r#"{"id":"old","name":"旧分类"}"#).unwrap();
        assert!(!old.archived);
    }
    #[test]
    fn category_reset_waits_for_completion_and_keeps_explicit_next_selection() {
        let mut e = Engine::default();
        e.create_task("math".into(), "数学").unwrap();
        let mut settings = e.state.settings.clone();
        settings.retain_task = false;
        e.configure(settings).unwrap();
        e.start("focus".into(), "".into(), 1000).unwrap();
        e.pause(2000, "manual").unwrap();
        e.resume(3000).unwrap();
        assert_eq!(e.state.selected_task_id.as_deref(), Some("math"));
        let restored = Engine::restore(e.state.clone());
        assert_eq!(
            restored.state.current.unwrap().task_id.as_deref(),
            Some("math")
        );
        let done = e.switch("rest".into(), 4000).unwrap();
        assert_eq!(done.task_id.as_deref(), Some("math"));
        assert!(e.state.selected_task_id.is_none());
        e.select_task(Some("math".into())).unwrap();
        e.resume(5000).unwrap();
        e.switch("next".into(), 6000).unwrap();
        assert_eq!(
            e.state.current.as_ref().unwrap().task_id.as_deref(),
            Some("math")
        );
        e.resume(7000).unwrap();
        e.finish(8000).unwrap();
        assert!(e.state.selected_task_id.is_none());
        e.start("unclassified".into(), "".into(), 9000).unwrap();
        assert!(e.state.current.as_ref().unwrap().task_id.is_none());
    }
    #[test]
    fn timed_completion_resets_category_and_old_settings_keep_previous_behavior() {
        let mut e = Engine::default();
        let mut legacy = serde_json::to_value(&e.state.settings).unwrap();
        legacy.as_object_mut().unwrap().remove("retain_task");
        let settings: crate::Settings = serde_json::from_value(legacy).unwrap();
        assert!(settings.retain_task);
        e.create_task("math".into(), "数学").unwrap();
        let mut settings = settings;
        settings.retain_task = false;
        settings.focus_mode = crate::FocusMode::Timed;
        settings.focus_minutes = 1;
        e.configure(settings).unwrap();
        e.start("focus".into(), "".into(), 0).unwrap();
        let advance = e.advance_phase(60_000, 60_000, "rest".into());
        assert_eq!(advance.completed.unwrap().task_id.as_deref(), Some("math"));
        assert!(e.state.selected_task_id.is_none());
        e.resume(60_000).unwrap();
        e.advance_phase(600_000, 660_000, "next".into());
        assert!(e.state.current.as_ref().unwrap().task_id.is_none());
    }
    #[test]
    fn one_task_per_round_survives_switching_without_changing_effective_time() {
        let mut e = Engine::default();
        e.create_task("math".into(), " 数学 ").unwrap();
        e.start("focus".into(), "习题".into(), 1000).unwrap();
        e.advance(90_123, 91_123);
        e.create_task("physics".into(), "物理").unwrap();
        let current = e.state.current.as_ref().unwrap();
        assert_eq!(current.id, "focus");
        assert_eq!(current.title, "习题");
        assert_eq!(current.duration_ms, 90_123);
        assert_eq!(current.task_id.as_deref(), Some("physics"));
        assert_eq!(e.state.phase, Phase::Running);
        let done = e.switch("break".into(), 91_123).unwrap();
        assert_eq!(done.task_id.as_deref(), Some("physics"));
        assert!(e.state.current.as_ref().unwrap().task_id.is_none());
        e.select_task(Some("math".into())).unwrap();
        e.resume(91_500).unwrap();
        e.switch("next".into(), 92_000).unwrap();
        assert_eq!(
            e.state.current.as_ref().unwrap().task_id.as_deref(),
            Some("math")
        );
        assert!(e.state.current.as_ref().unwrap().intervals.is_empty());
    }
    #[test]
    fn invalid_names_duplicate_names_and_unknown_ids_do_not_mutate_state() {
        let mut e = Engine::default();
        e.create_task("one".into(), "Math").unwrap();
        let before = serde_json::to_value(&e.state).unwrap();
        assert!(e.create_task("two".into(), " math ").is_err());
        assert!(e.create_task("two".into(), "  ").is_err());
        assert!(e.create_task("two".into(), &"字".repeat(41)).is_err());
        assert!(e.select_task(Some("missing".into())).is_err());
        assert_eq!(serde_json::to_value(&e.state).unwrap(), before);
    }
}
