use crate::batch_report::{MatchStrategy, MatchedPair, UnmatchedSide};
use crate::batch_scan::scan_png_files_best_effort;
use crate::diff::{DiffNode, DiffSummary, compare_metadata, flatten_changes, summarize_changes};
use crate::error::{CompareError, UiError};
use crate::metadata::{MetadataLoadResult, load_metadata};
use crate::png_reader::extract_stop_plate_metadata_from_file;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SideInspection {
    pub side: &'static str,
    pub file_path: String,
    pub file_name: String,
    pub raw_json: Option<String>,
    pub metadata: Option<Value>,
    pub error: Option<UiError>,
}

struct LoadedSide {
    inspection: SideInspection,
    metadata_result: MetadataLoadResult,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PairInspection {
    pub left: SideInspection,
    pub right: SideInspection,
    pub diff_root: DiffNode,
    pub diff_summary: DiffSummary,
    pub default_selected_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchListItemKind {
    Identical,
    Different,
    LeftOnly,
    RightOnly,
    Error,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct BatchCounts {
    pub identical: usize,
    pub different: usize,
    pub left_only: usize,
    pub right_only: usize,
    pub error: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BatchListItem {
    pub id: String,
    pub kind: BatchListItemKind,
    pub label: String,
    pub left_path: Option<String>,
    pub right_path: Option<String>,
    pub difference_count: usize,
    pub match_strategy: Option<MatchStrategy>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DirectorySummary {
    pub counts: BatchCounts,
    pub items: Vec<BatchListItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStage {
    Scanning,
    Comparing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ScanProgress {
    pub stage: ScanStage,
    pub done: usize,
    pub total: usize,
}

pub fn inspect_pair(left_path: &Path, right_path: &Path) -> PairInspection {
    let left = load_side("left", left_path);
    let right = load_side("right", right_path);
    let diff_root = compare_metadata(&left.metadata_result, &right.metadata_result);
    let change_list = flatten_changes(&diff_root);
    let diff_summary = summarize_service_changes(&change_list);
    let default_selected_path = default_selected_path(&change_list);

    PairInspection {
        left: left.inspection,
        right: right.inspection,
        diff_root,
        diff_summary,
        default_selected_path,
    }
}

pub fn inspect_single_side(path: &Path, side: UnmatchedSide) -> SideInspection {
    let side = match side {
        UnmatchedSide::Left => "left",
        UnmatchedSide::Right => "right",
    };

    load_side(side, path).inspection
}

pub fn scan_directory_summary(left_dir: &Path, right_dir: &Path) -> DirectorySummary {
    scan_directory_summary_with_progress(left_dir, right_dir, |_| {})
}

pub fn scan_directory_summary_with_progress<F>(
    left_dir: &Path,
    right_dir: &Path,
    progress: F,
) -> DirectorySummary
where
    F: Fn(ScanProgress) + Sync,
{
    progress(ScanProgress {
        stage: ScanStage::Scanning,
        done: 0,
        total: 0,
    });
    let left_scan = scan_png_files_best_effort(left_dir);
    let right_scan = scan_png_files_best_effort(right_dir);
    let mut counts = BatchCounts::default();
    let mut items = Vec::new();
    let mut pairing = crate::batch_scan::build_pairing(&left_scan.files, &right_scan.files);

    if right_scan.root_scan_failed {
        pairing.left_only.clear();
    }
    if left_scan.root_scan_failed {
        pairing.right_only.clear();
    }

    let pairs = pairing.matched;
    let total = pairs.len();
    progress(ScanProgress {
        stage: ScanStage::Comparing,
        done: 0,
        total,
    });
    let summaries = compare_pairs_parallel(&pairs, &|done| {
        progress(ScanProgress {
            stage: ScanStage::Comparing,
            done,
            total,
        });
    });

    for (pair, diff_summary) in pairs.into_iter().zip(summaries) {
        let difference_count = diff_summary.total();
        let kind = if difference_count == 0 {
            counts.identical += 1;
            BatchListItemKind::Identical
        } else {
            counts.different += 1;
            BatchListItemKind::Different
        };

        items.push(BatchListItem {
            id: format!(
                "{}::{}::{}",
                pair.file_name,
                pair.left.relative_path.display(),
                pair.right.relative_path.display()
            ),
            kind,
            label: pair.file_name,
            left_path: Some(pair.left.absolute_path.display().to_string()),
            right_path: Some(pair.right.absolute_path.display().to_string()),
            difference_count,
            match_strategy: Some(pair.match_strategy),
            message: None,
        });
    }

    for unmatched in pairing.left_only {
        counts.left_only += 1;
        items.push(BatchListItem {
            id: format!("left-only::{}", unmatched.file.absolute_path.display()),
            kind: BatchListItemKind::LeftOnly,
            label: unmatched.file.file_name,
            left_path: Some(unmatched.file.absolute_path.display().to_string()),
            right_path: None,
            difference_count: 0,
            match_strategy: None,
            message: Some(unmatched.reason),
        });
    }

    for unmatched in pairing.right_only {
        counts.right_only += 1;
        items.push(BatchListItem {
            id: format!("right-only::{}", unmatched.file.absolute_path.display()),
            kind: BatchListItemKind::RightOnly,
            label: unmatched.file.file_name,
            left_path: None,
            right_path: Some(unmatched.file.absolute_path.display().to_string()),
            difference_count: 0,
            match_strategy: None,
            message: Some(unmatched.reason),
        });
    }

    for issue in left_scan.issues.into_iter().chain(right_scan.issues) {
        counts.error += 1;
        items.push(BatchListItem {
            id: format!("issue::{}", issue.path.display()),
            kind: BatchListItemKind::Error,
            label: issue.path.display().to_string(),
            left_path: None,
            right_path: None,
            difference_count: 0,
            match_strategy: None,
            message: Some(issue.reason),
        });
    }

    DirectorySummary { counts, items }
}

fn load_side(side: &'static str, path: &Path) -> LoadedSide {
    side_from_raw(side, path, extract_stop_plate_metadata_from_file(path))
}

/// Compares matched pairs across worker threads, preserving input order in the
/// returned summaries. `on_done` is called once per completed pair with the
/// running completion count (completion order, not input order).
fn compare_pairs_parallel<F>(pairs: &[MatchedPair], on_done: &F) -> Vec<DiffSummary>
where
    F: Fn(usize) + Sync,
{
    let total = pairs.len();
    if total == 0 {
        return Vec::new();
    }

    let worker_count = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(4)
        .min(total);
    let next_index = AtomicUsize::new(0);
    let completed = AtomicUsize::new(0);
    let results: Vec<Mutex<Option<DiffSummary>>> = (0..total).map(|_| Mutex::new(None)).collect();

    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            scope.spawn(|| {
                loop {
                    let index = next_index.fetch_add(1, Ordering::Relaxed);
                    if index >= total {
                        break;
                    }
                    let pair = &pairs[index];
                    let summary =
                        compare_pair_summary(&pair.left.absolute_path, &pair.right.absolute_path);
                    *results[index].lock().expect("result slot lock poisoned") = Some(summary);
                    let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    on_done(done);
                }
            });
        }
    });

    results
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .expect("result slot lock poisoned")
                .expect("every pair is compared by a worker")
        })
        .collect()
}

fn compare_pair_summary(left_path: &Path, right_path: &Path) -> DiffSummary {
    let left_raw = extract_stop_plate_metadata_from_file(left_path);
    let right_raw = extract_stop_plate_metadata_from_file(right_path);
    let left_metadata = load_metadata(left_raw);
    let right_metadata = load_metadata(right_raw);
    let diff_root = compare_metadata(&left_metadata, &right_metadata);
    let change_list = flatten_changes(&diff_root);
    summarize_service_changes(&change_list)
}

fn side_from_raw(side: &'static str, path: &Path, raw: Result<String, CompareError>) -> LoadedSide {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    match raw {
        Ok(raw_json) => match load_metadata(Ok(raw_json.clone())) {
            MetadataLoadResult::Parsed(metadata) => LoadedSide {
                inspection: SideInspection {
                    side,
                    file_path: path.display().to_string(),
                    file_name,
                    raw_json: Some(raw_json),
                    metadata: Some(metadata.clone()),
                    error: None,
                },
                metadata_result: MetadataLoadResult::Parsed(metadata),
            },
            MetadataLoadResult::Error(error) => LoadedSide {
                inspection: SideInspection {
                    side,
                    file_path: path.display().to_string(),
                    file_name,
                    raw_json: Some(raw_json),
                    metadata: None,
                    error: Some(error.to_ui_error()),
                },
                metadata_result: MetadataLoadResult::Error(error),
            },
        },
        Err(error) => LoadedSide {
            inspection: SideInspection {
                side,
                file_path: path.display().to_string(),
                file_name,
                raw_json: None,
                metadata: None,
                error: Some(error.to_ui_error()),
            },
            metadata_result: MetadataLoadResult::Error(error),
        },
    }
}

fn default_selected_path(change_list: &[DiffNode]) -> Option<String> {
    change_list
        .iter()
        .find(|node| node.left_value.is_some() || node.right_value.is_some())
        .or_else(|| change_list.first())
        .map(|node| node.path.clone())
}

fn summarize_service_changes(change_list: &[DiffNode]) -> DiffSummary {
    let filtered: Vec<DiffNode> = change_list
        .iter()
        .filter(|node| !is_aggregate_only_change(node, change_list))
        .cloned()
        .collect();

    summarize_changes(&filtered)
}

fn is_aggregate_only_change(node: &DiffNode, change_list: &[DiffNode]) -> bool {
    if node.left_value.is_some() || node.right_value.is_some() {
        return false;
    }

    let path = node.path.as_str();
    if path == "StopPlateMetadata" {
        return change_list.iter().any(|other| other.path != path);
    }

    change_list
        .iter()
        .any(|other| other.path != path && is_descendant_path(path, &other.path))
}

fn is_descendant_path(parent: &str, child: &str) -> bool {
    child
        .strip_prefix(parent)
        .is_some_and(|suffix| suffix.starts_with('.') || suffix.starts_with('['))
}

#[cfg(test)]
mod tests {
    use super::{
        BatchListItemKind, ScanProgress, ScanStage, compare_pair_summary, inspect_pair,
        inspect_single_side, scan_directory_summary, scan_directory_summary_with_progress,
    };
    use crate::batch_report::{MatchStrategy, UnmatchedSide};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn inspect_pair_returns_raw_json_and_preview_paths() {
        let fixture = TestFixture::new("pair");
        let left = fixture.write_png("left.png", r#"{"Title":"Left"}"#);
        let right = fixture.write_png("right.png", r#"{"Title":"Right"}"#);

        let payload = inspect_pair(&left, &right);

        assert_eq!(payload.left.file_path, left.display().to_string());
        assert_eq!(payload.right.file_path, right.display().to_string());
        assert_eq!(payload.left.raw_json.as_deref(), Some(r#"{"Title":"Left"}"#));
        assert_eq!(payload.right.raw_json.as_deref(), Some(r#"{"Title":"Right"}"#));
        assert_eq!(payload.diff_summary.modified, 1);
        assert!(payload.default_selected_path.is_some());
    }

    #[test]
    fn inspect_single_side_surfaces_missing_side_context() {
        let fixture = TestFixture::new("single_side");
        let left = fixture.write_png("left-only.png", r#"{"Title":"Solo"}"#);

        let payload = inspect_single_side(&left, UnmatchedSide::Left);

        assert_eq!(payload.side, "left");
        assert_eq!(payload.file_path, left.display().to_string());
        assert_eq!(payload.raw_json.as_deref(), Some(r#"{"Title":"Solo"}"#));
        assert!(payload.error.is_none());
    }

    #[test]
    fn scan_directory_summary_classifies_results_without_full_diff_payloads() {
        let fixture = BatchFixture::new("summary");
        fixture.write_left_png("same.png", "shared", r#"{"Title":"Same"}"#);
        fixture.write_right_png("same.png", "shared", r#"{"Title":"Same"}"#);
        fixture.write_left_png("diff.png", "shared", r#"{"Title":"Left"}"#);
        fixture.write_right_png("diff.png", "shared", r#"{"Title":"Right"}"#);
        fixture.write_left_png("left-only.png", "shared", r#"{"Title":"Only"}"#);

        let payload = scan_directory_summary(fixture.left_dir(), fixture.right_dir());

        assert_eq!(payload.counts.identical, 1);
        assert_eq!(payload.counts.different, 1);
        assert_eq!(payload.counts.left_only, 1);
        assert!(payload
            .items
            .iter()
            .any(|item| item.kind == BatchListItemKind::Different));
        assert!(payload
            .items
            .iter()
            .any(|item| item.kind == BatchListItemKind::LeftOnly));
    }

    #[test]
    fn scan_directory_summary_counts_each_scan_issue_once() {
        let fixture = BatchFixture::new("scan_issue");
        fixture.write_left_png("same.png", "shared", r#"{"Title":"Same"}"#);
        fixture.write_right_png("same.png", "shared", r#"{"Title":"Same"}"#);
        let invalid_right = fixture.root.join("not-a-directory.png");
        fs::write(&invalid_right, b"not a directory").unwrap();

        let payload = scan_directory_summary(fixture.left_dir(), &invalid_right);

        assert_eq!(payload.counts.error, 1);
        assert_eq!(
            payload
                .items
                .iter()
                .filter(|item| item.kind == BatchListItemKind::Error)
                .count(),
            1
        );
    }

    #[test]
    fn scan_directory_summary_with_progress_reports_each_compared_pair() {
        let fixture = BatchFixture::new("progress");
        fixture.write_left_png("a.png", "shared", r#"{"Title":"A"}"#);
        fixture.write_right_png("a.png", "shared", r#"{"Title":"A"}"#);
        fixture.write_left_png("b.png", "shared", r#"{"Title":"Left"}"#);
        fixture.write_right_png("b.png", "shared", r#"{"Title":"Right"}"#);
        fixture.write_left_png("c.png", "shared", r#"{"Title":"C"}"#);
        fixture.write_right_png("c.png", "shared", r#"{"Title":"C"}"#);

        let events = std::sync::Mutex::new(Vec::new());
        let payload =
            scan_directory_summary_with_progress(fixture.left_dir(), fixture.right_dir(), |p| {
                events.lock().unwrap().push(p);
            });
        let events = events.into_inner().unwrap();

        assert_eq!(
            events.first().copied(),
            Some(ScanProgress {
                stage: ScanStage::Scanning,
                done: 0,
                total: 0
            })
        );

        let comparing: Vec<ScanProgress> = events
            .iter()
            .filter(|event| event.stage == ScanStage::Comparing)
            .copied()
            .collect();
        assert_eq!(
            comparing.first().map(|event| (event.done, event.total)),
            Some((0, 3))
        );
        assert!(comparing.iter().all(|event| event.total == 3));
        // Pairs are compared concurrently, so completion events may arrive in any
        // order — but each pair must be reported exactly once.
        let mut done_counts: Vec<usize> = comparing
            .iter()
            .skip(1)
            .map(|event| event.done)
            .collect();
        done_counts.sort_unstable();
        assert_eq!(done_counts, vec![1, 2, 3]);

        assert_eq!(
            payload,
            scan_directory_summary(fixture.left_dir(), fixture.right_dir())
        );
        assert_eq!(payload.counts.identical, 2);
        assert_eq!(payload.counts.different, 1);
    }

    #[test]
    fn compare_pair_summary_matches_pair_inspection_diff_summary() {
        let fixture = TestFixture::new("pair_summary");
        let left = fixture.write_png("left.png", r#"{"Title":"Left"}"#);
        let right = fixture.write_png("right.png", r#"{"Title":"Right"}"#);

        let inspection = inspect_pair(&left, &right);
        let summary = compare_pair_summary(&left, &right);

        assert_eq!(summary, inspection.diff_summary);
    }

    #[test]
    fn scan_directory_summary_uses_typed_match_strategy() {
        let fixture = BatchFixture::new("typed_strategy");
        fixture.write_left_png("dup.png", "route-a", r#"{"Title":"Left A"}"#);
        fixture.write_left_png("dup.png", "route-b", r#"{"Title":"Left B"}"#);
        fixture.write_right_png("dup.png", "route-b", r#"{"Title":"Right B"}"#);
        fixture.write_right_png("dup.png", "route-a", r#"{"Title":"Right A"}"#);

        let payload = scan_directory_summary(fixture.left_dir(), fixture.right_dir());

        assert!(payload.items.iter().all(|item| {
            item.kind != BatchListItemKind::Different
                || item.match_strategy == Some(MatchStrategy::FileNameAndParentDir)
        }));
    }

    struct TestFixture {
        root: PathBuf,
    }

    impl TestFixture {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "png_metadata_compare_inspection_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn write_png(&self, name: &str, json: &str) -> PathBuf {
            let path = self.root.join(name);
            fs::write(&path, png_with_metadata(json)).unwrap();
            path
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct BatchFixture {
        root: PathBuf,
        left: PathBuf,
        right: PathBuf,
    }

    impl BatchFixture {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "png_metadata_compare_batch_summary_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            let left = root.join("left");
            let right = root.join("right");
            fs::create_dir_all(&left).unwrap();
            fs::create_dir_all(&right).unwrap();
            Self { root, left, right }
        }

        fn left_dir(&self) -> &Path {
            &self.left
        }

        fn right_dir(&self) -> &Path {
            &self.right
        }

        fn write_left_png(&self, name: &str, dir: &str, json: &str) {
            write_batch_png(&self.left, name, dir, json);
        }

        fn write_right_png(&self, name: &str, dir: &str, json: &str) {
            write_batch_png(&self.right, name, dir, json);
        }
    }

    impl Drop for BatchFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write_batch_png(root: &Path, name: &str, dir: &str, json: &str) {
        let folder = root.join(dir);
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join(name), png_with_metadata(json)).unwrap();
    }

    fn png_with_metadata(json: &str) -> Vec<u8> {
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
}
