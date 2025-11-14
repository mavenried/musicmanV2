use reedline::{Prompt, PromptEditMode, PromptHistorySearch};
use std::borrow::Cow;

pub struct MusicmanPrompt {
    pub left: String,
}

impl Prompt for MusicmanPrompt {
    fn render_prompt_left(&self) -> Cow<'static, str> {
        Cow::Owned(self.left.clone())
    }

    fn render_prompt_right(&self) -> Cow<'static, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _mode: PromptEditMode) -> Cow<'static, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'static, str> {
        Cow::Borrowed("> ")
    }

    fn get_prompt_color(&self) -> reedline::Color {
        reedline::Color::Green
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Borrowed("?")
    }
}
