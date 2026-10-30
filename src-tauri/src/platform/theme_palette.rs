use serde_json::Value;

/// Each appearance contains focus, rest and overtime background/ink pairs.
/// An absent focus palette keeps the native label color and transparent fill.
pub struct MenuPalette {
    pub colors: [u8; 36],
    pub focus_background: bool,
}
pub fn menu_palette(theme: &Value) -> MenuPalette {
    let mut result = MenuPalette {
        colors: [0; 36],
        focus_background: theme["menu"]["focus"].is_object(),
    };
    for (appearance, mode) in ["light", "dark"].iter().enumerate() {
        for (state, name) in ["focus", "rest", "overtime"].iter().enumerate() {
            let palette = if *name == "rest" && theme["menu"][name].is_null() {
                &theme["rest"][mode]
            } else {
                &theme["menu"][name][mode]
            };
            if *name == "focus" && !result.focus_background {
                continue;
            }
            for (color, field) in ["background", "ink"].iter().enumerate() {
                let hex = palette[field].as_str().expect("theme color");
                assert!(
                    hex.len() == 7 && hex.starts_with('#') && hex.is_ascii(),
                    "invalid theme color"
                );
                for channel in 0..3 {
                    result.colors[appearance * 18 + state * 6 + color * 3 + channel] =
                        u8::from_str_radix(&hex[1 + channel * 2..3 + channel * 2], 16)
                            .expect("theme color channel");
                }
            }
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coffee_retains_native_focus_and_popup_rest_fallback() {
        let theme = timer_core::themes::theme("coffee");
        let palette = menu_palette(theme);
        assert!(!palette.focus_background);
        assert_eq!(&palette.colors[6..12], &[191, 141, 99, 61, 37, 24]);
        assert_eq!(&palette.colors[24..30], &[73, 51, 41, 231, 199, 166]);
        let mut fixture = theme.clone();
        fixture["rest"]["light"]["background"] = "#123456".into();
        assert_eq!(&menu_palette(&fixture).colors[6..9], &[18, 52, 86]);
    }
    #[test]
    fn snow_menu_states_are_independent_from_popup_colors() {
        let theme = timer_core::themes::theme("snow-mountain");
        let palette = menu_palette(theme);
        assert!(palette.focus_background);
        assert_eq!(
            palette.colors,
            [
                229, 237, 245, 52, 79, 105, 23, 37, 54, 207, 220, 235, 243, 245, 247, 98, 109, 121,
                52, 73, 95, 212, 224, 238, 37, 42, 48, 199, 208, 218, 23, 37, 54, 207, 220, 235,
            ]
        );
        assert_eq!(theme["rest"]["light"]["background"], "#FFFFFF");
        assert_eq!(theme["rest"]["dark"]["background"], "#212121");
    }
}
