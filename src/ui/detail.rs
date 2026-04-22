use crate::app::{BatchSelection, CompareResultView};
use crate::batch_report::{BatchCompareReport, MatchStrategy, UnmatchedFile};
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
    if let Some(context) = detail_context_text(node) {
        ui.label(context);
    }
    ui.separator();
    ui.label("Left value");
    ui.monospace(detail_value_text(node, node.left_value.as_deref()));
    ui.separator();
    ui.label("Right value");
    ui.monospace(detail_value_text(node, node.right_value.as_deref()));
}

pub fn draw_batch_detail(
    ui: &mut eframe::egui::Ui,
    report: Option<&BatchCompareReport>,
    selection: Option<BatchSelection>,
) {
    let Some(report) = report else {
        ui.label("Run directory compare to inspect batch details.");
        return;
    };

    let Some(selection) = selection else {
        ui.label("Select a batch item to inspect details.");
        return;
    };

    for line in batch_detail_lines(report, Some(selection)) {
        ui.label(line);
    }
}

pub(crate) fn batch_detail_lines(
    report: &BatchCompareReport,
    selection: Option<BatchSelection>,
) -> Vec<String> {
    let Some(selection) = selection else {
        return vec!["Select a batch item to inspect details.".to_string()];
    };

    match selection {
        BatchSelection::Identical(index) => {
            let Some(identical) = report.identical.get(index) else {
                return vec!["The selected batch item is no longer available.".to_string()];
            };
            vec![
                format!("File: {}", identical.pair.file_name),
                "Status: identical".to_string(),
                format!(
                    "Match strategy: {}",
                    match_strategy_label(&identical.pair.match_strategy)
                ),
                format!(
                    "Left: {}",
                    identical.pair.left.relative_path.to_string_lossy()
                ),
                format!(
                    "Right: {}",
                    identical.pair.right.relative_path.to_string_lossy()
                ),
            ]
        }
        BatchSelection::Different(index) => {
            let Some(different) = report.different.get(index) else {
                return vec!["The selected batch item is no longer available.".to_string()];
            };
            vec![
                format!("File: {}", different.pair.file_name),
                "Status: different".to_string(),
                format!("Total changes: {}", different.summary.total()),
                format!("Modified: {}", different.summary.modified),
                format!("Added: {}", different.summary.added),
                format!("Removed: {}", different.summary.removed),
                format!("Reordered: {}", different.summary.reordered),
                format!("Errors: {}", different.summary.error),
            ]
        }
        BatchSelection::LeftOnly(index) => {
            let Some(unmatched) = report.left_only.get(index) else {
                return vec!["The selected batch item is no longer available.".to_string()];
            };
            unmatched_detail_lines(unmatched, "left only")
        }
        BatchSelection::RightOnly(index) => {
            let Some(unmatched) = report.right_only.get(index) else {
                return vec!["The selected batch item is no longer available.".to_string()];
            };
            unmatched_detail_lines(unmatched, "right only")
        }
    }
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

fn is_error_node(node: &DiffNode) -> bool {
    node.status == DiffStatus::Error
}

fn is_aggregate_node(node: &DiffNode) -> bool {
    !is_error_node(node) && node.left_value.is_none() && node.right_value.is_none()
}

fn detail_context_text(node: &DiffNode) -> Option<String> {
    if is_error_node(node) && node.left_value.is_none() && node.right_value.is_none() {
        Some("Error node; diagnostics are shown in the summary for this node.".to_string())
    } else if is_aggregate_node(node) {
        Some(format!(
            "Aggregate node with {} child change(s); inspect child entries for concrete values.",
            node.children.len()
        ))
    } else {
        None
    }
}

fn detail_value_text(node: &DiffNode, value: Option<&str>) -> String {
    match value {
        Some(value) => value.to_string(),
        None if is_error_node(node) => "(not captured for error node)".to_string(),
        None if is_aggregate_node(node) && node.children.is_empty() => {
            "(no direct value snapshot recorded for this node)".to_string()
        }
        None if is_aggregate_node(node) => "(container node; no direct value snapshot)".to_string(),
        None => "(missing)".to_string(),
    }
}

fn unmatched_detail_lines(unmatched: &UnmatchedFile, status: &str) -> Vec<String> {
    vec![
        format!("File: {}", unmatched.file.file_name),
        format!("Status: {status}"),
        format!(
            "Relative path: {}",
            unmatched.file.relative_path.to_string_lossy()
        ),
        format!("Reason: {}", unmatched.reason),
    ]
}

fn match_strategy_label(strategy: &MatchStrategy) -> &'static str {
    match strategy {
        MatchStrategy::FileName => "file name",
        MatchStrategy::FileNameAndParentDir => "file name + parent directory",
    }
}

#[cfg(test)]
mod tests {
    use super::{batch_detail_lines, detail_context_text, detail_value_text};
    use crate::app::BatchSelection;
    use crate::batch_report::{BatchCompareReport, UnmatchedFile, UnmatchedSide};
    use crate::batch_scan::BatchFileRecord;
    use crate::diff::{DiffNode, DiffStatus};
    use std::path::PathBuf;

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

    #[test]
    fn detail_context_text_distinguishes_error_nodes_without_snapshots() {
        let node = DiffNode {
            path: "Lines.__error__.left[0]".into(),
            status: DiffStatus::Error,
            left_value: None,
            right_value: None,
            summary: "missing business key in left at Lines[0]".into(),
            children: Vec::new(),
        };

        assert_eq!(
            detail_context_text(&node).as_deref(),
            Some("Error node; diagnostics are shown in the summary for this node.")
        );
    }

    #[test]
    fn batch_detail_lines_describe_unmatched_items_with_reason() {
        let report = BatchCompareReport {
            left_only: vec![UnmatchedFile {
                side: UnmatchedSide::Left,
                file: BatchFileRecord {
                    absolute_path: PathBuf::from("C:/batch/left/left-only.png"),
                    relative_path: PathBuf::from("left-only.png"),
                    file_name: "left-only.png".to_string(),
                    parent_dir_name: None,
                },
                reason: "no file named 'left-only.png' found on right side".to_string(),
            }],
            ..BatchCompareReport::default()
        };

        assert_eq!(
            batch_detail_lines(&report, Some(BatchSelection::LeftOnly(0))),
            vec![
                "File: left-only.png".to_string(),
                "Status: left only".to_string(),
                "Relative path: left-only.png".to_string(),
                "Reason: no file named 'left-only.png' found on right side".to_string(),
            ]
        );
    }
}
