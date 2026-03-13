//! Renders test result trees with colors and Unicode box-drawing characters.

use std::io::Write;

use crossterm::style::{Color, ResetColor, SetForegroundColor};

use super::output::Summary;
use super::parser::TestOutcome;
use super::tree::TreeNode;

/// Rendering context that bundles writer and color settings.
struct RenderCtx<'a, W: Write> {
    writer: &'a mut W,
    use_color: bool,
}

/// Renders a list of root tree nodes to the given writer.
///
/// # Errors
///
/// Returns `std::io::Error` if writing fails.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::tree::TreeNode;
/// use behave::cli::render::render_tree;
///
/// let roots = vec![TreeNode::new_group("suite".to_string())];
/// let mut buf = Vec::new();
/// render_tree(&mut buf, &roots, false)?;
/// # }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn render_tree(
    writer: &mut impl Write,
    roots: &[TreeNode],
    use_color: bool,
) -> std::io::Result<()> {
    let mut ctx = RenderCtx { writer, use_color };
    for root in roots {
        render_node(&mut ctx, root, "", true)?;
    }
    Ok(())
}

fn render_node<W: Write>(
    ctx: &mut RenderCtx<'_, W>,
    node: &TreeNode,
    prefix: &str,
    is_last: bool,
) -> std::io::Result<()> {
    let connector = if is_last { "└─ " } else { "├─ " };
    let display_name = humanize_with_markers(node);

    write!(ctx.writer, "{prefix}{connector}")?;
    render_name_with_status(
        ctx.writer,
        &display_name,
        node.outcome.as_ref(),
        ctx.use_color,
    )?;
    writeln!(ctx.writer)?;

    let child_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });

    let count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        render_node(ctx, child, &child_prefix, i + 1 == count)?;
    }

    Ok(())
}

fn render_name_with_status(
    writer: &mut impl Write,
    name: &str,
    outcome: Option<&TestOutcome>,
    use_color: bool,
) -> std::io::Result<()> {
    let Some(outcome) = outcome else {
        return write!(writer, "{name}");
    };

    let (symbol, color) = match outcome {
        TestOutcome::Pass => ("✓", Color::Green),
        TestOutcome::Fail => ("✗", Color::Red),
        TestOutcome::Ignored => ("○", Color::Yellow),
        TestOutcome::Skipped => ("⊘", Color::Cyan),
    };

    if use_color {
        write!(
            writer,
            "{}{symbol} {name}{}",
            SetForegroundColor(color),
            ResetColor
        )
    } else {
        write!(writer, "{symbol} {name}")
    }
}

fn humanize(slug: &str) -> String {
    slug.replace('_', " ")
}

fn humanize_with_markers(node: &TreeNode) -> String {
    let mut name = humanize(&node.name);

    if node.pending {
        name = format!("[pending] {name}");
    }
    if node.focused {
        name = format!("[focus] {name}");
    }
    if !node.tags.is_empty() {
        let tag_list = node.tags.join(", ");
        name = format!("{name} [{tag_list}]");
    }

    name
}

/// Prints a summary line with pass/fail/ignored counts.
///
/// # Errors
///
/// Returns `std::io::Error` if writing fails.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::output::Summary;
/// use behave::cli::render::render_summary;
///
/// let mut buf = Vec::new();
/// let summary = Summary::new(5, 1, 2, 0, 8);
/// render_summary(&mut buf, &summary, false)?;
/// # }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn render_summary(
    writer: &mut impl Write,
    summary: &Summary,
    use_color: bool,
) -> std::io::Result<()> {
    writeln!(writer)?;

    if use_color {
        if summary.failed > 0 {
            write!(writer, "{}", SetForegroundColor(Color::Red))?;
        } else {
            write!(writer, "{}", SetForegroundColor(Color::Green))?;
        }
    }

    write!(writer, "{} passed", summary.passed)?;

    if summary.failed > 0 {
        write!(writer, ", {} failed", summary.failed)?;
    }
    if summary.skipped > 0 {
        write!(writer, ", {} skipped", summary.skipped)?;
    }
    if summary.ignored > 0 {
        write!(writer, ", {} ignored", summary.ignored)?;
    }

    if use_color {
        write!(writer, "{ResetColor}")?;
    }

    writeln!(writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::output::Summary;
    use crate::cli::parser::TestOutcome;
    use crate::cli::tree::TreeNode;

    #[test]
    fn humanize_replaces_underscores() {
        assert_eq!(humanize("hello_world"), "hello world");
    }

    #[test]
    fn humanize_no_underscores() {
        assert_eq!(humanize("hello"), "hello");
    }

    #[test]
    fn humanize_multiple_underscores() {
        assert_eq!(humanize("a_b_c_d"), "a b c d");
    }

    #[test]
    fn humanize_with_focus_marker() {
        let mut node = TreeNode::new_leaf("focused_test".to_string());
        node.focused = true;
        assert_eq!(humanize_with_markers(&node), "[focus] focused test");
    }

    #[test]
    fn humanize_with_pending_marker() {
        let mut node = TreeNode::new_leaf("todo_test".to_string());
        node.pending = true;
        assert_eq!(humanize_with_markers(&node), "[pending] todo test");
    }

    #[test]
    fn render_tree_simple() {
        let mut root = TreeNode::new_group("suite".to_string());
        let mut leaf = TreeNode::new_leaf("test".to_string());
        leaf.outcome = Some(TestOutcome::Pass);
        root.children.push(leaf);

        let mut buf = Vec::new();
        render_tree(&mut buf, &[root], false).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("suite"));
        assert!(output.contains("test"));
    }

    #[test]
    fn render_tree_marks_focus_and_pending() {
        let mut root = TreeNode::new_group("suite".to_string());

        let mut focused = TreeNode::new_leaf("important_case".to_string());
        focused.focused = true;
        focused.outcome = Some(TestOutcome::Pass);

        let mut pending = TreeNode::new_leaf("todo_case".to_string());
        pending.pending = true;
        pending.outcome = Some(TestOutcome::Ignored);

        root.children.push(focused);
        root.children.push(pending);

        let mut buf = Vec::new();
        render_tree(&mut buf, &[root], false).ok();
        let output = String::from_utf8(buf).unwrap_or_default();

        assert!(output.contains("[focus] important case"));
        assert!(output.contains("[pending] todo case"));
    }

    #[test]
    fn render_skipped_test_node() {
        let mut root = TreeNode::new_group("suite".to_string());
        let mut leaf = TreeNode::new_leaf("skipped_test".to_string());
        leaf.outcome = Some(TestOutcome::Skipped);
        root.children.push(leaf);

        let mut buf = Vec::new();
        render_tree(&mut buf, &[root], false).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("⊘"));
        assert!(output.contains("skipped test"));
    }

    #[test]
    fn render_summary_no_color() {
        let mut buf = Vec::new();
        render_summary(&mut buf, &Summary::new(5, 1, 2, 0, 8), false).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("5 passed"));
        assert!(output.contains("1 failed"));
        assert!(output.contains("2 ignored"));
    }

    #[test]
    fn render_summary_with_skipped() {
        let mut buf = Vec::new();
        render_summary(&mut buf, &Summary::new(3, 0, 0, 2, 5), false).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("3 passed"));
        assert!(output.contains("2 skipped"));
    }

    #[test]
    fn render_summary_no_failures() {
        let mut buf = Vec::new();
        render_summary(&mut buf, &Summary::new(3, 0, 0, 0, 3), false).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("3 passed"));
        assert!(!output.contains("failed"));
    }

    #[test]
    fn render_summary_with_color() {
        let mut buf = Vec::new();
        render_summary(&mut buf, &Summary::new(5, 0, 0, 0, 5), true).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("5 passed"));
    }

    #[test]
    fn humanize_with_single_tag() {
        let mut node = TreeNode::new_leaf("my_test".to_string());
        node.tags = vec!["slow".to_string()];
        assert_eq!(humanize_with_markers(&node), "my test [slow]");
    }

    #[test]
    fn humanize_with_multiple_tags() {
        let mut node = TreeNode::new_leaf("my_test".to_string());
        node.tags = vec!["slow".to_string(), "integration".to_string()];
        assert_eq!(humanize_with_markers(&node), "my test [slow, integration]");
    }

    #[test]
    fn humanize_with_focus_and_tags() {
        let mut node = TreeNode::new_leaf("my_test".to_string());
        node.focused = true;
        node.tags = vec!["critical".to_string()];
        assert_eq!(humanize_with_markers(&node), "[focus] my test [critical]");
    }

    #[test]
    fn humanize_with_pending_and_tags() {
        let mut node = TreeNode::new_leaf("todo_test".to_string());
        node.pending = true;
        node.tags = vec!["unit".to_string()];
        assert_eq!(humanize_with_markers(&node), "[pending] todo test [unit]");
    }

    #[test]
    fn render_all_four_outcome_types() {
        let mut root = TreeNode::new_group("suite".to_string());

        let mut pass = TreeNode::new_leaf("passing".to_string());
        pass.outcome = Some(TestOutcome::Pass);
        let mut fail = TreeNode::new_leaf("failing".to_string());
        fail.outcome = Some(TestOutcome::Fail);
        let mut ignored = TreeNode::new_leaf("ignored".to_string());
        ignored.outcome = Some(TestOutcome::Ignored);
        let mut skipped = TreeNode::new_leaf("skipped".to_string());
        skipped.outcome = Some(TestOutcome::Skipped);

        root.children.push(pass);
        root.children.push(fail);
        root.children.push(ignored);
        root.children.push(skipped);

        let mut buf = Vec::new();
        render_tree(&mut buf, &[root], false).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("✓"));
        assert!(output.contains("✗"));
        assert!(output.contains("○"));
        assert!(output.contains("⊘"));
    }

    #[test]
    fn render_summary_all_counts() {
        let mut buf = Vec::new();
        render_summary(&mut buf, &Summary::new(3, 2, 1, 4, 10), false).ok();
        let output = String::from_utf8(buf).unwrap_or_default();
        assert!(output.contains("3 passed"));
        assert!(output.contains("2 failed"));
        assert!(output.contains("1 ignored"));
        assert!(output.contains("4 skipped"));
    }
}
