use crate::app::{BatchSelection, CompareResultView};
use crate::batch_report::{
    BatchCompareReport, BatchIssue, DifferentPairResult, IdenticalPairResult, MatchedPair,
    UnmatchedFile, UnmatchedSide,
};
use crate::diff::DiffSummary;
use std::path::Path;

pub fn summary_lines(summary: &DiffSummary) -> Vec<String> {
    let mut lines = vec![format!("{} total changes", summary.total())];

    for (count, label) in [
        (summary.modified, "modified"),
        (summary.added, "added"),
        (summary.removed, "removed"),
        (summary.reordered, "reordered"),
        (summary.error, "errors"),
    ] {
        if count > 0 {
            lines.push(format!("{count} {label}"));
        }
    }

    lines
}

pub fn draw_summary(ui: &mut eframe::egui::Ui, result: &mut CompareResultView) {
    for line in summary_lines(&result.summary) {
        ui.label(line);
    }

    ui.separator();
    ui.label("Changed nodes");

    if result.change_list.is_empty() {
        ui.label("No changed nodes to inspect.");
        return;
    }

    let entries: Vec<(String, String)> = result
        .change_list
        .iter()
        .map(|node| (node.path.clone(), node.summary.clone()))
        .collect();

    eframe::egui::ScrollArea::vertical().show(ui, |ui| {
        for (path, summary) in entries {
            let selected = result.selected_path.as_deref() == Some(path.as_str());
            if ui
                .selectable_label(selected, format!("{path}: {summary}"))
                .clicked()
            {
                result.selected_path = Some(path);
            }
        }
    });
}

pub fn batch_section_labels(
    identical: usize,
    different: usize,
    left_only: usize,
    right_only: usize,
) -> Vec<String> {
    vec![
        format!("Identical ({identical})"),
        format!("Different ({different})"),
        format!("Left Only ({left_only})"),
        format!("Right Only ({right_only})"),
    ]
}

pub(crate) fn batch_issue_lines(issues: &[BatchIssue]) -> Vec<String> {
    if issues.is_empty() {
        return Vec::new();
    }

    let mut lines = vec![format!("Issues ({})", issues.len())];
    lines.extend(issues.iter().map(batch_issue_line));
    lines
}

fn batch_issue_line(issue: &BatchIssue) -> String {
    match issue {
        BatchIssue::ScanFailure { side, path, reason } => {
            format!(
                "{} scan failure [{}] :: {}",
                unmatched_side_label(side),
                normalized_path_text(path),
                reason
            )
        }
    }
}

fn identical_item_label(identical: &IdenticalPairResult) -> String {
    format!(
        "{} [{}] :: Metadata identical",
        identical.pair.file_name,
        matched_pair_location_context(&identical.pair)
    )
}

fn different_item_label(different: &DifferentPairResult) -> String {
    format!(
        "{} [{}] :: {} changes",
        different.pair.file_name,
        matched_pair_location_context(&different.pair),
        different.summary.total()
    )
}

fn unmatched_item_label(unmatched: &UnmatchedFile) -> String {
    format!(
        "{} [{}] :: {}",
        unmatched.file.file_name,
        normalized_path_text(&unmatched.file.relative_path),
        unmatched.reason
    )
}

fn matched_pair_location_context(pair: &MatchedPair) -> String {
    let left = normalized_path_text(&pair.left.relative_path);
    let right = normalized_path_text(&pair.right.relative_path);

    if left == right {
        left
    } else {
        format!("L {left} | R {right}")
    }
}

fn unmatched_side_label(side: &UnmatchedSide) -> &'static str {
    match side {
        UnmatchedSide::Left => "Left",
        UnmatchedSide::Right => "Right",
    }
}

fn normalized_path_text(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn draw_batch_summary(
    ui: &mut eframe::egui::Ui,
    report: &BatchCompareReport,
    selection: &mut Option<BatchSelection>,
) {
    let labels = batch_section_labels(
        report.identical.len(),
        report.different.len(),
        report.left_only.len(),
        report.right_only.len(),
    );
    for label in labels {
        ui.label(label);
    }

    let issue_lines = batch_issue_lines(&report.issues);
    if !issue_lines.is_empty() {
        ui.separator();
        for line in issue_lines {
            ui.label(line);
        }
    }

    ui.separator();
    ui.label("Batch items");
    eframe::egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("Identical");
        for (index, identical) in report.identical.iter().enumerate() {
            let item_selection = BatchSelection::Identical(index);
            let is_selected = *selection == Some(item_selection);
            if ui
                .selectable_label(is_selected, identical_item_label(identical))
                .clicked()
            {
                *selection = Some(item_selection);
            }
        }

        ui.separator();
        ui.label("Different");
        for (index, different) in report.different.iter().enumerate() {
            let item_selection = BatchSelection::Different(index);
            let is_selected = *selection == Some(item_selection);
            if ui
                .selectable_label(is_selected, different_item_label(different))
                .clicked()
            {
                *selection = Some(item_selection);
            }
        }

        ui.separator();
        ui.label("Left Only");
        for (index, unmatched) in report.left_only.iter().enumerate() {
            let item_selection = BatchSelection::LeftOnly(index);
            let is_selected = *selection == Some(item_selection);
            if ui
                .selectable_label(is_selected, unmatched_item_label(unmatched))
                .clicked()
            {
                *selection = Some(item_selection);
            }
        }

        ui.separator();
        ui.label("Right Only");
        for (index, unmatched) in report.right_only.iter().enumerate() {
            let item_selection = BatchSelection::RightOnly(index);
            let is_selected = *selection == Some(item_selection);
            if ui
                .selectable_label(is_selected, unmatched_item_label(unmatched))
                .clicked()
            {
                *selection = Some(item_selection);
            }
        }
    });
}

#[cfg(test)]
pub(crate) fn formats_summary_lines_for_sidebar_test() {
    let summary = DiffSummary {
        modified: 5,
        added: 1,
        removed: 1,
        reordered: 2,
        error: 0,
    };

    assert_eq!(
        summary_lines(&summary),
        vec![
            "9 total changes".to_string(),
            "5 modified".to_string(),
            "1 added".to_string(),
            "1 removed".to_string(),
            "2 reordered".to_string(),
        ]
    );
}

#[cfg(test)]
mod tests {
    use super::{
        batch_issue_lines, batch_section_labels, different_item_label,
        formats_summary_lines_for_sidebar_test, identical_item_label, unmatched_item_label,
    };
    use crate::batch_report::{
        BatchIssue, DifferentPairResult, IdenticalPairResult, MatchStrategy, MatchedPair,
        UnmatchedFile, UnmatchedSide,
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
            absolute_path: PathBuf::from("C:/tests").join(relative),
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

    #[test]
    fn formats_summary_lines_for_sidebar() {
        formats_summary_lines_for_sidebar_test();
    }

    #[test]
    fn batch_section_labels_include_all_counts_in_fixed_order() {
        assert_eq!(
            batch_section_labels(3, 2, 1, 4),
            vec![
                "Identical (3)".to_string(),
                "Different (2)".to_string(),
                "Left Only (1)".to_string(),
                "Right Only (4)".to_string(),
            ]
        );
    }

    #[test]
    fn identical_item_label_includes_location_context() {
        let identical = IdenticalPairResult {
            pair: matched_pair(
                "shared.png",
                "routes/weekday/shared.png",
                "routes/weekday/shared.png",
            ),
        };

        assert_eq!(
            identical_item_label(&identical),
            "shared.png [routes/weekday/shared.png] :: Metadata identical"
        );
    }

    #[test]
    fn different_item_label_includes_location_context_and_change_count() {
        let different = DifferentPairResult {
            pair: matched_pair(
                "shared.png",
                "routes/weekday/shared.png",
                "routes/weekend/shared.png",
            ),
            diff_root: DiffNode {
                path: "StopPlateMetadata".into(),
                status: DiffStatus::Modified,
                left_value: None,
                right_value: None,
                summary: "root modified".into(),
                children: Vec::new(),
            },
            change_list: Vec::new(),
            summary: DiffSummary {
                modified: 2,
                ..DiffSummary::default()
            },
            selected_path: None,
        };

        assert_eq!(
            different_item_label(&different),
            "shared.png [L routes/weekday/shared.png | R routes/weekend/shared.png] :: 2 changes"
        );
    }

    #[test]
    fn unmatched_item_label_includes_location_context_and_reason() {
        let unmatched = UnmatchedFile {
            side: UnmatchedSide::Left,
            file: record("routes/weekday/shared.png"),
            reason: "duplicate file name could not be uniquely resolved".to_string(),
        };

        assert_eq!(
            unmatched_item_label(&unmatched),
            "shared.png [routes/weekday/shared.png] :: duplicate file name could not be uniquely resolved"
        );
    }

    #[test]
    fn batch_issue_lines_include_issue_count_and_details() {
        let issues = vec![BatchIssue::ScanFailure {
            side: UnmatchedSide::Right,
            path: PathBuf::from("C:/tests/right"),
            reason: "permission denied".to_string(),
        }];

        assert_eq!(
            batch_issue_lines(&issues),
            vec![
                "Issues (1)".to_string(),
                "Right scan failure [C:/tests/right] :: permission denied".to_string(),
            ]
        );
    }
}
