use crate::app::{BatchSelection, CompareResultView};
use crate::batch_report::BatchCompareReport;
use crate::diff::DiffSummary;

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
        format!("Left only ({left_only})"),
        format!("Right only ({right_only})"),
    ]
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

    ui.separator();
    ui.label("Batch items");
    eframe::egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("Identical");
        for (index, identical) in report.identical.iter().enumerate() {
            let item_selection = BatchSelection::Identical(index);
            let is_selected = *selection == Some(item_selection);
            if ui
                .selectable_label(is_selected, identical.pair.file_name.clone())
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
            let label = format!(
                "{} ({} changes)",
                different.pair.file_name,
                different.summary.total()
            );
            if ui.selectable_label(is_selected, label).clicked() {
                *selection = Some(item_selection);
            }
        }

        ui.separator();
        ui.label("Left only");
        for (index, unmatched) in report.left_only.iter().enumerate() {
            let item_selection = BatchSelection::LeftOnly(index);
            let is_selected = *selection == Some(item_selection);
            if ui
                .selectable_label(is_selected, unmatched.file.file_name.clone())
                .clicked()
            {
                *selection = Some(item_selection);
            }
        }

        ui.separator();
        ui.label("Right only");
        for (index, unmatched) in report.right_only.iter().enumerate() {
            let item_selection = BatchSelection::RightOnly(index);
            let is_selected = *selection == Some(item_selection);
            if ui
                .selectable_label(is_selected, unmatched.file.file_name.clone())
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
    use super::{batch_section_labels, formats_summary_lines_for_sidebar_test};

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
                "Left only (1)".to_string(),
                "Right only (4)".to_string(),
            ]
        );
    }
}
