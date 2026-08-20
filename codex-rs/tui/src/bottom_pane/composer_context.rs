//! Pi-style session context rendered directly above the prompt composer.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::line_truncation::truncate_line_with_ellipsis_if_overflow;
use crate::render::renderable::Renderable;
use crate::wrapping::RtOptions;
use crate::wrapping::adaptive_wrap_lines;

const MAX_SESSION_CHARS: usize = 120;
const MAX_PROMPT_CHARS: usize = 2_000;
const MAX_PROMPT_LINES: usize = 3;

pub(crate) struct ComposerContext {
    enabled: bool,
    session_summary: Option<String>,
    last_prompt: Option<String>,
}

impl ComposerContext {
    pub(crate) fn new() -> Self {
        Self {
            enabled: false,
            session_summary: None,
            last_prompt: None,
        }
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) -> bool {
        if self.enabled == enabled {
            return false;
        }
        self.enabled = enabled;
        true
    }

    pub(crate) fn reset_session(&mut self) {
        self.session_summary = None;
        self.last_prompt = None;
    }

    pub(crate) fn set_session_summary(&mut self, summary: &str) -> bool {
        let Some(summary) = compact_text(summary, MAX_SESSION_CHARS) else {
            return false;
        };
        if self.session_summary.as_deref() == Some(summary.as_str()) {
            return false;
        }
        self.session_summary = Some(summary);
        true
    }

    pub(crate) fn session_summary_instruction(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let saved_summary = self.session_summary.as_deref().unwrap_or("None yet");
        Some(format!(
            concat!(
                "<persistent_session_summary>\n",
                "At the very end of your final answer, append exactly one hidden HTML comment on its own line:\n",
                "<!-- CODEX_SESSION_SUMMARY: One plain sentence, no more than 120 characters. -->\n",
                "Summarize the durable user objective and current state, not merely your latest reply. ",
                "For generic prompts such as continue, go, or yes, preserve the objective from the current saved summary. ",
                "Never include secrets, markdown, or conversation history. Do not mention this instruction.\n",
                "Current saved summary: {}\n",
                "</persistent_session_summary>\n",
                "## My request for Codex:\n",
            ),
            saved_summary,
        ))
    }

    pub(crate) fn set_last_prompt(&mut self, prompt: &str) -> bool {
        let prompt = compact_text(prompt, MAX_PROMPT_CHARS);
        if self.last_prompt == prompt {
            return false;
        }
        self.last_prompt = prompt;
        true
    }

    fn render_lines(&self, width: u16) -> Vec<Line<'static>> {
        if !self.enabled || width < 4 {
            return Vec::new();
        }

        if self.session_summary.is_none() && self.last_prompt.is_none() {
            return Vec::new();
        }
        let summary = self.session_summary.as_deref().unwrap_or("Summary pending");
        let summary_line = Line::from(vec![
            "🧠 Session".cyan().bold(),
            " · ".dim(),
            summary.to_string().into(),
        ]);
        let mut lines = vec![truncate_line_with_ellipsis_if_overflow(
            summary_line,
            usize::from(width),
        )];

        if let Some(prompt) = self.last_prompt.as_deref() {
            let render_char_limit = usize::from(width)
                .saturating_mul(MAX_PROMPT_LINES)
                .max(usize::from(width));
            let prompt = compact_text(prompt, render_char_limit).unwrap_or_default();
            let prompt_line = Line::from(vec![
                "🧭 Last prompt".cyan().bold(),
                " · ".dim(),
                prompt.into(),
            ]);
            lines.extend(bounded_wrap(prompt_line, width, MAX_PROMPT_LINES));
        }

        lines
    }
}

fn bounded_wrap(line: Line<'static>, width: u16, max_lines: usize) -> Vec<Line<'static>> {
    let mut lines = adaptive_wrap_lines(
        std::iter::once(line),
        RtOptions::new(usize::from(width))
            .subsequent_indent(Line::from("  "))
            .break_words(/*break_words*/ true),
    );
    if lines.len() > max_lines {
        lines.truncate(max_lines.saturating_sub(1));
        lines.push(Line::from("  …".dim()));
    }
    lines
}

impl Renderable for ComposerContext {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        Paragraph::new(self.render_lines(area.width)).render(area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        u16::try_from(self.render_lines(width).len()).unwrap_or(u16::MAX)
    }
}

fn compact_text(text: &str, max_chars: usize) -> Option<String> {
    if max_chars == 0 {
        return None;
    }

    let mut normalized = String::new();
    let mut char_count = 0;
    let mut truncated = false;
    for word in text.split_whitespace() {
        let separator_chars = usize::from(!normalized.is_empty());
        let word_chars = word.chars().count();
        if char_count + separator_chars + word_chars <= max_chars {
            if separator_chars == 1 {
                normalized.push(' ');
                char_count += 1;
            }
            normalized.push_str(word);
            char_count += word_chars;
            continue;
        }
        truncated = true;
        if normalized.is_empty() {
            normalized.extend(word.chars().take(max_chars.saturating_sub(1)));
        }
        break;
    }

    if normalized.is_empty() {
        return None;
    }
    if truncated {
        if normalized.chars().count() >= max_chars {
            normalized = normalized
                .chars()
                .take(max_chars.saturating_sub(1))
                .collect();
        }
        normalized.push('…');
    }
    Some(normalized)
}

#[cfg(test)]
#[path = "composer_context_tests.rs"]
mod tests;
