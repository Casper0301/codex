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
    assert!(instruction.ends_with("</persistent_session_summary>"));
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

#[test]
fn substantive_prompt_becomes_an_immediate_session_summary() {
    let mut context = ComposerContext::new();
    context.set_enabled(/*enabled*/ true);

    context.set_last_prompt("Make Codex reload itself and resume the current session.");

    assert_eq!(
        context.session_summary.as_deref(),
        Some("Make Codex reload itself and resume the current session.")
    );
    let rendered = context
        .render_lines(/*width*/ 100)
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!rendered.contains("Summary pending"));
}

#[test]
fn generic_control_prompt_preserves_the_durable_summary() {
    let mut context = ComposerContext::new();
    context.set_session_summary("Implement live Codex session context.");

    context.set_last_prompt("continue");

    assert_eq!(
        context.session_summary.as_deref(),
        Some("Implement live Codex session context.")
    );
}

#[test]
fn completed_answer_is_a_summary_fallback_when_the_marker_is_missing() {
    let mut context = ComposerContext::new();
    context.set_session_summary("Make session summaries update reliably.");

    context.set_assistant_summary_fallback(
        "## Done\n\n- Codex now replaces a pending summary after every completed answer.\n- Tests pass.",
    );

    assert_eq!(
        context.session_summary.as_deref(),
        Some("Done Codex now replaces a pending summary after every completed answer. Tests pass.")
    );
}
