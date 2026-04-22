use crate::app::CompareResultView;
use crate::diff::{DiffNode, DiffStatus};

pub fn find_node_by_path<'a>(node: &'a DiffNode, path: &str) -> Option<&'a DiffNode> {
    if node.path == path {
        return Some(node);
    }

    node.children
        .iter()
        .find_map(|child| find_node_by_path(child, path))
}

pub fn draw_detail(ui: &mut eframe::egui::Ui, result: Option<&CompareResultView>) {
    let Some(result) = result else {
        ui.label("Run compare to inspect node details.");
        return;
    };

    let Some(selected_path) = result.selected_path.as_deref() else {
        ui.label("Select a changed node to inspect its details.");
        return;
    };

    let Some(node) = find_node_by_path(&result.root, selected_path) else {
        ui.label("The selected node is no longer available in the current diff.");
        return;
    };

    ui.label(format!("Path: {}", node.path));
    ui.label(format!("Status: {}", status_label(&node.status)));
    ui.label(format!("Summary: {}", node.summary));
    if is_aggregate_node(node) {
        ui.label(format!(
            "Aggregate node with {} child change(s); inspect child entries for concrete values.",
            node.children.len()
        ));
    }
    ui.separator();
    ui.label("Left value");
    ui.monospace(detail_value_text(node, node.left_value.as_deref()));
    ui.separator();
    ui.label("Right value");
    ui.monospace(detail_value_text(node, node.right_value.as_deref()));
}

fn status_label(status: &DiffStatus) -> &'static str {
    match status {
        DiffStatus::Unchanged => "Unchanged",
        DiffStatus::Modified => "Modified",
        DiffStatus::Added => "Added",
        DiffStatus::Removed => "Removed",
        DiffStatus::Reordered => "Reordered",
        DiffStatus::Error => "Error",
    }
}

fn is_aggregate_node(node: &DiffNode) -> bool {
    node.left_value.is_none() && node.right_value.is_none()
}

fn detail_value_text(node: &DiffNode, value: Option<&str>) -> String {
    match value {
        Some(value) => value.to_string(),
        None if is_aggregate_node(node) && node.children.is_empty() => {
            "(no direct value snapshot recorded for this node)".to_string()
        }
        None if is_aggregate_node(node) => "(container node; no direct value snapshot)".to_string(),
        None => "(missing)".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::detail_value_text;
    use crate::diff::{DiffNode, DiffStatus};

    #[test]
    fn detail_value_text_marks_container_nodes_without_direct_snapshot() {
        let node = DiffNode {
            path: "Lines".into(),
            status: DiffStatus::Modified,
            left_value: None,
            right_value: None,
            summary: "Lines modified".into(),
            children: vec![DiffNode {
                path: "Lines[0]".into(),
                status: DiffStatus::Modified,
                left_value: Some("\"left\"".into()),
                right_value: Some("\"right\"".into()),
                summary: "child modified".into(),
                children: Vec::new(),
            }],
        };

        assert_eq!(
            detail_value_text(&node, node.left_value.as_deref()),
            "(container node; no direct value snapshot)"
        );
    }

    #[test]
    fn detail_value_text_keeps_missing_for_one_sided_leaf_changes() {
        let node = DiffNode {
            path: "LegacyCode".into(),
            status: DiffStatus::Removed,
            left_value: Some("\"A1\"".into()),
            right_value: None,
            summary: "LegacyCode removed".into(),
            children: Vec::new(),
        };

        assert_eq!(
            detail_value_text(&node, node.right_value.as_deref()),
            "(missing)"
        );
    }
}
