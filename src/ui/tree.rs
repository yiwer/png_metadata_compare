use crate::app::CompareResultView;
use crate::diff::{DiffNode, DiffStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeFilters {
    pub only_differences: bool,
    pub show_reordered: bool,
    pub show_unchanged: bool,
    pub show_errors: bool,
}

impl Default for TreeFilters {
    fn default() -> Self {
        Self {
            only_differences: true,
            show_reordered: true,
            show_unchanged: false,
            show_errors: true,
        }
    }
}

pub fn should_show(node: &DiffNode, filters: &TreeFilters) -> bool {
    node_matches_filters(node, filters)
        || node
            .children
            .iter()
            .any(|child| should_show(child, filters))
}

pub fn draw_tree(ui: &mut eframe::egui::Ui, result: &mut CompareResultView, filters: &TreeFilters) {
    if !should_show(&result.root, filters) {
        ui.label("No diff nodes match the current filters.");
        return;
    }

    eframe::egui::ScrollArea::vertical().show(ui, |ui| {
        draw_node(ui, &result.root, &mut result.selected_path, filters);
    });
}

fn draw_node(
    ui: &mut eframe::egui::Ui,
    node: &DiffNode,
    selected_path: &mut Option<String>,
    filters: &TreeFilters,
) {
    if !should_show(node, filters) {
        return;
    }

    let selected = selected_path.as_deref() == Some(node.path.as_str());
    let label = node_label(node, selected);
    let has_visible_children = node
        .children
        .iter()
        .any(|child| should_show(child, filters));

    if has_visible_children {
        let response = eframe::egui::CollapsingHeader::new(label)
            .id_salt(&node.path)
            .default_open(default_open(node))
            .show(ui, |ui| {
                for child in &node.children {
                    draw_node(ui, child, selected_path, filters);
                }
            });

        if response.header_response.clicked() {
            *selected_path = Some(node.path.clone());
        }
    } else if ui.selectable_label(selected, label).clicked() {
        *selected_path = Some(node.path.clone());
    }
}

fn node_matches_filters(node: &DiffNode, filters: &TreeFilters) -> bool {
    match node.status {
        DiffStatus::Unchanged => !filters.only_differences && filters.show_unchanged,
        DiffStatus::Modified | DiffStatus::Added | DiffStatus::Removed => true,
        DiffStatus::Reordered => filters.show_reordered,
        DiffStatus::Error => filters.show_errors,
    }
}

fn node_label(node: &DiffNode, selected: bool) -> String {
    let prefix = if selected { "> " } else { "" };
    let path = if node.path.is_empty() {
        "(root)"
    } else {
        node.path.as_str()
    };

    format!(
        "{prefix}{path} [{}] - {}",
        status_label(&node.status),
        node.summary
    )
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

fn default_open(node: &DiffNode) -> bool {
    node.path.is_empty() || (!node.path.contains('.') && !node.path.contains('['))
}

#[cfg(test)]
pub(crate) fn hides_unchanged_nodes_when_only_differences_is_enabled_test() {
    tests::hides_unchanged_nodes_when_only_differences_is_enabled();
}

#[cfg(test)]
mod tests {
    use super::{TreeFilters, should_show};
    use crate::diff::{DiffNode, DiffStatus};

    #[test]
    pub(super) fn hides_unchanged_nodes_when_only_differences_is_enabled() {
        let filters = TreeFilters {
            only_differences: true,
            show_unchanged: true,
            ..Default::default()
        };
        let unchanged = DiffNode {
            path: "Title".into(),
            status: DiffStatus::Unchanged,
            left_value: Some("\"same\"".into()),
            right_value: Some("\"same\"".into()),
            summary: "Title unchanged".into(),
            children: Vec::new(),
        };
        let modified = DiffNode {
            path: "LegacyCode".into(),
            status: DiffStatus::Modified,
            left_value: Some("\"A1\"".into()),
            right_value: Some("\"B2\"".into()),
            summary: "LegacyCode modified".into(),
            children: Vec::new(),
        };

        assert!(!should_show(&unchanged, &filters));
        assert!(should_show(&modified, &filters));
    }
}
