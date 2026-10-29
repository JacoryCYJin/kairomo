use serde_json::Value;
/// Two appearances, each with rest background/ink and overtime background/ink.
/// Native drawing receives bytes and never knows the name or geometry of a theme.
pub fn menu_palette(theme: &Value) -> [u8; 24] {
    let mut result = [0; 24];
    for (index, mode) in ["light", "dark"].iter().enumerate() {
        for (color, value) in [
            &theme["rest"][mode]["background"],
            &theme["rest"][mode]["ink"],
            &theme["menu"]["overtime"][mode]["background"],
            &theme["menu"]["overtime"][mode]["ink"],
        ]
        .iter()
        .enumerate()
        {
            let hex = value.as_str().expect("theme color");
            assert!(
                hex.len() == 7 && hex.starts_with('#') && hex.is_ascii(),
                "invalid theme color"
            );
            for channel in 0..3 {
                result[index * 12 + color * 3 + channel] =
                    u8::from_str_radix(&hex[1 + channel * 2..3 + channel * 2], 16)
                        .expect("theme color channel");
            }
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_colors_match_shared_theme_and_accept_non_coffee_data() {
        let theme = timer_core::themes::theme("coffee");
        let palette = menu_palette(theme);
        assert_eq!(&palette[..6], &[191, 141, 99, 61, 37, 24]);
        assert_eq!(&palette[12..18], &[73, 51, 41, 231, 199, 166]);
        let mut fixture = theme.clone();
        fixture["rest"]["light"]["background"] = "#123456".into();
        assert_eq!(&menu_palette(&fixture)[..3], &[18, 52, 86]);
    }
}
