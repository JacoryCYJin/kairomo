use super::*;
impl Runtime {
    pub fn create_task(&mut self, name: &str) -> Result<Snapshot, String> {
        let (mut next, reached, completed, measured) = self.advanced();
        next.create_task(Uuid::new_v4().to_string(), name)
            .map_err(|e| e.to_string())?;
        self.commit_step(next, &completed, measured, reached)
    }
    pub fn select_task(&mut self, task_id: Option<String>) -> Result<Snapshot, String> {
        let (mut next, reached, completed, measured) = self.advanced();
        next.select_task(task_id).map_err(|e| e.to_string())?;
        self.commit_step(next, &completed, measured, reached)
    }
    pub fn manage_tasks(&mut self, change: timer_core::TaskChange) -> Result<Snapshot, String> {
        let (mut next, reached, completed, measured) = self.advanced();
        match change {
            timer_core::TaskChange::Create { name, color } => {
                next.add_task_with_color(Uuid::new_v4().to_string(), &name, color.as_deref(), false)
            }
            timer_core::TaskChange::Rename { id, name } => next.rename_task(&id, &name),
            timer_core::TaskChange::Update { id, name, color } => {
                next.update_task(&id, &name, color.as_deref())
            }
            timer_core::TaskChange::Archive { id, archived } => next.archive_task(&id, archived),
            timer_core::TaskChange::Delete { id } => next.delete_task(&id),
            timer_core::TaskChange::Reorder { ids } => next.reorder_tasks(&ids),
        }
        .map_err(|e| e.to_string())?;
        self.commit_step(next, &completed, measured, reached)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletion_clears_a_focus_record_completed_in_the_same_dispatch() {
        use timer_core::{FocusMode, TaskChange};
        let folder = std::env::temp_dir().join(format!("kairomo-task-deadline-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("db.sqlite");
        let mut rt = Runtime::open(&path).unwrap();
        let id = rt.create_task("数学").unwrap().selected_task_id.unwrap();
        let mut settings = rt.engine.state.settings.clone();
        settings.focus_mode = FocusMode::Timed;
        settings.focus_minutes = 1;
        rt.configure(settings).unwrap();
        rt.action("start", "").unwrap();
        rt.engine.advance(60_000, now_ms());
        assert!(rt.repository.sessions().unwrap().is_empty());
        rt.manage_tasks(TaskChange::Delete { id: id.clone() })
            .unwrap();
        assert!(!rt.engine.state.tasks.iter().any(|t| t.id == id));
        let records = rt.repository.sessions().unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0].task_id.is_none());
        drop(rt);
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn management_is_durable_without_rewriting_historical_time_or_category_ids() {
        use timer_core::TaskChange;
        let folder = std::env::temp_dir().join(format!("kairomo-management-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("db.sqlite");
        let mut rt = Runtime::open(&path).unwrap();
        let math = rt.create_task("数学").unwrap().selected_task_id.unwrap();
        rt.action("start", "旧记录内容").unwrap();
        rt.engine.advance(61_501, now_ms());
        rt.action("pause", "").unwrap();
        assert!(rt
            .manage_tasks(TaskChange::Archive {
                id: math.clone(),
                archived: true
            })
            .is_err());
        rt.action("finish", "").unwrap();
        let original = rt.repository.sessions().unwrap();
        rt.manage_tasks(TaskChange::Create {
            name: "物理".into(),
            color: Some("#3e63dd".into()),
        })
        .unwrap();
        let physics = rt
            .engine
            .state
            .tasks
            .iter()
            .find(|t| t.name == "物理")
            .unwrap()
            .id
            .clone();
        assert_eq!(
            rt.engine.state.selected_task_id.as_deref(),
            Some(math.as_str())
        );
        rt.manage_tasks(TaskChange::Rename {
            id: math.clone(),
            name: "数学复习".into(),
        })
        .unwrap();
        rt.manage_tasks(TaskChange::Reorder {
            ids: vec![physics.clone(), math.clone()],
        })
        .unwrap();
        rt.manage_tasks(TaskChange::Archive {
            id: math.clone(),
            archived: true,
        })
        .unwrap();
        drop(rt);
        let mut rt = Runtime::open(&path).unwrap();
        assert_eq!(rt.engine.state.tasks[0].id, physics);
        assert_eq!(rt.engine.state.tasks[1].name, "数学复习");
        assert!(rt.engine.state.tasks[1].archived);
        assert_eq!(
            serde_json::to_value(rt.repository.sessions().unwrap()).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        rt.manage_tasks(TaskChange::Archive {
            id: math.clone(),
            archived: false,
        })
        .unwrap();
        rt.manage_tasks(TaskChange::Delete { id: math.clone() })
            .unwrap();
        assert_eq!(rt.engine.state.tasks.len(), 1);
        drop(rt);
        let rt = Runtime::open(&path).unwrap();
        assert_eq!(rt.engine.state.tasks.len(), 1);
        assert_eq!(rt.engine.state.tasks[0].id, physics);
        let records = rt.repository.sessions().unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0].task_id.is_none());
        assert_eq!(records[0].started_at_ms, original[0].started_at_ms);
        assert_eq!(records[0].duration_ms, original[0].duration_ms);
        assert_eq!(
            serde_json::to_value(&records[0].intervals).unwrap(),
            serde_json::to_value(&original[0].intervals).unwrap()
        );
        drop(rt);
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn category_preference_is_durable_and_preserves_the_finished_record() {
        let folder = std::env::temp_dir().join(format!("kairomo-task-setting-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("db.sqlite");
        let mut rt = Runtime::open(&path).unwrap();
        let task = rt.create_task("数学").unwrap().selected_task_id;
        rt.action("start", "").unwrap();
        rt.engine.advance(61_501, now_ms());
        rt.action("pause", "").unwrap();
        let mut settings = rt.engine.state.settings.clone();
        settings.retain_task = false;
        rt.configure(settings).unwrap();
        assert_eq!(rt.engine.state.current.as_ref().unwrap().task_id, task);
        drop(rt);
        let mut rt = Runtime::open(&path).unwrap();
        assert!(!rt.engine.state.settings.retain_task);
        assert_eq!(rt.engine.state.current.as_ref().unwrap().task_id, task);
        rt.action("finish", "").unwrap();
        assert!(rt.engine.state.selected_task_id.is_none());
        assert_eq!(rt.repository.sessions().unwrap()[0].task_id, task);
        drop(rt);
        let rt = Runtime::open(&path).unwrap();
        assert!(rt.engine.state.selected_task_id.is_none());
        drop(rt);
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn task_reassignment_is_durable_and_preserves_exact_record_intervals() {
        let folder = std::env::temp_dir().join(format!("kairomo-tasks-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("db.sqlite");
        let mut rt = Runtime::open(&path).unwrap();
        let state = rt.create_task("数学").unwrap();
        let math = state.selected_task_id.unwrap();
        rt.action("start", "原记录内容").unwrap();
        rt.engine.advance(61_501, now_ms());
        rt.engine.pause(now_ms(), "manual").unwrap();
        rt.action("finish", "").unwrap();
        let before = rt.repository.sessions().unwrap()[0].clone();
        let state = rt.create_task("物理").unwrap();
        let physics = state.selected_task_id.unwrap();
        rt.correct_in_task(&before.id, None, None, Some(physics.clone()))
            .unwrap();
        let after = rt.repository.sessions().unwrap()[0].clone();
        assert_eq!(after.task_id.as_deref(), Some(physics.as_str()));
        assert_eq!(after.duration_ms, before.duration_ms);
        assert_eq!(
            serde_json::to_value(&after.intervals).unwrap(),
            serde_json::to_value(&before.intervals).unwrap()
        );
        assert_eq!(after.started_at_ms, before.started_at_ms);
        assert_eq!(after.ended_at_ms, before.ended_at_ms);
        assert_eq!(after.corrected, before.corrected);
        assert!(rt
            .correct_in_task(&before.id, None, None, Some("missing".into()))
            .is_err());
        rt.select_task(Some(math.clone())).unwrap();
        drop(rt);
        let reopened = Runtime::open(&path).unwrap();
        assert_eq!(reopened.engine.state.tasks.len(), 2);
        assert_eq!(
            reopened.engine.state.selected_task_id.as_deref(),
            Some(math.as_str())
        );
        assert_eq!(
            reopened.repository.sessions().unwrap()[0]
                .task_id
                .as_deref(),
            Some(physics.as_str())
        );
        drop(reopened);
        std::fs::remove_dir_all(folder).unwrap();
    }
}
