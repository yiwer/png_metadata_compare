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

pub(crate) fn reconcile_selected_path(result: &mut CompareResultView, filters: &TreeFilters) {
    let selection_is_visible = result
        .selected_path
        .as_deref()
        .is_some_and(|path| selected_path_is_visible(&result.root, path, filters));

    if selection_is_visible {
        return;
    }

    result.selected_path = first_visible_selection_path(&result.root, filters);
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
            .default_open(should_default_open(node, filters))
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

fn should_default_open(node: &DiffNode, filters: &TreeFilters) -> bool {
    node.children
        .iter()
        .any(|child| contains_visible_difference(child, filters))
}

fn contains_visible_difference(node: &DiffNode, filters: &TreeFilters) -> bool {
    directly_visible_difference(node, filters)
        || node
            .children
            .iter()
            .any(|child| contains_visible_difference(child, filters))
}

fn directly_visible_difference(node: &DiffNode, filters: &TreeFilters) -> bool {
    node.status != DiffStatus::Unchanged && node_matches_filters(node, filters)
}

fn selected_path_is_visible(node: &DiffNode, path: &str, filters: &TreeFilters) -> bool {
    if node.path == path {
        return should_show(node, filters);
    }

    node.children
        .iter()
        .any(|child| selected_path_is_visible(child, path, filters))
}

fn first_visible_selection_path(node: &DiffNode, filters: &TreeFilters) -> Option<String> {
    first_visible_snapshot_path(node, filters)
        .or_else(|| first_directly_visible_path(node, filters))
}

fn first_visible_snapshot_path(node: &DiffNode, filters: &TreeFilters) -> Option<String> {
    if node_matches_filters(node, filters)
        && (node.left_value.is_some() || node.right_value.is_some())
    {
        return Some(node.path.clone());
    }

    for child in &node.children {
        if let Some(path) = first_visible_snapshot_path(child, filters) {
            return Some(path);
        }
    }

    None
}

fn first_directly_visible_path(node: &DiffNode, filters: &TreeFilters) -> Option<String> {
    if node_matches_filters(node, filters) {
        return Some(node.path.clone());
    }

    for child in &node.children {
        if let Some(path) = first_directly_visible_path(child, filters) {
            return Some(path);
        }
    }

    None
}

#[cfg(test)]
pub(crate) fn hides_unchanged_nodes_when_only_differences_is_enabled_test() {
    tests::hides_unchanged_nodes_when_only_differences_is_enabled();
}

#[cfg(test)]
mod tests {
    use super::{TreeFilters, reconcile_selected_path, should_default_open, should_show};
    use crate::app::CompareResultView;
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

    #[test]
    fn opens_nested_branches_when_they_contain_visible_differences() {
        let filters = TreeFilters::default();
        let branch = DiffNode {
            path: "Lines[LineName=M375]".into(),
            status: DiffStatus::Modified,
            left_value: None,
            right_value: None,
            summary: "Line modified".into(),
            children: vec![DiffNode {
                path: "Lines[LineName=M375].PriceDescription".into(),
                status: DiffStatus::Modified,
                left_value: Some("\"2\"".into()),
                right_value: Some("\"3\"".into()),
                summary: "PriceDescription modified".into(),
                children: Vec::new(),
            }],
        };

        assert!(should_default_open(&branch, &filters));
    }

    #[test]
    fn moves_selection_to_visible_aggregate_when_leaf_selection_is_hidden() {
        let filters = TreeFilters {
            only_differences: false,
            show_reordered: false,
            show_unchanged: false,
            show_errors: false,
        };
        let root = DiffNode {
            path: "StopPlateMetadata".into(),
            status: DiffStatus::Modified,
            left_value: None,
            right_value: None,
            summary: "root modified".into(),
            children: vec![DiffNode {
                path: "Lines[0]".into(),
                status: DiffStatus::Reordered,
                left_value: Some("0".into()),
                right_value: Some("1".into()),
                summary: "row moved".into(),
                children: Vec::new(),
            }],
        };
        let mut result = CompareResultView {
            root,
            change_list: Vec::new(),
            summary: Default::default(),
            selected_path: Some("Lines[0]".into()),
        };

        reconcile_selected_path(&mut result, &filters);

        assert_eq!(result.selected_path.as_deref(), Some("StopPlateMetadata"));
    }
}
