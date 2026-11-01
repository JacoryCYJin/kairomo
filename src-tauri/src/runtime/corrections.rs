use super::{now_ms, Runtime};
use timer_core::{correct_time_bounds, ranges_overlap, Kind, Repository};

impl Runtime {
    pub fn correct_in_task(
        &mut self,
        id: &str,
        start: Option<i64>,
        end: Option<i64>,
        task_id: Option<String>,
    ) -> Result<(), String> {
        self.engine
            .validate_task(task_id.as_deref())
            .map_err(|e| e.to_string())?;
        let records = self.repository.sessions().map_err(|e| e.to_string())?;
        let record = records
            .iter()
            .find(|s| s.id == id && s.kind == Kind::Focus && s.ended_at_ms.is_some())
            .ok_or("record_not_found")?;
        let mut updated = match (start, end) {
            (None, None) => record.clone(),
            (Some(a), Some(b)) => correct_time_bounds(record, a, b).map_err(str::to_owned)?,
            _ => return Err("invalid_record_time".into()),
        };
        let time_changed = updated.started_at_ms != record.started_at_ms
            || updated.ended_at_ms != record.ended_at_ms;
        if time_changed {
            let a = updated.started_at_ms;
            let b = updated.ended_at_ms.ok_or("invalid_record_time")?;
            if b > now_ms() {
                return Err("record_time_future".into());
            }
            if let Some(other) = records.iter().find(|other| {
                other.id != id
                    && other.kind == Kind::Focus
                    && other
                        .ended_at_ms
                        .is_some_and(|stop| ranges_overlap(a, b, other.started_at_ms, stop))
            }) {
                return Err(format!(
                    "record_overlap:{}:{}",
                    other.started_at_ms,
                    other.ended_at_ms.unwrap()
                ));
            }
            if let Some(current) = self
                .engine
                .state
                .current
                .as_ref()
                .filter(|s| s.kind == Kind::Focus && !s.intervals.is_empty())
            {
                if ranges_overlap(a, b, current.started_at_ms, i64::MAX) {
                    return Err("record_overlap_current".into());
                }
            }
        }
        updated.task_id = task_id;
        self.repository
            .update_session(&updated)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use timer_core::{Interval, Session};
    fn saved(id: &str, start: i64, end: i64) -> Session {
        Session {
            id: id.into(),
            kind: Kind::Focus,
            title: "原名称".into(),
            task_id: None,
            started_at_ms: start,
            ended_at_ms: Some(end),
            target_ms: 300000,
            focus_mode: timer_core::FocusMode::Manual,
            duration_ms: (end - start) as u64,
            intervals: vec![Interval {
                started_at_ms: start,
                ended_at_ms: Some(end),
                duration_ms: (end - start) as u64,
            }],
            corrected: false,
        }
    }
    #[test]
    fn rejects_overlap_without_writing_and_allows_adjacency_durably() {
        let folder =
            std::env::temp_dir().join(format!("kairomo-correction-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("db.sqlite");
        let mut rt = Runtime::open(&path).unwrap();
        let base = now_ms() - 1_000_000;
        let mut one = saved("one", base, base + 360000);
        one.intervals = vec![
            Interval {
                started_at_ms: base,
                ended_at_ms: Some(base + 120000),
                duration_ms: 120000,
            },
            Interval {
                started_at_ms: base + 180000,
                ended_at_ms: Some(base + 360000),
                duration_ms: 180000,
            },
        ];
        one.duration_ms = 300000;
        let two = saved("two", base + 400000, base + 460000);
        rt.repository
            .persist(&rt.engine.state, &[one.clone(), two])
            .unwrap();
        let before = serde_json::to_value(rt.repository.sessions().unwrap()).unwrap();
        let failure = rt
            .correct_in_task("one", Some(base + 60000), Some(base + 420000), None)
            .unwrap_err();
        assert!(failure.starts_with("record_overlap:"));
        assert_eq!(
            serde_json::to_value(rt.repository.sessions().unwrap()).unwrap(),
            before
        );
        assert!(rt
            .correct_in_task("one", Some(base + 60000), None, None)
            .is_err());
        rt.correct_in_task("one", Some(base + 60000), Some(base + 400000), None)
            .unwrap();
        drop(rt);
        let rt = Runtime::open(&path).unwrap();
        let one = rt
            .repository
            .sessions()
            .unwrap()
            .into_iter()
            .find(|s| s.id == "one")
            .unwrap();
        assert_eq!(one.title, "原名称");
        assert_eq!(one.duration_ms, 280000);
        assert_eq!(one.intervals[0].ended_at_ms, Some(base + 120000));
        assert_eq!(one.intervals[1].started_at_ms, base + 180000);
        assert!(one.corrected);
        drop(rt);
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn classification_only_preserves_time_even_with_legacy_overlap_and_current_focus_blocks_time_edits(
    ) {
        let folder =
            std::env::temp_dir().join(format!("kairomo-correction-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("db.sqlite");
        let mut rt = Runtime::open(&path).unwrap();
        let base = now_ms() - 1_000_000;
        let one = saved("one", base, base + 60000);
        rt.repository
            .persist(&rt.engine.state, std::slice::from_ref(&one))
            .unwrap();
        rt.correct_in_task("one", None, None, None).unwrap();
        assert_eq!(
            serde_json::to_value(rt.repository.sessions().unwrap()[0].clone()).unwrap(),
            serde_json::to_value(one).unwrap()
        );
        rt.engine
            .start("current".into(), "正在专注".into(), base + 120000)
            .unwrap();
        assert_eq!(
            rt.correct_in_task("one", Some(base), Some(base + 180000), None)
                .unwrap_err(),
            "record_overlap_current"
        );
        assert!(rt
            .correct_in_task("one", Some(base), Some(now_ms() + 60000), None)
            .is_err());
        drop(rt);
        std::fs::remove_dir_all(folder).unwrap();
    }
}
