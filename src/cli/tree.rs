//! Builds a hierarchical tree from flat test names.

use serde::Serialize;

use super::parser::{TestOutcome, TestResult};

/// A node in the test result tree.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::tree::TreeNode;
///
/// let node = TreeNode::new_leaf("my_test".to_string());
/// assert!(node.children.is_empty());
/// # }
/// ```
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct TreeNode {
    /// The display name of this node segment.
    pub name: String,
    /// Child nodes (empty for leaf tests).
    pub children: Vec<Self>,
    /// The test outcome, if this is a leaf node.
    pub outcome: Option<TestOutcome>,
    /// Whether this test was focused.
    pub focused: bool,
    /// Whether this test was pending.
    pub pending: bool,
    /// Tags applied to this node.
    pub tags: Vec<String>,
}

impl TreeNode {
    /// Creates a new group node.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "cli")]
    /// # {
    /// use behave::cli::tree::TreeNode;
    ///
    /// let node = TreeNode::new_group("suite".to_string());
    /// assert!(node.outcome.is_none());
    /// # }
    /// ```
    pub const fn new_group(name: String) -> Self {
        Self {
            name,
            children: Vec::new(),
            outcome: None,
            focused: false,
            pending: false,
            tags: Vec::new(),
        }
    }

    /// Creates a new leaf (test) node.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "cli")]
    /// # {
    /// use behave::cli::tree::TreeNode;
    ///
    /// let node = TreeNode::new_leaf("test".to_string());
    /// assert!(node.children.is_empty());
    /// # }
    /// ```
    pub const fn new_leaf(name: String) -> Self {
        Self {
            name,
            children: Vec::new(),
            outcome: None,
            focused: false,
            pending: false,
            tags: Vec::new(),
        }
    }
}

/// Builds a tree hierarchy from a flat list of test results.
///
/// Splits test names on `::` and groups them into a tree structure.
/// Detects `__FOCUS__` and `__PENDING__` prefixes in leaf names.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::tree::build_tree;
/// use behave::cli::parser::{TestResult, TestOutcome};
///
/// let results = vec![
///     TestResult::new("math::add".to_string(), TestOutcome::Pass),
/// ];
/// let roots = build_tree(&results);
/// assert_eq!(roots.len(), 1);
/// # }
/// ```
pub fn build_tree(results: &[TestResult]) -> Vec<TreeNode> {
    let mut roots: Vec<TreeNode> = Vec::new();

    for result in results {
        let segments: Vec<&str> = result.full_name.split("::").collect();
        insert_into_tree(&mut roots, &segments, &result.outcome);
    }

    sort_nodes(&mut roots);
    roots
}

fn insert_into_tree(nodes: &mut Vec<TreeNode>, segments: &[&str], outcome: &TestOutcome) {
    if segments.is_empty() {
        return;
    }

    let name = segments[0];
    let rest = &segments[1..];
    let (clean_name, focused, pending, tags) = detect_markers(name);

    let existing = nodes.iter_mut().find(|n| n.name == clean_name);

    let node = if let Some(node) = existing {
        node.focused |= focused;
        node.pending |= pending;
        merge_tags(&mut node.tags, &tags);
        node
    } else if rest.is_empty() {
        let mut leaf = create_leaf_node(clean_name, focused, pending);
        leaf.tags = tags;
        leaf.outcome = Some(outcome.clone());
        nodes.push(leaf);
        return;
    } else {
        let mut group = create_group_node(clean_name, focused, pending);
        group.tags = tags;
        nodes.push(group);
        // Safe: we just pushed
        let len = nodes.len();
        &mut nodes[len - 1]
    };

    if rest.is_empty() {
        node.outcome = Some(outcome.clone());
    } else {
        insert_into_tree(&mut node.children, rest, outcome);
    }
}

fn merge_tags(existing: &mut Vec<String>, new_tags: &[String]) {
    for tag in new_tags {
        if !existing.contains(tag) {
            existing.push(tag.clone());
        }
    }
}

const fn create_group_node(name: String, focused: bool, pending: bool) -> TreeNode {
    let mut node = TreeNode::new_group(name);
    node.focused = focused;
    node.pending = pending;
    node
}

const fn create_leaf_node(name: String, focused: bool, pending: bool) -> TreeNode {
    let mut node = TreeNode::new_leaf(name);
    node.focused = focused;
    node.pending = pending;
    node
}

fn detect_markers(name: &str) -> (String, bool, bool, Vec<String>) {
    let mut clean = name.to_string();
    let mut focused = false;
    let mut pending = false;
    let mut tags = Vec::new();

    if let Some(rest) = clean.strip_prefix("__FOCUS__") {
        focused = true;
        clean = rest.to_string();
    }
    if let Some(rest) = clean.strip_prefix("__PENDING__") {
        pending = true;
        clean = rest.to_string();
    }

    // Strip `__TAG_xxx__` prefixes in a loop
    while let Some(rest) = clean.strip_prefix("__TAG_") {
        if let Some(end_pos) = rest.find("__") {
            let tag = rest[..end_pos].to_string();
            tags.push(tag);
            clean = rest[end_pos + 2..].to_string();
        } else {
            break;
        }
    }

    (clean, focused, pending, tags)
}

fn sort_nodes(nodes: &mut [TreeNode]) {
    nodes.sort_by(|left, right| left.name.cmp(&right.name));

    for node in nodes {
        sort_nodes(&mut node.children);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_simple_tree() {
        let results = vec![
            TestResult::new("math::add".to_string(), TestOutcome::Pass),
            TestResult::new("math::sub".to_string(), TestOutcome::Pass),
        ];
        let tree = build_tree(&results);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 2);
    }

    #[test]
    fn detects_focus_marker() {
        let (name, focused, pending, tags) = detect_markers("__FOCUS__my_test");
        assert_eq!(name, "my_test");
        assert!(focused);
        assert!(!pending);
        assert!(tags.is_empty());
    }

    #[test]
    fn detects_pending_marker() {
        let (name, focused, pending, tags) = detect_markers("__PENDING__my_test");
        assert_eq!(name, "my_test");
        assert!(!focused);
        assert!(pending);
        assert!(tags.is_empty());
    }

    #[test]
    fn detects_tag_markers() {
        let (name, focused, _, tags) = detect_markers("__TAG_slow____TAG_integration__my_test");
        assert_eq!(name, "my_test");
        assert!(!focused);
        assert_eq!(tags, vec!["slow", "integration"]);
    }

    #[test]
    fn detects_focus_and_tags() {
        let (name, focused, _, tags) = detect_markers("__FOCUS____TAG_slow__my_test");
        assert_eq!(name, "my_test");
        assert!(focused);
        assert_eq!(tags, vec!["slow"]);
    }

    #[test]
    fn sorts_tree_nodes_stably() {
        let results = vec![
            TestResult::new("suite::zeta".to_string(), TestOutcome::Pass),
            TestResult::new("suite::alpha".to_string(), TestOutcome::Pass),
        ];
        let tree = build_tree(&results);

        assert_eq!(tree[0].children[0].name, "alpha");
        assert_eq!(tree[0].children[1].name, "zeta");
    }

    #[test]
    fn normalizes_focus_markers_on_groups() {
        let results = vec![TestResult::new(
            "__FOCUS__checkout::alpha".to_string(),
            TestOutcome::Pass,
        )];
        let tree = build_tree(&results);

        assert_eq!(tree[0].name, "checkout");
        assert!(tree[0].focused);
    }

    #[test]
    fn detects_no_markers() {
        let (name, focused, pending, tags) = detect_markers("plain_test");
        assert_eq!(name, "plain_test");
        assert!(!focused);
        assert!(!pending);
        assert!(tags.is_empty());
    }

    #[test]
    fn detects_pending_with_tags() {
        let (name, _, pending, tags) = detect_markers("__PENDING____TAG_slow__my_test");
        assert_eq!(name, "my_test");
        assert!(pending);
        assert_eq!(tags, vec!["slow"]);
    }

    #[test]
    fn detects_single_tag() {
        let (name, _, _, tags) = detect_markers("__TAG_unit__helper_test");
        assert_eq!(name, "helper_test");
        assert_eq!(tags, vec!["unit"]);
    }

    #[test]
    fn tags_propagate_to_groups_in_tree() {
        let results = vec![TestResult::new(
            "__TAG_slow__suite::test_a".to_string(),
            TestOutcome::Pass,
        )];
        let tree = build_tree(&results);
        assert_eq!(tree[0].name, "suite");
        assert!(tree[0].tags.contains(&"slow".to_string()));
    }

    #[test]
    fn multiple_tests_merge_tags_on_parent() {
        let results = vec![
            TestResult::new("__TAG_slow__suite::test_a".to_string(), TestOutcome::Pass),
            TestResult::new("__TAG_fast__suite::test_b".to_string(), TestOutcome::Pass),
        ];
        let tree = build_tree(&results);
        assert_eq!(tree[0].name, "suite");
        assert!(tree[0].tags.contains(&"slow".to_string()));
        assert!(tree[0].tags.contains(&"fast".to_string()));
    }

    #[test]
    fn tag_with_underscore_in_name() {
        let (name, _, _, tags) = detect_markers("__TAG_my_tag__test_fn");
        // Only strips up to first __ after TAG_, so "my_tag" is the tag
        assert_eq!(tags, vec!["my_tag"]);
        assert_eq!(name, "test_fn");
    }
}
