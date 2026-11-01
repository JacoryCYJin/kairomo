use crate::{Interval, Session};

pub fn ranges_overlap(start: i64, end: i64, other_start: i64, other_end: i64) -> bool {
    start < other_end && other_start < end
}

/// Keep existing pauses and effective durations; only crop or extend the outer bounds.
pub fn correct_time_bounds(
    record: &Session,
    start: i64,
    end: i64,
) -> Result<Session, &'static str> {
    let old_end = record.ended_at_ms.ok_or("invalid_record_time")?;
    if start < 0 || end <= start || end.checked_sub(start).is_none() {
        return Err("invalid_record_time");
    }
    if start == record.started_at_ms && end == old_end {
        return Ok(record.clone());
    }
    let mut intervals = Vec::new();
    fn push(intervals: &mut Vec<Interval>, a: i64, b: i64, duration_ms: u64) {
        if b > a && duration_ms > 0 {
            intervals.push(Interval {
                started_at_ms: a,
                ended_at_ms: Some(b),
                duration_ms,
            });
        }
    }
    if start < record.started_at_ms {
        let b = end.min(record.started_at_ms);
        push(&mut intervals, start, b, (b - start).max(0) as u64);
    }
    for interval in &record.intervals {
        let stop = interval.ended_at_ms.ok_or("invalid_record_time")?;
        let a = start.max(interval.started_at_ms);
        let b = end.min(stop);
        if a < b {
            let width = (stop - interval.started_at_ms) as u128;
            // Difference of cumulative portions avoids rounding drift when both ends are cropped.
            let portion = |at: i64| -> u64 {
                ((at - interval.started_at_ms) as u128 * interval.duration_ms as u128 / width)
                    as u64
            };
            push(&mut intervals, a, b, portion(b) - portion(a));
        } else if stop == interval.started_at_ms && start <= stop && stop < end {
            // Preserve legacy instantaneous effective intervals without inventing a pause length.
            intervals.push(interval.clone());
        }
    }
    if end > old_end {
        let a = start.max(old_end);
        push(&mut intervals, a, end, (end - a).max(0) as u64);
    }
    let duration = intervals
        .iter()
        .try_fold(0u64, |sum, i| sum.checked_add(i.duration_ms))
        .ok_or("invalid_duration")?;
    if duration == 0 || duration > 86_400_000 {
        return Err("invalid_duration");
    }
    let mut result = record.clone();
    result.started_at_ms = start;
    result.ended_at_ms = Some(end);
    result.duration_ms = duration;
    result.intervals = intervals;
    result.corrected = true;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record() -> Session {
        serde_json::from_value(serde_json::json!({"id":"one","kind":"focus","title":"Original","task_id":null,
            "started_at_ms":1000,"ended_at_ms":11000,"target_ms":60000,"focus_mode":"manual",
            "duration_ms":6000,"intervals":[{"started_at_ms":1000,"ended_at_ms":4000,"duration_ms":3000},
            {"started_at_ms":8000,"ended_at_ms":11000,"duration_ms":3000}],"corrected":false})).unwrap()
    }
    #[test]
    fn crops_and_extends_without_moving_the_pause_or_title() {
        let changed = correct_time_bounds(&record(), 2000, 12000).unwrap();
        assert_eq!(changed.duration_ms, 6000);
        assert_eq!(changed.title, "Original");
        assert_eq!(changed.intervals[0].ended_at_ms, Some(4000));
        assert_eq!(changed.intervals[1].started_at_ms, 8000);
        assert_eq!(
            correct_time_bounds(&record(), 0, 12000)
                .unwrap()
                .duration_ms,
            8000
        );
        assert_eq!(
            correct_time_bounds(&record(), 2000, 10000)
                .unwrap()
                .duration_ms,
            4000
        );
    }
    #[test]
    fn unchanged_bounds_preserve_exact_legacy_durations() {
        let mut original = record();
        original.intervals[0].duration_ms = 1234;
        original.duration_ms = 4234;
        assert_eq!(
            serde_json::to_value(correct_time_bounds(&original, 1000, 11000).unwrap()).unwrap(),
            serde_json::to_value(original).unwrap()
        );
    }
    #[test]
    fn rejects_empty_reversed_pause_only_and_excessive_ranges() {
        for (a, b) in [(2, 2), (10, 1), (-1, 100), (5000, 7000), (0, 100_000_000)] {
            assert!(correct_time_bounds(&record(), a, b).is_err());
        }
        assert!(!ranges_overlap(1000, 2000, 2000, 3000));
        assert!(ranges_overlap(1000, 2001, 2000, 3000));
        assert!(ranges_overlap(1000, 5000, 2000, 3000));
    }
}
