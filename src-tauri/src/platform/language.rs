//! App-owned system text follows the persisted interface language.
pub(super) struct Text {
    pub open: &'static str,
    pub pause_resume: &'static str,
    pub quit: &'static str,
    pub focus: &'static str,
    pub rest: &'static str,
    pub focus_done: &'static str,
    pub focus_target: &'static str,
    pub break_done: &'static str,
}
pub(super) fn text(locale: &str) -> Text {
    match locale {
        "ja-JP" => Text {
            open: "{app}を開く",
            pause_resume: "一時停止／再開",
            quit: "{app}を終了",
            focus: "集中",
            rest: "休憩",
            focus_done: "今回の集中が終了しました。準備ができたら休憩を開始してください。",
            focus_target: "集中の目標時間に達しました。引き続き計測しています。準備ができたら休憩してください。",
            break_done: "休憩が終了しました。準備ができたら次の集中を開始してください。",
        },
        "en-US" => Text {
            open: "Open {app}",
            pause_resume: "Pause / Resume",
            quit: "Quit {app}",
            focus: "Focus",
            rest: "Break",
            focus_done: "Focus session complete. Start your break when you are ready.",
            focus_target: "Focus target reached. The timer is still running. Take a break when you are ready.",
            break_done: "Break complete. Start your next focus session when you are ready.",
        },
        _ => Text {
            open: "打开 {app}",
            pause_resume: "暂停 / 继续",
            quit: "退出 {app}",
            focus: "专注",
            rest: "休息",
            focus_done: "本轮专注已完成。准备好后点击开始休息。",
            focus_target: "已达到专注目标。继续计时，准备好后再休息。",
            break_done: "休息已结束。准备好后点击开始专注。",
        },
    }
}
#[cfg(test)]
mod tests {
    use super::text;
    #[test]
    fn system_text_covers_each_supported_language_and_stage() {
        for (locale, focus, rest, open) in [
            ("zh-CN", "专注", "休息", "打开 Kairomo"),
            ("en-US", "Focus", "Break", "Open Kairomo"),
            ("ja-JP", "集中", "休憩", "Kairomoを開く"),
        ] {
            let copy = text(locale);
            assert_eq!(copy.focus, focus);
            assert_eq!(copy.rest, rest);
            assert_eq!(copy.open.replace("{app}", "Kairomo"), open);
            for message in [
                copy.focus_done,
                copy.focus_target,
                copy.break_done,
                copy.pause_resume,
                copy.quit,
            ] {
                assert!(!message.is_empty());
            }
        }
        assert_ne!(text("ja-JP").focus_done, text("ja-JP").focus_target);
    }
}
