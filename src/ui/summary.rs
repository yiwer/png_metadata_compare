use crate::app::CompareResultView;
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

#[cfg(test)]
mod tests {
    use super::summary_lines;
    use crate::diff::DiffSummary;

    #[test]
    fn formats_summary_lines_for_sidebar() {
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
}
