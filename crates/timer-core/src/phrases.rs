use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PhraseMode {
    #[default]
    Fixed,
    Random,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Book,
    Poem,
    Song,
    Other,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Phrase {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub source_title: String,
    #[serde(default)]
    pub source_kind: Option<SourceKind>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Phrases {
    pub enabled: bool,
    pub mode: PhraseMode,
    pub entries: Vec<Phrase>,
    pub selected_id: Option<String>,
}
impl Phrases {
    pub fn valid(&self) -> bool {
        let mut ids = std::collections::HashSet::new();
        self.entries.iter().all(|entry| {
            !entry.id.is_empty() && !entry.text.trim().is_empty() && ids.insert(&entry.id)
        }) && self
            .selected_id
            .as_ref()
            .is_none_or(|id| self.entries.iter().any(|p| &p.id == id))
    }
    pub fn pick(&self, seed: &str, previous: Option<&str>) -> Option<String> {
        let candidates: Vec<_> = self
            .entries
            .iter()
            .filter(|p| self.entries.len() == 1 || Some(p.id.as_str()) != previous)
            .collect();
        let hash = seed.bytes().fold(2166136261_u64, |value, byte| {
            value.wrapping_mul(16777619) ^ u64::from(byte)
        });
        candidates
            .get((hash % candidates.len().max(1) as u64) as usize)
            .map(|p| p.id.clone())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhraseSelection {
    pub session_id: String,
    pub entry_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_phrase_loads_without_attribution_and_new_fields_round_trip() {
        let mut phrase: Phrase =
            serde_json::from_str(r#"{"id":"old","text":"keep this"}"#).unwrap();
        assert_eq!(phrase.author, "");
        assert_eq!(phrase.source_title, "");
        assert_eq!(phrase.source_kind, None);
        phrase.author = "王维".into();
        phrase.source_title = "山居秋暝".into();
        phrase.source_kind = Some(SourceKind::Poem);
        let encoded = serde_json::to_string(&phrase).unwrap();
        assert_eq!(serde_json::from_str::<Phrase>(&encoded).unwrap(), phrase);
    }
    #[test]
    fn selection_handles_empty_single_and_avoids_repeating() {
        let mut settings = Phrases::default();
        assert_eq!(settings.pick("seed", None), None);
        settings.entries.push(Phrase {
            id: "a".into(),
            text: "one".into(),
            ..Default::default()
        });
        assert_eq!(settings.pick("seed", Some("a")), Some("a".into()));
        settings.entries.push(Phrase {
            id: "b".into(),
            text: "two".into(),
            ..Default::default()
        });
        assert_eq!(settings.pick("seed", Some("a")), Some("b".into()));
        settings.entries.push(settings.entries[0].clone());
        assert!(!settings.valid());
    }
}
