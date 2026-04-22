use crate::diff::{DiffNode, DiffSummary, compare_metadata, flatten_changes, summarize_changes};
use crate::metadata::load_metadata;
use crate::png_reader::extract_stop_plate_metadata_from_file;
use crate::ui::{detail, summary, tree};
use rfd::FileDialog;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CompareResultView {
    pub root: DiffNode,
    pub change_list: Vec<DiffNode>,
    pub summary: DiffSummary,
    pub selected_path: Option<String>,
}

pub struct PngMetadataCompareApp {
    pub left_path: Option<String>,
    pub right_path: Option<String>,
    pub result: Option<CompareResultView>,
    pub filters: tree::TreeFilters,
}

impl Default for PngMetadataCompareApp {
    fn default() -> Self {
        Self {
            left_path: None,
            right_path: None,
            result: None,
            filters: tree::TreeFilters::default(),
        }
    }
}

impl PngMetadataCompareApp {
    pub fn can_compare(&self) -> bool {
        self.left_path.is_some() && self.right_path.is_some()
    }

    pub fn run_compare(&mut self) {
        let (Some(left_path), Some(right_path)) =
            (self.left_path.as_deref(), self.right_path.as_deref())
        else {
            self.result = None;
            return;
        };

        let left_metadata =
            load_metadata(extract_stop_plate_metadata_from_file(Path::new(left_path)));
        let right_metadata =
            load_metadata(extract_stop_plate_metadata_from_file(Path::new(right_path)));
        let root = compare_metadata(&left_metadata, &right_metadata);
        let change_list = flatten_changes(&root);
        let summary = summarize_changes(&change_list);
        let selected_path = default_selected_path(&change_list);

        self.result = Some(CompareResultView {
            root,
            change_list,
            summary,
            selected_path,
        });
    }

    fn render_scaffold(&mut self, ctx: &eframe::egui::Context) {
        eframe::egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.heading("PNG Metadata Compare");
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                if ui.button("Choose left PNG").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("PNG image", &["png"])
                        .pick_file()
                    {
                        self.left_path = Some(path.display().to_string());
                        self.result = None;
                    }
                }
                ui.label(
                    self.left_path
                        .as_deref()
                        .unwrap_or("Left file not selected"),
                );

                if ui.button("Choose right PNG").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("PNG image", &["png"])
                        .pick_file()
                    {
                        self.right_path = Some(path.display().to_string());
                        self.result = None;
                    }
                }
                ui.label(
                    self.right_path
                        .as_deref()
                        .unwrap_or("Right file not selected"),
                );

                if ui
                    .add_enabled(self.can_compare(), eframe::egui::Button::new("Compare"))
                    .clicked()
                {
                    self.run_compare();
                }

                if ui
                    .add_enabled(
                        self.left_path.is_some() || self.right_path.is_some(),
                        eframe::egui::Button::new("Swap"),
                    )
                    .clicked()
                {
                    std::mem::swap(&mut self.left_path, &mut self.right_path);
                    self.result = None;
                }
            });
        });

        eframe::egui::SidePanel::left("summary_panel").show(ctx, |ui| {
            ui.heading("Summary");
            ui.separator();

            if let Some(result) = self.result.as_mut() {
                summary::draw_summary(ui, result);
            } else {
                ui.label("Choose two PNG files and run compare to view the summary.");
            }
        });

        eframe::egui::TopBottomPanel::bottom("detail_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Details");
                ui.separator();
                detail::draw_detail(ui, self.result.as_ref());
            });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Diff Tree");
            ui.separator();
            let Some(result) = self.result.as_mut() else {
                ui.label("Run compare to populate the diff tree.");
                return;
            };

            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut self.filters.only_differences, "Only differences");
                ui.checkbox(&mut self.filters.show_reordered, "Show reordered");
                ui.checkbox(&mut self.filters.show_unchanged, "Show unchanged");
                ui.checkbox(&mut self.filters.show_errors, "Show errors");
            });
            ui.separator();

            if result.summary.total() == 0 {
                ui.label("No differences found between the selected PNG metadata.");
                return;
            }

            tree::draw_tree(ui, result, &self.filters);
        });
    }
}

fn default_selected_path(change_list: &[DiffNode]) -> Option<String> {
    change_list
        .iter()
        .find(|node| node.left_value.is_some() || node.right_value.is_some())
        .or_else(|| change_list.first())
        .map(|node| node.path.clone())
}

impl eframe::App for PngMetadataCompareApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.render_scaffold(ctx);
    }
}

#[cfg(test)]
pub(crate) fn run_compare_pipeline_builds_diff_and_counts_test() {
    run_compare_pipeline_builds_diff_and_counts_impl();
}

#[cfg(test)]
fn run_compare_pipeline_builds_diff_and_counts_impl() {
    use crate::diff::DiffStatus;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestPngFile {
        path: PathBuf,
    }

    impl TestPngFile {
        fn new(label: &str, metadata_json: &str) -> Self {
            let path = unique_test_path(label);
            let bytes = png_with_stop_plate_metadata(metadata_json);
            fs::write(&path, bytes).expect("test png should be written");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestPngFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn unique_test_path(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "png_metadata_compare_{label}_{}_{}.png",
            std::process::id(),
            unique
        ))
    }

    fn png_with_stop_plate_metadata(json: &str) -> Vec<u8> {
        let mut bytes = Vec::from(b"\x89PNG\r\n\x1a\n".as_slice());
        bytes.extend(png_chunk(*b"iTXt", stop_plate_itxt_data(json)));
        bytes.extend(png_chunk(*b"IEND", Vec::new()));
        bytes
    }

    fn stop_plate_itxt_data(json: &str) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"StopPlateMetadata");
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.extend_from_slice(json.as_bytes());
        data
    }

    fn png_chunk(kind: [u8; 4], data: Vec<u8>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&kind);
        bytes.extend_from_slice(&data);
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes
    }

    let left = TestPngFile::new(
        "left",
        r#"{
                "Title": "Old title",
                "LegacyCode": "A1",
                "Lines": [
                    {
                        "LineName": "B932",
                        "Direction": "Terminal",
                        "PriceDescription": "1"
                    },
                    {
                        "LineName": "M375",
                        "Direction": "Downtown",
                        "PriceDescription": "2"
                    }
                ]
            }"#,
    );
    let right = TestPngFile::new(
        "right",
        r#"{
                "Title": "New title",
                "NewField": "added",
                "Lines": [
                    {
                        "LineName": "M375",
                        "Direction": "Downtown",
                        "PriceDescription": "3"
                    },
                    {
                        "LineName": "B932",
                        "Direction": "Terminal",
                        "PriceDescription": "1"
                    }
                ]
            }"#,
    );

    let mut app = PngMetadataCompareApp {
        left_path: Some(left.path().display().to_string()),
        right_path: Some(right.path().display().to_string()),
        ..Default::default()
    };

    app.run_compare();

    let result = app.result.expect("compare result should be stored");
    assert_eq!(result.root.path, "StopPlateMetadata");
    assert_eq!(result.root.status, DiffStatus::Modified);
    assert_ne!(result.selected_path.as_deref(), Some("StopPlateMetadata"));
    let selected_path = result
        .selected_path
        .as_deref()
        .expect("compare result should select a changed node with a snapshot");
    let selected_node = result
        .change_list
        .iter()
        .find(|node| node.path == selected_path)
        .unwrap_or_else(|| panic!("missing selected node in change list: {selected_path}"));
    assert!(
        selected_node.left_value.is_some() || selected_node.right_value.is_some(),
        "selected node should expose at least one direct snapshot: {selected_node:#?}"
    );
    assert_eq!(result.summary.modified, 5);
    assert_eq!(result.summary.added, 1);
    assert_eq!(result.summary.removed, 1);
    assert_eq!(result.summary.reordered, 2);
    assert_eq!(result.summary.error, 0);
    assert_eq!(result.summary.total(), result.change_list.len());
    assert!(
        result
            .change_list
            .iter()
            .any(|node| node.path == "Title" && node.status == DiffStatus::Modified),
        "expected modified Title node: {:#?}",
        result.change_list
    );
    assert!(
        result
            .change_list
            .iter()
            .any(|node| node.path == "NewField" && node.status == DiffStatus::Added),
        "expected added NewField node: {:#?}",
        result.change_list
    );
    assert!(
        result
            .change_list
            .iter()
            .any(|node| node.path == "LegacyCode" && node.status == DiffStatus::Removed),
        "expected removed LegacyCode node: {:#?}",
        result.change_list
    );
}

#[cfg(test)]
mod tests {
    use super::PngMetadataCompareApp;
    use crate::diff::{DiffNode, DiffStatus};

    #[test]
    fn compare_is_disabled_until_both_paths_are_present() {
        let mut app = PngMetadataCompareApp::default();
        assert!(!app.can_compare());

        app.left_path = Some("left.png".into());
        assert!(!app.can_compare());

        app.right_path = Some("right.png".into());
        assert!(app.can_compare());
    }

    #[test]
    fn scaffold_render_produces_output() {
        let mut app = PngMetadataCompareApp::default();
        let ctx = eframe::egui::Context::default();
        let output = ctx.run(Default::default(), |ctx| {
            app.render_scaffold(ctx);
        });

        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn compare_pipeline_builds_diff_and_counts() {
        super::run_compare_pipeline_builds_diff_and_counts_impl();
    }

    #[test]
    fn default_selected_path_skips_root_aggregate_when_leaf_change_exists() {
        let changes = vec![
            DiffNode {
                path: "StopPlateMetadata".into(),
                status: DiffStatus::Modified,
                left_value: None,
                right_value: None,
                summary: "root modified".into(),
                children: Vec::new(),
            },
            DiffNode {
                path: "Title".into(),
                status: DiffStatus::Modified,
                left_value: Some("\"old\"".into()),
                right_value: Some("\"new\"".into()),
                summary: "Title modified".into(),
                children: Vec::new(),
            },
        ];

        assert_eq!(
            super::default_selected_path(&changes).as_deref(),
            Some("Title")
        );
    }

    #[test]
    fn default_selected_path_is_none_when_no_changes_exist() {
        assert_eq!(super::default_selected_path(&[]), None);
    }
}
