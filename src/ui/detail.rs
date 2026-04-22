use crate::app::{BatchSelection, CompareResultView};
use crate::batch_report::{BatchCompareReport, MatchStrategy, UnmatchedFile};
use crate::diff::{DiffNode, DiffStatus};
use crate::ui::summary::batch_issue_lines;
use std::path::Path;

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

    draw_detail_from_parts(ui, &result.root, result.selected_path.as_deref());
}

pub fn draw_detail_from_parts(
    ui: &mut eframe::egui::Ui,
    root: &DiffNode,
    selected_path: Option<&str>,
) {
    let node = match selected_detail_node(root, selected_path) {
        Ok(node) => node,
        Err(message) => {
            ui.label(message);
            return;
        }
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
    for line in batch_detail_panel_lines(report, selection) {
        ui.label(line);
    }
}

pub(crate) fn batch_detail_panel_lines(
    report: Option<&BatchCompareReport>,
    selection: Option<BatchSelection>,
) -> Vec<String> {
    let Some(report) = report else {
        return vec!["Run directory compare to inspect batch details.".to_string()];
    };

    let Some(selection) = selection else {
        let mut lines = batch_issue_lines(&report.issues);
        lines.push("Select a batch item to inspect details.".to_string());
        return lines;
    };

    batch_detail_lines(report, Some(selection))
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
                "Status: Identical".to_string(),
                format!(
                    "Match strategy: {}",
                    match_strategy_label(&identical.pair.match_strategy)
                ),
                format!(
                    "Left: {}",
                    normalized_path_text(&identical.pair.left.relative_path)
                ),
                format!(
                    "Right: {}",
                    normalized_path_text(&identical.pair.right.relative_path)
                ),
            ]
        }
        BatchSelection::Different(index) => {
            let Some(different) = report.different.get(index) else {
                return vec!["The selected batch item is no longer available.".to_string()];
            };
            node_detail_lines(&different.diff_root, different.selected_path.as_deref())
        }
        BatchSelection::LeftOnly(index) => {
            let Some(unmatched) = report.left_only.get(index) else {
                return vec!["The selected batch item is no longer available.".to_string()];
            };
            unmatched_detail_lines(unmatched, "Left Only")
        }
        BatchSelection::RightOnly(index) => {
            let Some(unmatched) = report.right_only.get(index) else {
                return vec!["The selected batch item is no longer available.".to_string()];
            };
            unmatched_detail_lines(unmatched, "Right Only")
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

fn selected_detail_node<'a>(
    root: &'a DiffNode,
    selected_path: Option<&str>,
) -> Result<&'a DiffNode, &'static str> {
    let Some(selected_path) = selected_path else {
        return Err("Select a changed node to inspect its details.");
    };

    find_node_by_path(root, selected_path)
        .ok_or("The selected node is no longer available in the current diff.")
}

pub(crate) fn node_detail_lines(root: &DiffNode, selected_path: Option<&str>) -> Vec<String> {
    let node = match selected_detail_node(root, selected_path) {
        Ok(node) => node,
        Err(message) => return vec![message.to_string()],
    };

    let mut lines = vec![
        format!("Path: {}", node.path),
        format!("Status: {}", status_label(&node.status)),
        format!("Summary: {}", node.summary),
    ];
    if let Some(context) = detail_context_text(node) {
        lines.push(context);
    }
    lines.push("Left value".to_string());
    lines.push(detail_value_text(node, node.left_value.as_deref()));
    lines.push("Right value".to_string());
    lines.push(detail_value_text(node, node.right_value.as_deref()));
    lines
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
            normalized_path_text(&unmatched.file.relative_path)
        ),
        format!("Reason: {}", unmatched.reason),
    ]
}

fn normalized_path_text(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn match_strategy_label(strategy: &MatchStrategy) -> &'static str {
    match strategy {
        MatchStrategy::FileName => "file name",
        MatchStrategy::FileNameAndParentDir => "file name + parent directory",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        batch_detail_lines, batch_detail_panel_lines, detail_context_text, detail_value_text,
        node_detail_lines,
    };
    use crate::app::BatchSelection;
    use crate::batch_report::{
        BatchCompareReport, BatchIssue, DifferentPairResult, IdenticalPairResult, MatchStrategy,
        MatchedPair, UnmatchedFile, UnmatchedSide,
    };
    use crate::batch_scan::BatchFileRecord;
    use crate::diff::{DiffNode, DiffStatus, DiffSummary};
    use std::path::PathBuf;

    fn record(relative: &str) -> BatchFileRecord {
        let relative_path = PathBuf::from(relative);
        let file_name = relative_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("test path should include a UTF-8 file name")
            .to_string();
        let parent_dir_name = relative_path
            .parent()
            .and_then(|parent| parent.file_name())
            .map(|name| name.to_string_lossy().to_string());

        BatchFileRecord {
            absolute_path: PathBuf::from("C:/batch").join(relative),
            relative_path,
            file_name,
            parent_dir_name,
        }
    }

    fn matched_pair(file_name: &str, left: &str, right: &str) -> MatchedPair {
        MatchedPair {
            file_name: file_name.to_string(),
            left: record(left),
            right: record(right),
            match_strategy: MatchStrategy::FileNameAndParentDir,
        }
    }

    fn sample_report() -> BatchCompareReport {
        BatchCompareReport {
            issues: vec![BatchIssue::ScanFailure {
                side: UnmatchedSide::Right,
                path: PathBuf::from("C:/batch/right"),
                reason: "permission denied".to_string(),
            }],
            identical: vec![IdenticalPairResult {
                pair: matched_pair("same.png", "left/routes/same.png", "right/routes/same.png"),
            }],
            different: vec![DifferentPairResult {
                pair: matched_pair(
                    "changed.png",
                    "left/routes/changed.png",
                    "right/routes/changed.png",
                ),
                diff_root: DiffNode {
                    path: "StopPlateMetadata".into(),
                    status: DiffStatus::Modified,
                    left_value: None,
                    right_value: None,
                    summary: "root modified".into(),
                    children: vec![DiffNode {
                        path: "StopPlateMetadata.Title".into(),
                        status: DiffStatus::Modified,
                        left_value: Some("\"left title\"".into()),
                        right_value: Some("\"right title\"".into()),
                        summary: "title modified".into(),
                        children: Vec::new(),
                    }],
                },
                change_list: Vec::new(),
                summary: DiffSummary {
                    modified: 2,
                    added: 1,
                    ..DiffSummary::default()
                },
                selected_path: Some("StopPlateMetadata.Title".into()),
            }],
            left_only: vec![UnmatchedFile {
                side: UnmatchedSide::Left,
                file: record("left/left-only.png"),
                reason: "no file named 'left-only.png' found on right side".to_string(),
            }],
            right_only: vec![UnmatchedFile {
                side: UnmatchedSide::Right,
                file: record("right/right-only.png"),
                reason: "no file named 'right-only.png' found on left side".to_string(),
            }],
        }
    }

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
    fn node_detail_lines_prompt_when_no_node_is_selected() {
        let root = DiffNode {
            path: "StopPlateMetadata".into(),
            status: DiffStatus::Modified,
            left_value: None,
            right_value: None,
            summary: "root modified".into(),
            children: Vec::new(),
        };

        assert_eq!(
            node_detail_lines(&root, None),
            vec!["Select a changed node to inspect its details.".to_string()]
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
                "Status: Left Only".to_string(),
                "Relative path: left-only.png".to_string(),
                "Reason: no file named 'left-only.png' found on right side".to_string(),
            ]
        );
    }

    #[test]
    fn batch_detail_panel_lines_prompt_before_report_exists() {
        assert_eq!(
            batch_detail_panel_lines(None, None),
            vec!["Run directory compare to inspect batch details.".to_string()]
        );
    }

    #[test]
    fn batch_detail_panel_lines_show_issues_when_no_item_is_selected() {
        let report = sample_report();

        assert_eq!(
            batch_detail_panel_lines(Some(&report), None),
            vec![
                "Issues (1)".to_string(),
                "Right scan failure [C:/batch/right] :: permission denied".to_string(),
                "Select a batch item to inspect details.".to_string(),
            ]
        );
    }

    #[test]
    fn batch_detail_panel_lines_describe_identical_selection() {
        let report = sample_report();

        assert_eq!(
            batch_detail_panel_lines(Some(&report), Some(BatchSelection::Identical(0))),
            vec![
                "File: same.png".to_string(),
                "Status: Identical".to_string(),
                "Match strategy: file name + parent directory".to_string(),
                "Left: left/routes/same.png".to_string(),
                "Right: right/routes/same.png".to_string(),
            ]
        );
    }

    #[test]
    fn batch_detail_panel_lines_use_selected_node_details_for_different_selection() {
        let report = sample_report();

        assert_eq!(
            batch_detail_panel_lines(Some(&report), Some(BatchSelection::Different(0))),
            vec![
                "Path: StopPlateMetadata.Title".to_string(),
                "Status: Modified".to_string(),
                "Summary: title modified".to_string(),
                "Left value".to_string(),
                "\"left title\"".to_string(),
                "Right value".to_string(),
                "\"right title\"".to_string(),
            ]
        );
    }

    #[test]
    fn batch_detail_panel_lines_fall_back_when_selection_is_stale() {
        let report = sample_report();

        assert_eq!(
            batch_detail_panel_lines(Some(&report), Some(BatchSelection::Different(99))),
            vec!["The selected batch item is no longer available.".to_string()]
        );
    }
}
