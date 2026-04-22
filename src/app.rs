use crate::batch_report::{
    BatchCompareReport, BatchIssue, MatchedPairCompareResult, UnmatchedSide, build_batch_results,
};
use crate::batch_scan::{build_pairing, scan_png_files};
use crate::diff::{DiffNode, DiffSummary, compare_metadata, flatten_changes, summarize_changes};
use crate::error::CompareError;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    SingleFile,
    Directory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchSelection {
    Identical(usize),
    Different(usize),
    LeftOnly(usize),
    RightOnly(usize),
}

pub struct PngMetadataCompareApp {
    pub left_path: Option<String>,
    pub right_path: Option<String>,
    pub left_dir: Option<String>,
    pub right_dir: Option<String>,
    pub result: Option<CompareResultView>,
    pub batch_report: Option<BatchCompareReport>,
    pub batch_selection: Option<BatchSelection>,
    pub mode: AppMode,
    pub filters: tree::TreeFilters,
}

impl Default for PngMetadataCompareApp {
    fn default() -> Self {
        Self {
            left_path: None,
            right_path: None,
            left_dir: None,
            right_dir: None,
            result: None,
            batch_report: None,
            batch_selection: None,
            mode: AppMode::SingleFile,
            filters: tree::TreeFilters::default(),
        }
    }
}

impl PngMetadataCompareApp {
    pub fn can_compare(&self) -> bool {
        match self.mode {
            AppMode::SingleFile => self.left_path.is_some() && self.right_path.is_some(),
            AppMode::Directory => self.left_dir.is_some() && self.right_dir.is_some(),
        }
    }

    fn reconcile_tree_selection(&mut self) {
        if let Some(result) = self.result.as_mut() {
            tree::reconcile_selected_path(result, &self.filters);
        }
    }

    pub fn run_compare(&mut self) {
        let (Some(left_path), Some(right_path)) =
            (self.left_path.as_deref(), self.right_path.as_deref())
        else {
            self.result = None;
            self.batch_report = None;
            self.batch_selection = None;
            return;
        };

        self.result = Some(compare_paths(Path::new(left_path), Path::new(right_path)));
        self.batch_report = None;
        self.batch_selection = None;
    }

    pub fn run_directory_compare(&mut self) {
        let (Some(left_dir), Some(right_dir)) =
            (self.left_dir.as_deref(), self.right_dir.as_deref())
        else {
            self.result = None;
            self.batch_report = None;
            self.batch_selection = None;
            return;
        };

        let mut issues = Vec::new();
        let left_files = scan_directory_side(Path::new(left_dir), UnmatchedSide::Left, &mut issues);
        let right_files =
            scan_directory_side(Path::new(right_dir), UnmatchedSide::Right, &mut issues);
        let pairing = build_pairing(&left_files, &right_files);
        let matched_results = pairing
            .matched
            .into_iter()
            .map(compare_matched_pair)
            .collect();

        let mut batch_report = build_batch_results(
            matched_results,
            pairing.left_only,
            pairing.right_only,
            issues,
        );

        for different in &mut batch_report.different {
            different.selected_path = default_selected_path(&different.change_list);
        }

        self.result = None;
        self.batch_selection = default_batch_selection(&batch_report);
        self.batch_report = Some(batch_report);
    }

    fn render_scaffold(&mut self, ctx: &eframe::egui::Context) {
        self.reconcile_tree_selection();

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
            let state = central_panel_state(self.result.as_ref(), &self.filters);
            if let Some(message) = state.empty_message {
                ui.label(message);
                return;
            }

            let mut filters_changed = false;
            ui.horizontal_wrapped(|ui| {
                filters_changed |= ui
                    .checkbox(&mut self.filters.only_differences, "Only differences")
                    .changed();
                filters_changed |= ui
                    .checkbox(&mut self.filters.show_reordered, "Show reordered")
                    .changed();
                filters_changed |= ui
                    .checkbox(&mut self.filters.show_unchanged, "Show unchanged")
                    .changed();
                filters_changed |= ui
                    .checkbox(&mut self.filters.show_errors, "Show errors")
                    .changed();
            });

            if filters_changed {
                self.reconcile_tree_selection();
            }
            ui.separator();

            let Some(result) = self.result.as_mut() else {
                ui.label("Run compare to populate the diff tree.");
                return;
            };

            let state = central_panel_state(Some(&*result), &self.filters);
            if state.show_no_differences_message {
                ui.label("No differences found between the selected PNG metadata.");
            }

            if state.show_tree && state.show_no_differences_message {
                ui.separator();
            }

            if state.show_tree {
                tree::draw_tree(ui, result, &self.filters);
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CentralPanelState {
    empty_message: Option<&'static str>,
    show_no_differences_message: bool,
    show_tree: bool,
}

fn central_panel_state(
    result: Option<&CompareResultView>,
    filters: &tree::TreeFilters,
) -> CentralPanelState {
    let Some(result) = result else {
        return CentralPanelState {
            empty_message: Some("Run compare to populate the diff tree."),
            show_no_differences_message: false,
            show_tree: false,
        };
    };

    if result.summary.total() == 0 {
        return CentralPanelState {
            empty_message: None,
            show_no_differences_message: true,
            show_tree: tree::should_show(&result.root, filters),
        };
    }

    CentralPanelState {
        empty_message: None,
        show_no_differences_message: false,
        show_tree: true,
    }
}

fn compare_paths(left_path: &Path, right_path: &Path) -> CompareResultView {
    let left_metadata = load_metadata(extract_stop_plate_metadata_from_file(left_path));
    let right_metadata = load_metadata(extract_stop_plate_metadata_from_file(right_path));
    let root = compare_metadata(&left_metadata, &right_metadata);
    let change_list = flatten_changes(&root);
    let summary = summarize_changes(&change_list);
    let selected_path = default_selected_path(&change_list);

    CompareResultView {
        root,
        change_list,
        summary,
        selected_path,
    }
}

fn scan_directory_side(
    directory: &Path,
    side: UnmatchedSide,
    issues: &mut Vec<BatchIssue>,
) -> Vec<crate::batch_scan::BatchFileRecord> {
    match scan_png_files(directory) {
        Ok(files) => files,
        Err(error) => {
            let (path, reason) = scan_failure_details(directory, &error);
            issues.push(BatchIssue::ScanFailure { side, path, reason });
            Vec::new()
        }
    }
}

fn scan_failure_details(directory: &Path, error: &CompareError) -> (std::path::PathBuf, String) {
    match error {
        CompareError::FileRead { path, reason } => (path.clone(), reason.clone()),
        _ => (directory.to_path_buf(), error.to_string()),
    }
}

fn compare_matched_pair(pair: crate::batch_report::MatchedPair) -> MatchedPairCompareResult {
    let compare_result = compare_paths(&pair.left.absolute_path, &pair.right.absolute_path);

    if compare_result.summary.total() == 0 {
        MatchedPairCompareResult::identical(pair)
    } else {
        MatchedPairCompareResult::different(
            pair,
            compare_result.root,
            compare_result.change_list,
            compare_result.summary,
        )
    }
}

fn default_batch_selection(report: &BatchCompareReport) -> Option<BatchSelection> {
    if !report.different.is_empty() {
        Some(BatchSelection::Different(0))
    } else if !report.identical.is_empty() {
        Some(BatchSelection::Identical(0))
    } else if !report.left_only.is_empty() {
        Some(BatchSelection::LeftOnly(0))
    } else if !report.right_only.is_empty() {
        Some(BatchSelection::RightOnly(0))
    } else {
        None
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
        bytes.extend_from_slice(&png_chunk_crc(kind, &data).to_be_bytes());
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
mod test_support {
    use super::png_chunk_crc;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub(crate) struct BatchDirFixture {
        root: PathBuf,
        left: PathBuf,
        right: PathBuf,
    }

    impl BatchDirFixture {
        pub(crate) fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "png_metadata_compare_batch_fixture_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            let left = root.join("left");
            let right = root.join("right");
            fs::create_dir_all(&left).expect("left test directory should be created");
            fs::create_dir_all(&right).expect("right test directory should be created");
            Self { root, left, right }
        }

        pub(crate) fn left_dir(&self) -> &Path {
            &self.left
        }

        pub(crate) fn right_dir(&self) -> &Path {
            &self.right
        }

        pub(crate) fn write_left_png(&self, relative: &str, metadata_json: &str) {
            self.write_png(&self.left, relative, metadata_json);
        }

        pub(crate) fn write_right_png(&self, relative: &str, metadata_json: &str) {
            self.write_png(&self.right, relative, metadata_json);
        }

        fn write_png(&self, base_dir: &Path, relative: &str, metadata_json: &str) {
            let path = base_dir.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("fixture parent directory should be created");
            }
            fs::write(&path, png_with_stop_plate_metadata(metadata_json))
                .expect("fixture PNG should be written");
        }
    }

    impl Drop for BatchDirFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
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
        bytes.extend_from_slice(&png_chunk_crc(kind, &data).to_be_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::PngMetadataCompareApp;
    use super::test_support::BatchDirFixture;
    use crate::batch_report::UnmatchedSide;
    use crate::diff::{DiffNode, DiffStatus};
    use crate::ui::tree::TreeFilters;

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
    fn directory_mode_classifies_identical_and_different_pairs() {
        let fixture = BatchDirFixture::new("matched_pairs");
        fixture.write_left_png("same.png", r#"{"Title":"Same"}"#);
        fixture.write_right_png("same.png", r#"{"Title":"Same"}"#);
        fixture.write_left_png("changed.png", r#"{"Title":"Left"}"#);
        fixture.write_right_png("changed.png", r#"{"Title":"Right"}"#);

        let mut app = PngMetadataCompareApp {
            left_dir: Some(fixture.left_dir().display().to_string()),
            right_dir: Some(fixture.right_dir().display().to_string()),
            mode: super::AppMode::Directory,
            ..Default::default()
        };

        app.run_directory_compare();

        let report = app
            .batch_report
            .as_ref()
            .expect("directory compare should store a batch report");
        assert!(report.issues.is_empty());
        assert_eq!(report.identical.len(), 1);
        assert_eq!(report.identical[0].pair.file_name, "same.png");
        assert_eq!(report.different.len(), 1);
        assert_eq!(report.different[0].pair.file_name, "changed.png");
        assert_eq!(report.different[0].diff_root.path, "StopPlateMetadata");
        assert_eq!(report.different[0].diff_root.status, DiffStatus::Modified);
        assert!(
            report.different[0]
                .change_list
                .iter()
                .any(|node| node.path == "Title" && node.status == DiffStatus::Modified),
            "expected changed metadata leaf in flattened batch diff: {:#?}",
            report.different[0].change_list
        );
        assert_eq!(
            app.batch_selection,
            Some(super::BatchSelection::Different(0))
        );
    }

    #[test]
    fn directory_mode_reports_unmatched_files_in_left_only_and_right_only() {
        let fixture = BatchDirFixture::new("unmatched_pairs");
        fixture.write_left_png("left-only.png", r#"{"Title":"Left only"}"#);
        fixture.write_right_png("right-only.png", r#"{"Title":"Right only"}"#);

        let mut app = PngMetadataCompareApp {
            left_dir: Some(fixture.left_dir().display().to_string()),
            right_dir: Some(fixture.right_dir().display().to_string()),
            mode: super::AppMode::Directory,
            ..Default::default()
        };

        app.run_directory_compare();

        let report = app
            .batch_report
            .as_ref()
            .expect("directory compare should store a batch report");
        assert!(report.issues.is_empty());
        assert!(report.identical.is_empty());
        assert!(report.different.is_empty());
        assert_eq!(report.left_only.len(), 1);
        assert_eq!(report.left_only[0].file.file_name, "left-only.png");
        assert_eq!(report.left_only[0].side, UnmatchedSide::Left);
        assert_eq!(report.right_only.len(), 1);
        assert_eq!(report.right_only[0].file.file_name, "right-only.png");
        assert_eq!(report.right_only[0].side, UnmatchedSide::Right);
        assert_eq!(
            app.batch_selection,
            Some(super::BatchSelection::LeftOnly(0))
        );
    }

    #[test]
    fn directory_mode_records_scan_failures_as_batch_issues() {
        use crate::batch_report::BatchIssue;
        use std::fs;

        let fixture = BatchDirFixture::new("scan_failures");
        let invalid_right_path = fixture.right_dir().join("not-a-directory.png");
        fs::write(&invalid_right_path, b"test").expect("invalid scan target file should exist");

        let mut app = PngMetadataCompareApp {
            left_dir: Some(fixture.left_dir().display().to_string()),
            right_dir: Some(invalid_right_path.display().to_string()),
            mode: super::AppMode::Directory,
            ..Default::default()
        };

        app.run_directory_compare();

        let report = app
            .batch_report
            .as_ref()
            .expect("directory compare should still produce a batch report on scan failure");
        assert_eq!(report.issues.len(), 1);
        match &report.issues[0] {
            BatchIssue::ScanFailure { side, path, reason } => {
                assert_eq!(*side, UnmatchedSide::Right);
                assert_eq!(path, &invalid_right_path);
                assert!(!reason.is_empty());
            }
        }
        assert!(report.identical.is_empty());
        assert!(report.different.is_empty());
        assert!(report.left_only.is_empty());
        assert!(report.right_only.is_empty());
        assert_eq!(app.batch_selection, None);
    }

    #[test]
    fn compare_pipeline_surfaces_missing_metadata_as_error_result() {
        use std::fs;
        use std::path::{Path, PathBuf};
        use std::time::{SystemTime, UNIX_EPOCH};

        struct TestPngFile {
            path: PathBuf,
        }

        impl TestPngFile {
            fn new(label: &str, bytes: Vec<u8>) -> Self {
                let path = unique_test_path(label);
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

        fn png_without_stop_plate_metadata() -> Vec<u8> {
            let mut bytes = Vec::from(b"\x89PNG\r\n\x1a\n".as_slice());
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
            bytes.extend_from_slice(&super::png_chunk_crc(kind, &data).to_be_bytes());
            bytes
        }

        let left = TestPngFile::new("missing_left", png_without_stop_plate_metadata());
        let right = TestPngFile::new(
            "valid_right",
            png_with_stop_plate_metadata(r#"{"StopName":"A"}"#),
        );

        let mut app = PngMetadataCompareApp {
            left_path: Some(left.path().display().to_string()),
            right_path: Some(right.path().display().to_string()),
            ..Default::default()
        };

        app.run_compare();

        let result = app
            .result
            .as_ref()
            .expect("compare result should be stored for missing metadata");
        assert_eq!(result.root.path, "StopPlateMetadata");
        assert_eq!(result.root.status, DiffStatus::Error);
        assert_eq!(result.summary.error, 1);
        assert_eq!(result.summary.total(), 1);
        assert_eq!(result.selected_path.as_deref(), Some("StopPlateMetadata"));
        assert!(
            result
                .change_list
                .iter()
                .any(|node| node.path == "StopPlateMetadata" && node.status == DiffStatus::Error),
            "expected compare pipeline to surface an explicit error result: {:#?}",
            result.change_list
        );
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

    #[test]
    fn central_panel_keeps_identical_tree_available_when_filters_show_unchanged() {
        let result = super::CompareResultView {
            root: DiffNode {
                path: "StopPlateMetadata".into(),
                status: DiffStatus::Unchanged,
                left_value: Some("{\"Title\":\"Same\"}".into()),
                right_value: Some("{\"Title\":\"Same\"}".into()),
                summary: "metadata unchanged".into(),
                children: vec![DiffNode {
                    path: "Title".into(),
                    status: DiffStatus::Unchanged,
                    left_value: Some("\"Same\"".into()),
                    right_value: Some("\"Same\"".into()),
                    summary: "Title unchanged".into(),
                    children: Vec::new(),
                }],
            },
            change_list: Vec::new(),
            summary: Default::default(),
            selected_path: Some("Title".into()),
        };
        let filters = TreeFilters {
            only_differences: false,
            show_unchanged: true,
            ..Default::default()
        };

        let state = super::central_panel_state(Some(&result), &filters);

        assert!(state.show_no_differences_message);
        assert!(state.show_tree);
    }

    #[test]
    fn reconcile_tree_selection_moves_hidden_selection_to_visible_node() {
        let mut app = PngMetadataCompareApp {
            filters: TreeFilters::default(),
            result: Some(super::CompareResultView {
                root: DiffNode {
                    path: "StopPlateMetadata".into(),
                    status: DiffStatus::Modified,
                    left_value: None,
                    right_value: None,
                    summary: "root modified".into(),
                    children: vec![
                        DiffNode {
                            path: "Title".into(),
                            status: DiffStatus::Unchanged,
                            left_value: Some("\"same\"".into()),
                            right_value: Some("\"same\"".into()),
                            summary: "Title unchanged".into(),
                            children: Vec::new(),
                        },
                        DiffNode {
                            path: "LegacyCode".into(),
                            status: DiffStatus::Modified,
                            left_value: Some("\"A1\"".into()),
                            right_value: Some("\"B2\"".into()),
                            summary: "LegacyCode modified".into(),
                            children: Vec::new(),
                        },
                    ],
                },
                change_list: Vec::new(),
                summary: Default::default(),
                selected_path: Some("Title".into()),
            }),
            ..Default::default()
        };

        app.reconcile_tree_selection();

        assert_eq!(
            app.result
                .as_ref()
                .and_then(|result| result.selected_path.as_deref()),
            Some("LegacyCode")
        );
    }
}

#[cfg(test)]
fn png_chunk_crc(kind: [u8; 4], data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in kind.into_iter().chain(data.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}
