//! Built-in theme identities come from the same manifest as the visual adapters.
use crate::{Settings, TimerError};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;
pub fn catalog() -> &'static Value {
    static CATALOG: OnceLock<Value> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../../../theme-kits/catalog.json"))
            .expect("invalid built-in theme catalog")
    })
}
pub fn registered(id: &str) -> bool {
    catalog()["themes"].get(id).is_some()
}
pub fn theme(id: &str) -> &'static Value {
    let data = catalog();
    data["themes"]
        .get(id)
        .unwrap_or_else(|| &data["themes"][data["default"].as_str().expect("default theme")])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_themes_and_fallback_are_shared_with_the_frontend() {
        let default = catalog()["default"].as_str().unwrap();
        assert!(registered(default));
        assert!(!registered("uninstalled-fixture"));
        assert_eq!(theme("uninstalled-fixture"), theme(default));
        for (_, theme) in catalog()["themes"].as_object().unwrap() {
            for language in ["zh-CN", "zh-TW", "en-US", "ja-JP"] {
                assert!(theme["name"][language]
                    .as_str()
                    .is_some_and(|name| !name.is_empty()));
            }
        }
    }
}

/// Schema is packaged with the theme; user input can only alter declared values.
fn valid_value(spec: &Value, value: &Value) -> bool {
    match spec["type"].as_str() {
        Some("enum") => spec["values"]
            .as_array()
            .is_some_and(|values| values.contains(value)),
        Some("boolean") => value.is_boolean(),
        Some("number") => value.as_f64().is_some_and(|v| {
            v.is_finite()
                && spec["min"].as_f64().is_some_and(|min| v >= min)
                && spec["max"].as_f64().is_some_and(|max| v <= max)
        }),
        _ => false,
    }
}
pub fn validate_saved_options(
    options: &BTreeMap<String, Map<String, Value>>,
) -> Result<(), TimerError> {
    for (id, values) in options {
        if !registered(id) || values.len() > 64 {
            return Err(TimerError::InvalidThemeOptions);
        }
        let specs = &theme(id)["options"];
        for (key, value) in values {
            let spec = &specs[key];
            if spec.get("legacyField").is_some() || !valid_value(spec, value) {
                return Err(TimerError::InvalidThemeOptions);
            }
        }
    }
    Ok(())
}
pub fn apply_options(
    settings: &Settings,
    id: &str,
    values: Map<String, Value>,
) -> Result<Settings, TimerError> {
    if !registered(id) || id != settings.visual_series || values.len() > 64 {
        return Err(TimerError::InvalidThemeOptions);
    }
    let mut next = settings.clone();
    for (key, value) in values {
        let spec = &theme(id)["options"][&key];
        if !valid_value(spec, &value) {
            return Err(TimerError::InvalidThemeOptions);
        }
        if let Some(field) = spec["legacyField"].as_str() {
            // Existing beta.3 readers must keep seeing the same single authoritative value.
            if field != "bean_range" {
                return Err(TimerError::InvalidThemeOptions);
            }
            next.bean_range =
                serde_json::from_value(value).map_err(|_| TimerError::InvalidThemeOptions)?;
        } else {
            next.theme_options
                .entry(id.to_owned())
                .or_default()
                .insert(key, value);
        }
    }
    next.validate()?;
    Ok(next)
}
#[cfg(test)]
mod options_tests {
    use super::*;
    #[test]
    fn legacy_range_is_single_source_and_invalid_or_stale_changes_do_not_mutate_state() {
        let settings = Settings::default();
        let patch = |value| serde_json::from_value(serde_json::json!({"range":value})).unwrap();
        let next = apply_options(&settings, "coffee", patch("week")).unwrap();
        assert_eq!(next.bean_range, crate::BeanRange::Week);
        assert!(next.theme_options.is_empty());
        assert_eq!(settings.bean_range, crate::BeanRange::Day);
        assert!(apply_options(&settings, "coffee", patch("month")).is_err());
        assert!(apply_options(&settings, "not-installed", patch("day")).is_err());
        assert!(apply_options(
            &settings,
            "coffee",
            serde_json::from_value(serde_json::json!({"unknown":true})).unwrap()
        )
        .is_err());
        let saved = serde_json::to_string(&next).unwrap();
        let restored: Settings = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored.bean_range, crate::BeanRange::Week);
        restored.validate().unwrap();
    }
    #[test]
    fn schemas_support_non_coffee_values_and_enforce_bounds() {
        assert!(valid_value(
            &serde_json::json!({"type":"boolean"}),
            &true.into()
        ));
        assert!(!valid_value(
            &serde_json::json!({"type":"boolean"}),
            &"true".into()
        ));
        let number = serde_json::json!({"type":"number","min":0,"max":1});
        assert!(valid_value(&number, &serde_json::json!(0.5)));
        assert!(!valid_value(&number, &serde_json::json!(2)));
    }
}
