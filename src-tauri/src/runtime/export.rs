use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use timer_core::{Kind, Session};
fn cell(value: &str) -> String {
    let trimmed = value.trim_start();
    let safe = if trimmed.starts_with(['=', '+', '-', '@']) || value.starts_with(['\t', '\r', '\n'])
    {
        format!("'{value}")
    } else {
        value.into()
    };
    format!("\"{}\"", safe.replace('"', "\"\""))
}
pub fn write(path: &str, records: &[Session]) -> Result<(), String> {
    let destination = Path::new(path);
    if destination.extension().and_then(|e| e.to_str()) != Some("csv") {
        return Err("csv_extension_required".into());
    }
    let parent = destination.parent().ok_or("invalid_export_path")?;
    let temporary = parent.join(format!(".kairomo-export-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        file.write_all(
            "\u{feff}id,type,title,start_utc,end_utc,duration_seconds,target_seconds,corrected\r\n"
                .as_bytes(),
        )
        .map_err(|e| e.to_string())?;
        for s in records
            .iter()
            .filter(|s| s.kind == Kind::Focus && s.ended_at_ms.is_some())
        {
            let stamp = |ms: i64| {
                chrono::DateTime::from_timestamp_millis(ms)
                    .map(|d| d.to_rfc3339())
                    .unwrap_or_default()
            };
            let line = format!(
                "{},{},{},{},{},{},{},{}\r\n",
                cell(&s.id),
                "focus",
                cell(&s.title),
                cell(&stamp(s.started_at_ms)),
                cell(&stamp(s.ended_at_ms.unwrap_or(s.started_at_ms))),
                s.duration_ms as f64 / 1000.0,
                s.target_ms / 1000,
                s.corrected
            );
            file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
        }
        file.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&temporary, destination).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spreadsheet_titles_cannot_be_formulas() {
        assert_eq!(cell("  =SUM(1,2)"), "\"'  =SUM(1,2)\"");
        assert_eq!(cell("a,\"b\"\n"), "\"a,\"\"b\"\"\n\"");
    }
    #[test]
    fn csv_excludes_legacy_rest_and_incomplete_sessions() {
        let path =
            std::env::temp_dir().join(format!("kairomo-export-{}.csv", uuid::Uuid::new_v4()));
        let mut engine = timer_core::Engine::default();
        engine.start("focus".into(), "Read".into(), 1000).unwrap();
        let focus = engine.switch("rest".into(), 2000).unwrap();
        let ongoing = engine.state.current.as_ref().unwrap().clone();
        engine.resume(2500).unwrap();
        let rest = engine.finish(3000).unwrap();
        write(path.to_str().unwrap(), &[focus, rest, ongoing]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(content.lines().count(), 2);
        assert!(content.contains(",focus,"));
        assert!(!content.contains(",break,"));
    }
}
