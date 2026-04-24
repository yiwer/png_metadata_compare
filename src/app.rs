use crate::batch_report::{
    BatchCompareReport, BatchIssue, DifferentPairResult, MatchedPairCompareResult, UnmatchedSide,
    build_batch_results,
};
use crate::batch_scan::{BatchFileRecord, build_pairing, scan_png_files_best_effort};
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

enum ActiveTreeResult<'a> {
    Single(&'a CompareResultView),
    BatchDifferent(&'a DifferentPairResult),
}

impl<'a> ActiveTreeResult<'a> {
    fn root(&self) -> &'a DiffNode {
        match self {
            Self::Single(result) => &result.root,
            Self::BatchDifferent(result) => &result.diff_root,
        }
    }

    fn summary(&self) -> &'a DiffSummary {
        match self {
            Self::Single(result) => &result.summary,
            Self::BatchDifferent(result) => &result.summary,
        }
    }

    #[cfg(test)]
    fn selected_path(&self) -> Option<&'a str> {
        match self {
            Self::Single(result) => result.selected_path.as_deref(),
            Self::BatchDifferent(result) => result.selected_path.as_deref(),
        }
    }
}

enum ActiveTreeResultMut<'a> {
    Single(&'a mut CompareResultView),
    BatchDifferent(&'a mut DifferentPairResult),
}

impl<'a> ActiveTreeResultMut<'a> {
    fn into_parts(self) -> (&'a DiffNode, &'a DiffSummary, &'a mut Option<String>) {
        match self {
            Self::Single(result) => (&result.root, &result.summary, &mut result.selected_path),
            Self::BatchDifferent(result) => (
                &result.diff_root,
                &result.summary,
                &mut result.selected_path,
            ),
        }
    }
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

struct DirectoryScanSideResult {
    files: Vec<BatchFileRecord>,
    issues: Vec<BatchIssue>,
    root_scan_failed: bool,
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

    fn can_swap_active_inputs(&self) -> bool {
        match self.mode {
            AppMode::SingleFile => self.left_path.is_some() || self.right_path.is_some(),
            AppMode::Directory => self.left_dir.is_some() || self.right_dir.is_some(),
        }
    }

    fn clear_outputs(&mut self) {
        self.result = None;
        self.batch_report = None;
        self.batch_selection = None;
    }

    fn set_left_file_path(&mut self, path: String) {
        self.left_path = Some(path);
        self.clear_outputs();
    }

    fn set_right_file_path(&mut self, path: String) {
        self.right_path = Some(path);
        self.clear_outputs();
    }

    fn set_left_dir_path(&mut self, path: String) {
        self.left_dir = Some(path);
        self.clear_outputs();
    }

    fn set_right_dir_path(&mut self, path: String) {
        self.right_dir = Some(path);
        self.clear_outputs();
    }

    fn swap_active_inputs(&mut self) {
        match self.mode {
            AppMode::SingleFile => std::mem::swap(&mut self.left_path, &mut self.right_path),
            AppMode::Directory => std::mem::swap(&mut self.left_dir, &mut self.right_dir),
        }
        self.clear_outputs();
    }

    fn selected_batch_different(&self) -> Option<&DifferentPairResult> {
        let report = self.batch_report.as_ref()?;
        let BatchSelection::Different(index) = self.batch_selection? else {
            return None;
        };

        report.different.get(index)
    }

    fn selected_batch_different_mut(&mut self) -> Option<&mut DifferentPairResult> {
        let report = self.batch_report.as_mut()?;
        let BatchSelection::Different(index) = self.batch_selection? else {
            return None;
        };

        report.different.get_mut(index)
    }

    fn active_tree_result(&self) -> Option<ActiveTreeResult<'_>> {
        match self.mode {
            AppMode::SingleFile => self.result.as_ref().map(ActiveTreeResult::Single),
            AppMode::Directory => self
                .selected_batch_different()
                .map(ActiveTreeResult::BatchDifferent),
        }
    }

    fn active_tree_result_mut(&mut self) -> Option<ActiveTreeResultMut<'_>> {
        match self.mode {
            AppMode::SingleFile => self.result.as_mut().map(ActiveTreeResultMut::Single),
            AppMode::Directory => self
                .selected_batch_different_mut()
                .map(ActiveTreeResultMut::BatchDifferent),
        }
    }

    fn reconcile_tree_selection(&mut self) {
        let filters = self.filters.clone();

        match self.mode {
            AppMode::SingleFile => {
                if let Some(result) = self.result.as_mut() {
                    tree::reconcile_selected_path(result, &filters);
                }
            }
            AppMode::Directory => {
                if let Some(different) = self.selected_batch_different_mut() {
                    tree::reconcile_selected_path_for(
                        &different.diff_root,
                        &mut different.selected_path,
                        &filters,
                    );
                }
            }
        }
    }

    fn run_active_compare(&mut self) {
        match self.mode {
            AppMode::SingleFile => self.run_compare(),
            AppMode::Directory => self.run_directory_compare(),
        }
    }

    pub fn run_compare(&mut self) {
        let (Some(left_path), Some(right_path)) =
            (self.left_path.as_deref(), self.right_path.as_deref())
        else {
            self.clear_outputs();
            return;
        };

        let result = compare_paths(Path::new(left_path), Path::new(right_path));
        self.clear_outputs();
        self.result = Some(result);
    }

    pub fn run_directory_compare(&mut self) {
        let (Some(left_dir), Some(right_dir)) =
            (self.left_dir.as_deref(), self.right_dir.as_deref())
        else {
            self.clear_outputs();
            return;
        };

        let left_scan = scan_directory_side(Path::new(left_dir), UnmatchedSide::Left);
        let right_scan = scan_directory_side(Path::new(right_dir), UnmatchedSide::Right);
        let batch_report =
            build_directory_compare_report(left_scan, right_scan, compare_matched_pair);

        self.clear_outputs();
        self.batch_selection = default_batch_selection(&batch_report);
        self.batch_report = Some(batch_report);
    }

    fn render_scaffold(&mut self, ctx: &eframe::egui::Context) {
        eframe::egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.heading("PNG Metadata Compare");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Mode");
                ui.selectable_value(&mut self.mode, AppMode::SingleFile, "Single File");
                ui.selectable_value(&mut self.mode, AppMode::Directory, "Directory");
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                match self.mode {
                    AppMode::SingleFile => {
                        if ui.button("Choose left PNG").clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("PNG image", &["png"])
                                .pick_file()
                            {
                                self.set_left_file_path(path.display().to_string());
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
                                self.set_right_file_path(path.display().to_string());
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
                            self.run_active_compare();
                        }
                    }
                    AppMode::Directory => {
                        if ui.button("Choose left directory").clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                self.set_left_dir_path(path.display().to_string());
                            }
                        }
                        ui.label(
                            self.left_dir
                                .as_deref()
                                .unwrap_or("Left directory not selected"),
                        );

                        if ui.button("Choose right directory").clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                self.set_right_dir_path(path.display().to_string());
                            }
                        }
                        ui.label(
                            self.right_dir
                                .as_deref()
                                .unwrap_or("Right directory not selected"),
                        );

                        if ui
                            .add_enabled(
                                self.can_compare(),
                                eframe::egui::Button::new("Compare Directories"),
                            )
                            .clicked()
                        {
                            self.run_active_compare();
                        }
                    }
                }

                if ui
                    .add_enabled(
                        self.can_swap_active_inputs(),
                        eframe::egui::Button::new("Swap"),
                    )
                    .clicked()
                {
                    self.swap_active_inputs();
                }
            });
        });

        eframe::egui::SidePanel::left("summary_panel").show(ctx, |ui| {
            ui.heading("Summary");
            ui.separator();

            match self.mode {
                AppMode::SingleFile => {
                    if let Some(result) = self.result.as_mut() {
                        summary::draw_summary(ui, result);
                    } else {
                        ui.label("Choose two PNG files and run compare to view the summary.");
                    }
                }
                AppMode::Directory => {
                    if let Some(report) = self.batch_report.as_ref() {
                        summary::draw_batch_summary(ui, report, &mut self.batch_selection);
                    } else {
                        ui.label("Choose two directories and run compare to view the summary.");
                    }
                }
            }
        });

        self.reconcile_tree_selection();

        eframe::egui::TopBottomPanel::bottom("detail_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Details");
                ui.separator();
                match self.mode {
                    AppMode::SingleFile => detail::draw_detail(ui, self.result.as_ref()),
                    AppMode::Directory => {
                        if let Some(different) = self.selected_batch_different() {
                            detail::draw_detail_from_parts(
                                ui,
                                &different.diff_root,
                                different.selected_path.as_deref(),
                            );
                        } else {
                            detail::draw_batch_detail(
                                ui,
                                self.batch_report.as_ref(),
                                self.batch_selection,
                            );
                        }
                    }
                }
            });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Diff Tree");
            ui.separator();
            match self.mode {
                AppMode::SingleFile => {
                    self.render_active_diff_tree(
                        ui,
                        "Run compare to populate the diff tree.",
                        "No differences found between the selected PNG metadata.",
                    );
                }
                AppMode::Directory => match self.batch_selection {
                    Some(BatchSelection::Different(_)) => self.render_active_diff_tree(
                        ui,
                        "Run directory compare to populate the diff tree.",
                        "No differences found between the selected PNG metadata.",
                    ),
                    Some(BatchSelection::Identical(_))
                    | Some(BatchSelection::LeftOnly(_))
                    | Some(BatchSelection::RightOnly(_)) => {
                        if self.batch_report.is_none() {
                            ui.label("Run directory compare to populate the diff tree.");
                        } else {
                            ui.label("No diff tree for this item type.");
                        }
                    }
                    None => {
                        if self.batch_report.is_none() {
                            ui.label("Run directory compare to populate the diff tree.");
                        } else {
                            ui.label("Select a batch item to inspect its diff tree.");
                        }
                    }
                },
            }
        });
    }

    fn render_active_diff_tree(
        &mut self,
        ui: &mut eframe::egui::Ui,
        empty_message: &'static str,
        no_differences_message: &'static str,
    ) {
        let state = central_panel_state_parts(
            self.active_tree_result().map(|result| result.root()),
            self.active_tree_result().map(|result| result.summary()),
            &self.filters,
            empty_message,
        );
        if let Some(message) = state.empty_message {
            ui.label(message);
            return;
        }

        let _ = draw_tree_filter_controls(ui, &mut self.filters);
        ui.separator();
        let filters = self.filters.clone();
        self.reconcile_tree_selection();

        match self.mode {
            AppMode::SingleFile => {
                let Some(result) = self.result.as_mut() else {
                    ui.label("Run compare to populate the diff tree.");
                    return;
                };
                let state = central_panel_state(Some(&*result), &filters);
                if state.show_no_differences_message {
                    ui.label(no_differences_message);
                }

                if state.show_tree && state.show_no_differences_message {
                    ui.separator();
                }

                if !state.show_tree {
                    return;
                }

                tree::draw_tree(ui, result, &filters);
            }
            AppMode::Directory => {
                let Some(active_result) = self.active_tree_result_mut() else {
                    ui.label("The selected batch item is no longer available.");
                    return;
                };
                let (root, summary, selected_path) = active_result.into_parts();
                let state =
                    central_panel_state_parts(Some(root), Some(summary), &filters, empty_message);
                if state.show_no_differences_message {
                    ui.label(no_differences_message);
                }

                if state.show_tree && state.show_no_differences_message {
                    ui.separator();
                }

                if !state.show_tree {
                    return;
                }

                tree::draw_tree_from_parts(ui, root, selected_path, &filters);
            }
        }
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
    central_panel_state_parts(
        result.map(|result| &result.root),
        result.map(|result| &result.summary),
        filters,
        "Run compare to populate the diff tree.",
    )
}

fn central_panel_state_parts(
    root: Option<&DiffNode>,
    summary: Option<&DiffSummary>,
    filters: &tree::TreeFilters,
    empty_message: &'static str,
) -> CentralPanelState {
    let (Some(root), Some(summary)) = (root, summary) else {
        return CentralPanelState {
            empty_message: Some(empty_message),
            show_no_differences_message: false,
            show_tree: false,
        };
    };

    if summary.total() == 0 {
        return CentralPanelState {
            empty_message: None,
            show_no_differences_message: true,
            show_tree: tree::should_show(root, filters),
        };
    }

    CentralPanelState {
        empty_message: None,
        show_no_differences_message: false,
        show_tree: true,
    }
}

fn draw_tree_filter_controls(ui: &mut eframe::egui::Ui, filters: &mut tree::TreeFilters) -> bool {
    let mut filters_changed = false;
    ui.horizontal_wrapped(|ui| {
        filters_changed |= ui
            .checkbox(&mut filters.only_differences, "Only differences")
            .changed();
        filters_changed |= ui
            .checkbox(&mut filters.show_reordered, "Show reordered")
            .changed();
        filters_changed |= ui
            .checkbox(&mut filters.show_unchanged, "Show unchanged")
            .changed();
        filters_changed |= ui
            .checkbox(&mut filters.show_errors, "Show errors")
            .changed();
    });

    filters_changed
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

fn scan_directory_side(directory: &Path, side: UnmatchedSide) -> DirectoryScanSideResult {
    let scan = scan_png_files_best_effort(directory);
    DirectoryScanSideResult {
        files: scan.files,
        issues: scan
            .issues
            .into_iter()
            .map(|issue| BatchIssue::ScanFailure {
                side: side.clone(),
                path: issue.path,
                reason: issue.reason,
            })
            .collect(),
        root_scan_failed: scan.root_scan_failed,
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

fn build_directory_compare_report<F>(
    left_scan: DirectoryScanSideResult,
    right_scan: DirectoryScanSideResult,
    mut compare_pair: F,
) -> BatchCompareReport
where
    F: FnMut(crate::batch_report::MatchedPair) -> MatchedPairCompareResult,
{
    let mut pairing = build_pairing(&left_scan.files, &right_scan.files);
    if right_scan.root_scan_failed {
        pairing.left_only.clear();
    }
    if left_scan.root_scan_failed {
        pairing.right_only.clear();
    }

    let matched_results = pairing.matched.into_iter().map(&mut compare_pair).collect();

    let mut issues = left_scan.issues;
    issues.extend(right_scan.issues);

    let mut batch_report = build_batch_results(
        matched_results,
        pairing.left_only,
        pairing.right_only,
        issues,
    );

    for different in &mut batch_report.different {
        different.selected_path = default_selected_path(&different.change_list);
    }

    batch_report
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

        pub(crate) fn write_left_bytes(&self, relative: &str, bytes: &[u8]) {
            self.write_bytes(&self.left, relative, bytes);
        }

        fn write_png(&self, base_dir: &Path, relative: &str, metadata_json: &str) {
            self.write_bytes(
                base_dir,
                relative,
                &png_with_stop_plate_metadata(metadata_json),
            );
        }

        fn write_bytes(&self, base_dir: &Path, relative: &str, bytes: &[u8]) {
            let path = base_dir.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("fixture parent directory should be created");
            }
            fs::write(&path, bytes).expect("fixture file should be written");
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

    pub(crate) fn png_without_stop_plate_metadata() -> Vec<u8> {
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
        bytes.extend_from_slice(&png_chunk_crc(kind, &data).to_be_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::BatchDirFixture;
    use super::{CompareResultView, PngMetadataCompareApp};
    use crate::batch_report::{
        BatchCompareReport, BatchIssue, DifferentPairResult, MatchStrategy, MatchedPair,
        MatchedPairCompareResult, UnmatchedSide,
    };
    use crate::batch_scan::BatchFileRecord;
    use crate::diff::{DiffNode, DiffStatus, DiffSummary};
    use crate::ui::tree::TreeFilters;
    use std::path::PathBuf;

    fn sample_result_view() -> CompareResultView {
        CompareResultView {
            root: DiffNode {
                path: "StopPlateMetadata".into(),
                status: DiffStatus::Modified,
                left_value: None,
                right_value: None,
                summary: "root modified".into(),
                children: Vec::new(),
            },
            change_list: vec![DiffNode {
                path: "Title".into(),
                status: DiffStatus::Modified,
                left_value: Some("\"left\"".into()),
                right_value: Some("\"right\"".into()),
                summary: "Title modified".into(),
                children: Vec::new(),
            }],
            summary: Default::default(),
            selected_path: Some("Title".into()),
        }
    }

    fn sample_record(relative: &str) -> BatchFileRecord {
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

    fn sample_pair(file_name: &str, left: &str, right: &str) -> MatchedPair {
        MatchedPair {
            file_name: file_name.to_string(),
            left: sample_record(left),
            right: sample_record(right),
            match_strategy: MatchStrategy::FileNameAndParentDir,
        }
    }

    fn sample_different_result(
        file_name: &str,
        root_path: &str,
        leaf_path: &str,
    ) -> DifferentPairResult {
        DifferentPairResult {
            pair: sample_pair(
                file_name,
                &format!("left/{file_name}"),
                &format!("right/{file_name}"),
            ),
            diff_root: DiffNode {
                path: root_path.into(),
                status: DiffStatus::Modified,
                left_value: None,
                right_value: None,
                summary: format!("{file_name} root modified"),
                children: vec![DiffNode {
                    path: leaf_path.into(),
                    status: DiffStatus::Modified,
                    left_value: Some("\"left\"".into()),
                    right_value: Some("\"right\"".into()),
                    summary: format!("{file_name} leaf modified"),
                    children: Vec::new(),
                }],
            },
            change_list: vec![DiffNode {
                path: leaf_path.into(),
                status: DiffStatus::Modified,
                left_value: Some("\"left\"".into()),
                right_value: Some("\"right\"".into()),
                summary: format!("{file_name} leaf modified"),
                children: Vec::new(),
            }],
            summary: DiffSummary {
                modified: 1,
                ..DiffSummary::default()
            },
            selected_path: Some(leaf_path.into()),
        }
    }

    fn sample_hidden_selection_result(
        file_name: &str,
        root_path: &str,
        hidden_path: &str,
        visible_path: &str,
    ) -> DifferentPairResult {
        DifferentPairResult {
            pair: sample_pair(
                file_name,
                &format!("left/{file_name}"),
                &format!("right/{file_name}"),
            ),
            diff_root: DiffNode {
                path: root_path.into(),
                status: DiffStatus::Modified,
                left_value: None,
                right_value: None,
                summary: format!("{file_name} root modified"),
                children: vec![
                    DiffNode {
                        path: hidden_path.into(),
                        status: DiffStatus::Unchanged,
                        left_value: Some("\"same\"".into()),
                        right_value: Some("\"same\"".into()),
                        summary: format!("{file_name} hidden leaf unchanged"),
                        children: Vec::new(),
                    },
                    DiffNode {
                        path: visible_path.into(),
                        status: DiffStatus::Modified,
                        left_value: Some("\"left\"".into()),
                        right_value: Some("\"right\"".into()),
                        summary: format!("{file_name} visible leaf modified"),
                        children: Vec::new(),
                    },
                ],
            },
            change_list: vec![DiffNode {
                path: visible_path.into(),
                status: DiffStatus::Modified,
                left_value: Some("\"left\"".into()),
                right_value: Some("\"right\"".into()),
                summary: format!("{file_name} visible leaf modified"),
                children: Vec::new(),
            }],
            summary: DiffSummary {
                modified: 1,
                ..DiffSummary::default()
            },
            selected_path: Some(hidden_path.into()),
        }
    }

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
    fn directory_mode_prefers_selected_batch_diff_over_single_file_result_state() {
        let mut report = BatchCompareReport::default();
        report.different.push(sample_different_result(
            "batch.png",
            "BatchRoot",
            "BatchRoot.Title",
        ));

        let app = PngMetadataCompareApp {
            mode: super::AppMode::Directory,
            result: Some(sample_result_view()),
            batch_report: Some(report),
            batch_selection: Some(super::BatchSelection::Different(0)),
            ..Default::default()
        };

        let active = app
            .active_tree_result()
            .expect("directory mode should expose the selected batch diff");

        assert_eq!(active.root().path, "BatchRoot");
        assert_eq!(active.selected_path(), Some("BatchRoot.Title"));
        assert_eq!(active.summary().modified, 1);
    }

    #[test]
    fn directory_mode_uses_selected_batch_item_diff_state_for_tree_path() {
        let mut report = BatchCompareReport::default();
        report.different.push(sample_different_result(
            "first.png",
            "FirstRoot",
            "FirstRoot.Title",
        ));
        report.different.push(sample_different_result(
            "second.png",
            "SecondRoot",
            "SecondRoot.NewField",
        ));

        let app = PngMetadataCompareApp {
            mode: super::AppMode::Directory,
            batch_report: Some(report),
            batch_selection: Some(super::BatchSelection::Different(1)),
            ..Default::default()
        };

        let active = app
            .active_tree_result()
            .expect("selected batch item should drive the active diff tree state");

        assert_eq!(active.root().path, "SecondRoot");
        assert_eq!(active.selected_path(), Some("SecondRoot.NewField"));
        assert_eq!(active.summary().modified, 1);
    }

    #[test]
    fn render_active_diff_tree_reconciles_current_mode_at_render_time() {
        let mut report = BatchCompareReport::default();
        report.different.push(sample_hidden_selection_result(
            "batch.png",
            "BatchRoot",
            "BatchRoot.Hidden",
            "BatchRoot.Visible",
        ));

        let mut app = PngMetadataCompareApp {
            mode: super::AppMode::SingleFile,
            result: Some(sample_result_view()),
            batch_report: Some(report),
            batch_selection: Some(super::BatchSelection::Different(0)),
            ..Default::default()
        };

        app.reconcile_tree_selection();
        app.mode = super::AppMode::Directory;

        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                app.render_active_diff_tree(
                    ui,
                    "Run directory compare to populate the diff tree.",
                    "No differences found between the selected PNG metadata.",
                );
            });
        });

        assert_eq!(
            app.batch_report
                .as_ref()
                .and_then(|report| report.different.first())
                .and_then(|result| result.selected_path.as_deref()),
            Some("BatchRoot.Visible")
        );
    }

    #[test]
    fn render_active_diff_tree_reconciles_current_batch_selection_at_render_time() {
        let mut report = BatchCompareReport::default();
        report.different.push(sample_different_result(
            "first.png",
            "FirstRoot",
            "FirstRoot.Visible",
        ));
        report.different.push(sample_hidden_selection_result(
            "second.png",
            "SecondRoot",
            "SecondRoot.Hidden",
            "SecondRoot.Visible",
        ));

        let mut app = PngMetadataCompareApp {
            mode: super::AppMode::Directory,
            batch_report: Some(report),
            batch_selection: Some(super::BatchSelection::Different(0)),
            ..Default::default()
        };

        app.reconcile_tree_selection();
        app.batch_selection = Some(super::BatchSelection::Different(1));

        let ctx = eframe::egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                app.render_active_diff_tree(
                    ui,
                    "Run directory compare to populate the diff tree.",
                    "No differences found between the selected PNG metadata.",
                );
            });
        });

        assert_eq!(
            app.batch_report
                .as_ref()
                .and_then(|report| report.different.get(1))
                .and_then(|result| result.selected_path.as_deref()),
            Some("SecondRoot.Visible")
        );
    }

    #[test]
    fn single_file_mode_keeps_existing_active_diff_behavior() {
        let mut report = BatchCompareReport::default();
        report.different.push(sample_different_result(
            "batch.png",
            "BatchRoot",
            "BatchRoot.Title",
        ));

        let app = PngMetadataCompareApp {
            mode: super::AppMode::SingleFile,
            result: Some(sample_result_view()),
            batch_report: Some(report),
            batch_selection: Some(super::BatchSelection::Different(0)),
            ..Default::default()
        };

        let active = app
            .active_tree_result()
            .expect("single-file mode should keep using the single-file result");

        assert_eq!(active.root().path, "StopPlateMetadata");
        assert_eq!(active.selected_path(), Some("Title"));
        assert_eq!(active.summary().modified, 0);
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
    fn default_batch_selection_prefers_different_then_identical_then_left_only_then_right_only() {
        let mut report = BatchCompareReport::default();
        report.different.push(sample_different_result(
            "different.png",
            "DifferentRoot",
            "DifferentRoot.Visible",
        ));
        report
            .identical
            .push(crate::batch_report::IdenticalPairResult {
                pair: sample_pair("identical.png", "left/identical.png", "right/identical.png"),
            });
        report.left_only.push(crate::batch_report::UnmatchedFile {
            side: UnmatchedSide::Left,
            file: sample_record("left/left-only.png"),
            reason: "missing from right".into(),
        });
        report.right_only.push(crate::batch_report::UnmatchedFile {
            side: UnmatchedSide::Right,
            file: sample_record("right/right-only.png"),
            reason: "missing from left".into(),
        });

        assert_eq!(
            super::default_batch_selection(&report),
            Some(super::BatchSelection::Different(0))
        );

        report.different.clear();
        assert_eq!(
            super::default_batch_selection(&report),
            Some(super::BatchSelection::Identical(0))
        );

        report.identical.clear();
        assert_eq!(
            super::default_batch_selection(&report),
            Some(super::BatchSelection::LeftOnly(0))
        );

        report.left_only.clear();
        assert_eq!(
            super::default_batch_selection(&report),
            Some(super::BatchSelection::RightOnly(0))
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
    fn directory_mode_scan_failure_does_not_mark_successful_side_files_as_unmatched() {
        use std::fs;

        let fixture = BatchDirFixture::new("scan_failure_with_pngs");
        fixture.write_left_png("left-only.png", r#"{"Title":"Left only"}"#);
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
        assert!(
            report.left_only.is_empty(),
            "scan failures should suppress left-only classification when the other side scanned successfully: {report:#?}"
        );
        assert!(
            report.right_only.is_empty(),
            "scan failures should suppress right-only classification when the other side scanned successfully: {report:#?}"
        );
        assert_eq!(app.batch_selection, None);
    }

    #[test]
    fn directory_compare_keeps_partial_scan_results_alongside_scan_issues() {
        let left_scan = super::DirectoryScanSideResult {
            files: vec![
                sample_record("left/shared.png"),
                sample_record("left/left-only.png"),
            ],
            issues: vec![BatchIssue::ScanFailure {
                side: UnmatchedSide::Left,
                path: PathBuf::from("left/unreadable"),
                reason: "access denied".into(),
            }],
            root_scan_failed: false,
        };
        let right_scan = super::DirectoryScanSideResult {
            files: vec![
                sample_record("right/shared.png"),
                sample_record("right/right-only.png"),
            ],
            issues: Vec::new(),
            root_scan_failed: false,
        };

        let report = super::build_directory_compare_report(left_scan, right_scan, |pair| {
            if pair.file_name == "shared.png" {
                MatchedPairCompareResult::identical(pair)
            } else {
                panic!("unexpected matched pair in partial scan test: {pair:?}");
            }
        });

        assert_eq!(report.issues.len(), 1);
        assert_eq!(report.identical.len(), 1);
        assert_eq!(report.identical[0].pair.file_name, "shared.png");
        assert!(report.different.is_empty());
        assert_eq!(report.left_only.len(), 1);
        assert_eq!(report.left_only[0].file.file_name, "left-only.png");
        assert_eq!(report.right_only.len(), 1);
        assert_eq!(report.right_only[0].file.file_name, "right-only.png");
    }

    #[test]
    fn directory_mode_keeps_metadata_load_failures_in_different_bucket() {
        let fixture = BatchDirFixture::new("metadata_load_failures");
        fixture.write_left_bytes(
            "missing-metadata.png",
            &super::test_support::png_without_stop_plate_metadata(),
        );
        fixture.write_right_png("missing-metadata.png", r#"{"Title":"Present"}"#);
        fixture.write_left_png("invalid-json.png", "{not-json}");
        fixture.write_right_png("invalid-json.png", r#"{"Title":"Valid"}"#);

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
        assert_eq!(report.different.len(), 2);

        let mut different_names: Vec<&str> = report
            .different
            .iter()
            .map(|different| different.pair.file_name.as_str())
            .collect();
        different_names.sort_unstable();
        assert_eq!(
            different_names,
            vec!["invalid-json.png", "missing-metadata.png"]
        );
        assert!(report.different.iter().all(|different| {
            different.diff_root.status == DiffStatus::Error && different.summary.error >= 1
        }));
        assert!(report.left_only.is_empty());
        assert!(report.right_only.is_empty());
    }

    #[test]
    fn run_active_compare_uses_current_mode_inputs() {
        let fixture = BatchDirFixture::new("active_compare_mode");
        fixture.write_left_png("single-left.png", r#"{"Title":"Left"}"#);
        fixture.write_right_png("single-right.png", r#"{"Title":"Right"}"#);
        fixture.write_left_png("shared.png", r#"{"Title":"Same"}"#);
        fixture.write_right_png("shared.png", r#"{"Title":"Same"}"#);

        let left_file = fixture.left_dir().join("single-left.png");
        let right_file = fixture.right_dir().join("single-right.png");

        let mut app = PngMetadataCompareApp {
            left_path: Some(left_file.display().to_string()),
            right_path: Some(right_file.display().to_string()),
            left_dir: Some(fixture.left_dir().display().to_string()),
            right_dir: Some(fixture.right_dir().display().to_string()),
            ..Default::default()
        };

        app.run_active_compare();

        assert!(app.result.is_some());
        assert!(app.batch_report.is_none());
        assert!(app.batch_selection.is_none());

        app.mode = super::AppMode::Directory;
        app.run_active_compare();

        let report = app
            .batch_report
            .as_ref()
            .expect("directory mode compare should store batch report");
        assert!(app.result.is_none());
        assert_eq!(report.identical.len(), 1);
        assert_eq!(report.identical[0].pair.file_name, "shared.png");
    }

    #[test]
    fn input_changes_clear_stale_cross_mode_state() {
        let mut app = PngMetadataCompareApp {
            result: Some(sample_result_view()),
            batch_report: Some(BatchCompareReport::default()),
            batch_selection: Some(super::BatchSelection::LeftOnly(0)),
            ..Default::default()
        };

        app.set_left_file_path("left.png".into());

        assert_eq!(app.left_path.as_deref(), Some("left.png"));
        assert!(app.result.is_none());
        assert!(app.batch_report.is_none());
        assert!(app.batch_selection.is_none());

        app.result = Some(sample_result_view());
        app.batch_report = Some(BatchCompareReport::default());
        app.batch_selection = Some(super::BatchSelection::RightOnly(0));

        app.set_right_dir_path("right-dir".into());

        assert_eq!(app.right_dir.as_deref(), Some("right-dir"));
        assert!(app.result.is_none());
        assert!(app.batch_report.is_none());
        assert!(app.batch_selection.is_none());

        app.result = Some(sample_result_view());
        app.batch_report = Some(BatchCompareReport::default());
        app.batch_selection = Some(super::BatchSelection::LeftOnly(0));

        app.set_left_dir_path("left-dir".into());

        assert_eq!(app.left_dir.as_deref(), Some("left-dir"));
        assert!(app.result.is_none());
        assert!(app.batch_report.is_none());
        assert!(app.batch_selection.is_none());
    }

    #[test]
    fn swap_active_inputs_respects_mode_and_clears_state() {
        let mut app = PngMetadataCompareApp {
            left_path: Some("left.png".into()),
            right_path: Some("right.png".into()),
            left_dir: Some("left-dir".into()),
            right_dir: Some("right-dir".into()),
            result: Some(sample_result_view()),
            batch_report: Some(BatchCompareReport::default()),
            batch_selection: Some(super::BatchSelection::LeftOnly(0)),
            ..Default::default()
        };

        app.swap_active_inputs();

        assert_eq!(app.left_path.as_deref(), Some("right.png"));
        assert_eq!(app.right_path.as_deref(), Some("left.png"));
        assert_eq!(app.left_dir.as_deref(), Some("left-dir"));
        assert_eq!(app.right_dir.as_deref(), Some("right-dir"));
        assert!(app.result.is_none());
        assert!(app.batch_report.is_none());
        assert!(app.batch_selection.is_none());

        app.mode = super::AppMode::Directory;
        app.result = Some(sample_result_view());
        app.batch_report = Some(BatchCompareReport::default());
        app.batch_selection = Some(super::BatchSelection::RightOnly(0));

        app.swap_active_inputs();

        assert_eq!(app.left_path.as_deref(), Some("right.png"));
        assert_eq!(app.right_path.as_deref(), Some("left.png"));
        assert_eq!(app.left_dir.as_deref(), Some("right-dir"));
        assert_eq!(app.right_dir.as_deref(), Some("left-dir"));
        assert!(app.result.is_none());
        assert!(app.batch_report.is_none());
        assert!(app.batch_selection.is_none());
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
