use super::*;
use insta::assert_snapshot;
use pretty_assertions::assert_eq;

#[test]
fn context_is_hidden_when_status_line_is_disabled() {
    let mut context = ComposerContext::new();
    context.reset_session();
    context.set_session_summary("Hidden session");
    context.set_last_prompt("Hidden prompt");

    assert_eq!(context.desired_height(/*width*/ 80), 0);
}

#[test]
fn empty_context_is_hidden_until_a_summary_or_prompt_exists() {
    let mut context = ComposerContext::new();
    context.set_enabled(/*enabled*/ true);

    assert_eq!(context.desired_height(/*width*/ 80), 0);
}

#[test]
fn context_wraps_and_caps_long_prompts() {
    let mut context = ComposerContext::new();
    context.set_enabled(/*enabled*/ true);
    context.reset_session();
    context.set_session_summary(
        "Keeps the AI-written session summary separate from the thread title.",
    );
    context.set_last_prompt(
        "Add the Pi-style session and last prompt above the composer while preserving the footer and keeping very long prompts bounded.",
    );

    let width = 36;
    let height = context.desired_height(width);
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    context.render(Rect::new(0, 0, width, height), &mut buf);

    assert_snapshot!("composer_context_wrapped", format!("{buf:?}"));
}

#[test]
fn ai_summary_wins_over_thread_title_and_prompt() {
    let mut context = ComposerContext::new();
    context.set_enabled(/*enabled*/ true);
    context.reset_session();
    context.set_last_prompt("continue");
    context.set_session_summary(
        "Restore the durable Herdr appearance and verify the live installation.",
    );

    let lines = context.render_lines(/*width*/ 100);
    let rendered = lines
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(rendered.contains(
        "🧠 Session · Restore the durable Herdr appearance and verify the live installation."
    ));
    assert!(!rendered.contains("Herdr / Pi"));
}

#[test]
fn summary_instruction_asks_the_active_model_for_a_hidden_durable_summary() {
    let mut context = ComposerContext::new();
    context.set_enabled(/*enabled*/ true);
    context.set_session_summary("Keep the durable Herdr and Codex context visible after updates.");

    let instruction = context
        .session_summary_instruction()
        .expect("enabled context instruction");
    assert!(instruction.contains("CODEX_SESSION_SUMMARY"));
    assert!(instruction.contains("Current saved summary"));
    assert!(
        instruction.contains("Keep the durable Herdr and Codex context visible after updates.")
    );
    assert!(instruction.ends_with("## My request for Codex:\n"));
}

#[test]
fn empty_summary_does_not_replace_existing_summary() {
    let mut context = ComposerContext::new();
    context.set_session_summary("Keep the durable objective visible.");

    assert!(!context.set_session_summary("   "));
    assert_eq!(
        context.session_summary.as_deref(),
        Some("Keep the durable objective visible.")
    );
}
