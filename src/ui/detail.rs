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
    ui.separator();
    ui.label("Left value");
    ui.monospace(node.left_value.as_deref().unwrap_or("(missing)"));
    ui.separator();
    ui.label("Right value");
    ui.monospace(node.right_value.as_deref().unwrap_or("(missing)"));
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
