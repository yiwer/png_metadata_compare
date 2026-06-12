use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::batch_report::{
    MatchStrategy, MatchedPair, PairingResult, UnmatchedFile, UnmatchedSide,
};
use crate::error::CompareError;
use crate::pairing_key::canonical_pairing_key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PairingKeyKind {
    Canonical,
    RawFileName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
// Staged API for upcoming batch-compare wiring in later tasks.
#[allow(dead_code)]
pub struct BatchFileRecord {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_name: String,
    pub parent_dir_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchScanIssue {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BestEffortScanResult {
    pub files: Vec<BatchFileRecord>,
    pub issues: Vec<BatchScanIssue>,
    pub root_scan_failed: bool,
}

// Staged API for upcoming batch-compare wiring in later tasks.
#[allow(dead_code)]
pub fn scan_png_files(root: &Path) -> Result<Vec<BatchFileRecord>, CompareError> {
    let root_absolute = resolve_root_absolute(root)?;

    let mut records = Vec::new();
    walk_png_files(&root_absolute, &root_absolute, &mut records)?;
    records.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(records)
}

pub fn scan_png_files_best_effort(root: &Path) -> BestEffortScanResult {
    let root_absolute = match resolve_root_absolute(root) {
        Ok(root_absolute) => root_absolute,
        Err(error) => {
            return BestEffortScanResult {
                files: Vec::new(),
                issues: vec![scan_issue_from_error(root, &error)],
                root_scan_failed: true,
            };
        }
    };

    let mut result = BestEffortScanResult::default();
    result.root_scan_failed = !walk_png_files_best_effort(
        &root_absolute,
        &root_absolute,
        &mut result.files,
        &mut result.issues,
    );
    result
        .files
        .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    result
}

// Staged API for upcoming batch-compare wiring in later tasks.
#[allow(dead_code)]
pub fn build_pairing(left: &[BatchFileRecord], right: &[BatchFileRecord]) -> PairingResult {
    let mut result = PairingResult::default();
    let mut left_by_file_name = group_by_pairing_key(left);
    let mut right_by_file_name = group_by_pairing_key(right);
    let mut file_names = BTreeSet::new();

    file_names.extend(left_by_file_name.keys().cloned());
    file_names.extend(right_by_file_name.keys().cloned());

    for file_name in file_names {
        let key_kind = pairing_key_kind(&file_name);
        let left_group = left_by_file_name.remove(&file_name).unwrap_or_default();
        let right_group = right_by_file_name.remove(&file_name).unwrap_or_default();
        let display_file_name = display_file_name(&left_group, &right_group, &file_name);

        if left_group.len() == 1 && right_group.len() == 1 {
            result.matched.push(MatchedPair {
                file_name: display_file_name.clone(),
                left: left_group[0].clone(),
                right: right_group[0].clone(),
                match_strategy: match key_kind {
                    PairingKeyKind::Canonical => MatchStrategy::CanonicalStopId,
                    PairingKeyKind::RawFileName => MatchStrategy::FileName,
                },
            });
            continue;
        }

        if left_group.is_empty() {
            add_unmatched_files(
                &mut result.right_only,
                UnmatchedSide::Right,
                right_group,
                format!("no file named '{display_file_name}' found on left side"),
            );
            continue;
        }

        if right_group.is_empty() {
            add_unmatched_files(
                &mut result.left_only,
                UnmatchedSide::Left,
                left_group,
                format!("no file named '{display_file_name}' found on right side"),
            );
            continue;
        }

        resolve_duplicate_file_name_group(&display_file_name, left_group, right_group, &mut result);
    }

    result.matched.sort_by(|left, right| {
        left.file_name
            .cmp(&right.file_name)
            .then(left.left.relative_path.cmp(&right.left.relative_path))
            .then(left.right.relative_path.cmp(&right.right.relative_path))
    });
    result.left_only.sort_by(|left, right| {
        left.file
            .file_name
            .cmp(&right.file.file_name)
            .then(left.file.relative_path.cmp(&right.file.relative_path))
    });
    result.right_only.sort_by(|left, right| {
        left.file
            .file_name
            .cmp(&right.file.file_name)
            .then(left.file.relative_path.cmp(&right.file.relative_path))
    });

    result
}

fn group_by_pairing_key(records: &[BatchFileRecord]) -> HashMap<String, Vec<BatchFileRecord>> {
    let mut groups = HashMap::new();

    for record in records {
        let key = canonical_pairing_key(&record.file_name)
            .unwrap_or_else(|| normalize_pairing_component(&record.file_name));
        groups.entry(key).or_insert_with(Vec::new).push(record.clone());
    }

    groups
}

fn pairing_key_kind(key: &str) -> PairingKeyKind {
    // Canonical keys (bus-stop `站点名|方位|序号` and insert-strip
    // `站点名|方位|插片<n>|序号`) always contain the `|` separator, which
    // cannot appear in a normalized raw file name, so its presence is a
    // sufficient discriminator.
    if key.contains('|') {
        PairingKeyKind::Canonical
    } else {
        PairingKeyKind::RawFileName
    }
}

fn resolve_duplicate_file_name_group(
    file_name: &str,
    left_group: Vec<BatchFileRecord>,
    right_group: Vec<BatchFileRecord>,
    result: &mut PairingResult,
) {
    let mut left_by_parent = group_by_parent_dir(left_group);
    let mut right_by_parent = group_by_parent_dir(right_group);
    let mut parent_keys = BTreeSet::new();

    parent_keys.extend(left_by_parent.keys().cloned());
    parent_keys.extend(right_by_parent.keys().cloned());

    for parent_key in parent_keys {
        let left_candidates = left_by_parent.remove(&parent_key).unwrap_or_default();
        let right_candidates = right_by_parent.remove(&parent_key).unwrap_or_default();
        let display_parent_dir = display_parent_dir_name(&left_candidates, &right_candidates);

        if left_candidates.len() == 1 && right_candidates.len() == 1 {
            result.matched.push(MatchedPair {
                file_name: file_name.to_string(),
                left: left_candidates[0].clone(),
                right: right_candidates[0].clone(),
                match_strategy: MatchStrategy::FileNameAndParentDir,
            });
            continue;
        }

        if left_candidates.is_empty() {
            add_unmatched_files(
                &mut result.right_only,
                UnmatchedSide::Right,
                right_candidates,
                format!(
                    "duplicate file name '{file_name}' has no left-side counterpart in parent directory '{}'",
                    display_parent_dir
                ),
            );
            continue;
        }

        if right_candidates.is_empty() {
            add_unmatched_files(
                &mut result.left_only,
                UnmatchedSide::Left,
                left_candidates,
                format!(
                    "duplicate file name '{file_name}' has no right-side counterpart in parent directory '{}'",
                    display_parent_dir
                ),
            );
            continue;
        }

        let reason = format!(
            "ambiguous duplicate file name group for '{file_name}' after parent directory refinement"
        );
        add_unmatched_files(
            &mut result.left_only,
            UnmatchedSide::Left,
            left_candidates,
            reason.clone(),
        );
        add_unmatched_files(
            &mut result.right_only,
            UnmatchedSide::Right,
            right_candidates,
            reason,
        );
    }
}

fn group_by_parent_dir(
    records: Vec<BatchFileRecord>,
) -> HashMap<Option<String>, Vec<BatchFileRecord>> {
    let mut groups = HashMap::new();

    for record in records {
        groups
            .entry(
                record
                    .parent_dir_name
                    .as_ref()
                    .map(|name| normalize_pairing_component(name)),
            )
            .or_insert_with(Vec::new)
            .push(record);
    }

    groups
}

fn add_unmatched_files(
    target: &mut Vec<UnmatchedFile>,
    side: UnmatchedSide,
    files: Vec<BatchFileRecord>,
    reason: String,
) {
    for file in files {
        target.push(UnmatchedFile {
            side: side.clone(),
            file,
            reason: reason.clone(),
        });
    }
}

fn resolve_root_absolute(root: &Path) -> Result<PathBuf, CompareError> {
    if root.is_absolute() {
        Ok(root.to_path_buf())
    } else {
        root.canonicalize().map_err(|err| CompareError::FileRead {
            path: root.to_path_buf(),
            reason: err.to_string(),
        })
    }
}

fn walk_png_files(
    root: &Path,
    directory: &Path,
    records: &mut Vec<BatchFileRecord>,
) -> Result<(), CompareError> {
    let entries = fs::read_dir(directory).map_err(|err| CompareError::FileRead {
        path: directory.to_path_buf(),
        reason: err.to_string(),
    })?;

    for entry_result in entries {
        let entry = entry_result.map_err(|err| CompareError::FileRead {
            path: directory.to_path_buf(),
            reason: err.to_string(),
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|err| CompareError::FileRead {
            path: path.clone(),
            reason: err.to_string(),
        })?;

        if file_type.is_dir() {
            walk_png_files(root, &path, records)?;
            continue;
        }

        if !file_type.is_file() || !is_png_path(&path) {
            continue;
        }

        records.push(record_png_file(root, path)?);
    }

    Ok(())
}

fn walk_png_files_best_effort(
    root: &Path,
    directory: &Path,
    records: &mut Vec<BatchFileRecord>,
    issues: &mut Vec<BatchScanIssue>,
) -> bool {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(err) => {
            issues.push(BatchScanIssue {
                path: directory.to_path_buf(),
                reason: err.to_string(),
            });
            return false;
        }
    };

    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                issues.push(BatchScanIssue {
                    path: directory.to_path_buf(),
                    reason: err.to_string(),
                });
                continue;
            }
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(err) => {
                issues.push(BatchScanIssue {
                    path: path.clone(),
                    reason: err.to_string(),
                });
                continue;
            }
        };

        if file_type.is_dir() {
            walk_png_files_best_effort(root, &path, records, issues);
            continue;
        }

        if !file_type.is_file() || !is_png_path(&path) {
            continue;
        }

        match record_png_file(root, path) {
            Ok(record) => records.push(record),
            Err(error) => issues.push(scan_issue_from_error(root, &error)),
        }
    }

    true
}

fn record_png_file(root: &Path, path: PathBuf) -> Result<BatchFileRecord, CompareError> {
    let relative_path = path
        .strip_prefix(root)
        .map_err(|err| CompareError::FileRead {
            path: path.clone(),
            reason: err.to_string(),
        })?
        .to_path_buf();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CompareError::FileRead {
            path: path.clone(),
            reason: "file name is not valid UTF-8".to_string(),
        })?
        .to_string();
    let parent_dir_name = relative_path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().to_string());

    Ok(BatchFileRecord {
        absolute_path: path,
        relative_path,
        file_name,
        parent_dir_name,
    })
}

fn scan_issue_from_error(fallback_path: &Path, error: &CompareError) -> BatchScanIssue {
    match error {
        CompareError::FileRead { path, reason } => BatchScanIssue {
            path: path.clone(),
            reason: reason.clone(),
        },
        _ => BatchScanIssue {
            path: fallback_path.to_path_buf(),
            reason: error.to_string(),
        },
    }
}

fn normalize_pairing_component(value: &str) -> String {
    if cfg!(windows) {
        value.to_lowercase()
    } else {
        value.to_string()
    }
}

fn display_file_name(
    left_group: &[BatchFileRecord],
    right_group: &[BatchFileRecord],
    normalized_file_name: &str,
) -> String {
    left_group
        .first()
        .or_else(|| right_group.first())
        .map(|record| record.file_name.clone())
        .unwrap_or_else(|| normalized_file_name.to_string())
}

fn display_parent_dir_name(
    left_group: &[BatchFileRecord],
    right_group: &[BatchFileRecord],
) -> String {
    left_group
        .iter()
        .chain(right_group.iter())
        .find_map(|record| record.parent_dir_name.clone())
        .unwrap_or_else(|| "<root>".to_string())
}

fn is_png_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("png"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{BatchFileRecord, build_pairing, scan_png_files};
    use crate::batch_report::{MatchStrategy, UnmatchedSide};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "png_metadata_compare_batch_scan_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_file(root: &Path, relative: &str) {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("test parent directory should be created");
        }
        fs::write(path, b"test").expect("test file should be written");
    }

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

    #[test]
    fn recursively_finds_png_files_and_ignores_non_png_files() {
        let fixture = TestDir::new("recursive_pngs");
        write_file(fixture.path(), "root.png");
        write_file(fixture.path(), "nested/deep/second.PnG");
        write_file(fixture.path(), "nested/ignore.txt");
        write_file(fixture.path(), "not_png.jpg");

        let records = scan_png_files(fixture.path()).expect("scan should succeed");
        let relative_paths: Vec<PathBuf> = records
            .iter()
            .map(|record| record.relative_path.clone())
            .collect();
        assert_eq!(
            relative_paths,
            vec![
                PathBuf::from("nested").join("deep").join("second.PnG"),
                PathBuf::from("root.png")
            ]
        );
    }

    #[test]
    fn captures_immediate_parent_directory_name_for_nested_png_files() {
        let fixture = TestDir::new("parent_dir");
        write_file(fixture.path(), "a/b/c/image.png");

        let records = scan_png_files(fixture.path()).expect("scan should succeed");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].absolute_path,
            fixture
                .path()
                .join("a")
                .join("b")
                .join("c")
                .join("image.png")
        );
        assert_eq!(records[0].file_name, "image.png");
        assert_eq!(records[0].parent_dir_name.as_deref(), Some("c"));
    }

    #[test]
    fn root_level_png_has_no_parent_directory_name() {
        let fixture = TestDir::new("root_parent_none");
        write_file(fixture.path(), "root.png");

        let records = scan_png_files(fixture.path()).expect("scan should succeed");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].relative_path, PathBuf::from("root.png"));
        assert_eq!(records[0].parent_dir_name, None);
    }

    #[test]
    fn returns_file_read_error_when_scan_target_is_not_a_directory() {
        let fixture = TestDir::new("scan_failure");
        let file_path = fixture.path().join("not_a_directory.png");
        fs::write(&file_path, b"test").expect("test file should be written");

        let error = scan_png_files(&file_path).expect_err("scan should fail");
        match error {
            crate::error::CompareError::FileRead { path, .. } => assert_eq!(path, file_path),
            other => panic!("expected FileRead error, got {other:?}"),
        }
    }

    #[test]
    fn empty_directory_returns_no_records() {
        let fixture = TestDir::new("empty_directory");

        let records = scan_png_files(fixture.path()).expect("scan should succeed");
        assert!(records.is_empty());
    }

    #[test]
    fn build_pairing_pairs_unique_file_names_directly() {
        let left = vec![record("left/unique.png")];
        let right = vec![record("right/unique.png")];

        let pairing = build_pairing(&left, &right);
        assert_eq!(pairing.matched.len(), 1);
        assert_eq!(pairing.matched[0].file_name, "unique.png");
        assert_eq!(pairing.matched[0].match_strategy, MatchStrategy::FileName);
        assert!(pairing.left_only.is_empty());
        assert!(pairing.right_only.is_empty());
    }

    #[test]
    fn build_pairing_resolves_duplicate_file_names_using_parent_directory_name() {
        let left = vec![record("jobs/a/dup.png"), record("jobs/b/dup.png")];
        let right = vec![record("incoming/b/dup.png"), record("incoming/a/dup.png")];

        let pairing = build_pairing(&left, &right);
        assert_eq!(pairing.matched.len(), 2);
        assert!(
            pairing
                .matched
                .iter()
                .all(|pair| pair.match_strategy == MatchStrategy::FileNameAndParentDir)
        );

        let mut matched_parents: Vec<&str> = pairing
            .matched
            .iter()
            .map(|pair| pair.left.parent_dir_name.as_deref().unwrap_or(""))
            .collect();
        matched_parents.sort_unstable();
        assert_eq!(matched_parents, vec!["a", "b"]);
        assert!(pairing.left_only.is_empty());
        assert!(pairing.right_only.is_empty());
    }

    #[test]
    fn build_pairing_matches_file_and_parent_names_case_insensitively_on_windows() {
        let left = vec![record("jobs/RouteA/Foo.PNG"), record("jobs/RouteB/Foo.PNG")];
        let right = vec![
            record("incoming/routea/foo.png"),
            record("incoming/routeb/foo.png"),
        ];

        let pairing = build_pairing(&left, &right);

        if cfg!(windows) {
            assert_eq!(pairing.matched.len(), 2);
            assert!(
                pairing
                    .matched
                    .iter()
                    .all(|pair| pair.match_strategy == MatchStrategy::FileNameAndParentDir)
            );

            let mut matched_parents: Vec<&str> = pairing
                .matched
                .iter()
                .map(|pair| pair.left.parent_dir_name.as_deref().unwrap_or(""))
                .collect();
            matched_parents.sort_unstable();
            assert_eq!(matched_parents, vec!["RouteA", "RouteB"]);
            assert!(pairing.left_only.is_empty());
            assert!(pairing.right_only.is_empty());
        } else {
            assert!(pairing.matched.is_empty());
            assert_eq!(pairing.left_only.len(), 2);
            assert_eq!(pairing.right_only.len(), 2);
        }
    }

    #[test]
    fn build_pairing_matches_cross_format_bus_stop_names_via_canonical_key() {
        let left = vec![record("a/前进一路_新安公园地铁站_西_01_1350x2060_A.png")];
        let right = vec![record("b/前进一路_新安公园地铁站_西_1_1350x2060_正.png")];

        let pairing = build_pairing(&left, &right);
        assert_eq!(pairing.matched.len(), 1);
        assert_eq!(
            pairing.matched[0].match_strategy,
            MatchStrategy::CanonicalStopId
        );
        assert!(pairing.left_only.is_empty());
        assert!(pairing.right_only.is_empty());
    }

    #[test]
    fn build_pairing_matches_insert_strip_letter_to_numeral_via_canonical_key() {
        let left = vec![record("a/平冠道_平冠道_东_195x920_C.png")];
        let right = vec![record("b/平冠道_平冠道_东_三_001_195x920.png")];

        let pairing = build_pairing(&left, &right);
        assert_eq!(pairing.matched.len(), 1);
        assert_eq!(
            pairing.matched[0].match_strategy,
            MatchStrategy::CanonicalStopId
        );
        assert!(pairing.left_only.is_empty());
        assert!(pairing.right_only.is_empty());
    }

    #[test]
    fn build_pairing_matches_rack_02_a_to_stop_03_zheng_via_canonical_key() {
        let left = vec![record("a/R_S_北_02_W_A.png")];
        let right = vec![record("b/R_S_北_3_W_正.png")];

        let pairing = build_pairing(&left, &right);
        assert_eq!(pairing.matched.len(), 1);
        assert_eq!(
            pairing.matched[0].match_strategy,
            MatchStrategy::CanonicalStopId
        );
    }

    #[test]
    fn build_pairing_canonical_key_ignores_route_and_size_differences() {
        let left = vec![record("a/路A_站_南_03_100x200_正.png")];
        let right = vec![record("b/路B_站_南_03_999x999_正.png")];

        let pairing = build_pairing(&left, &right);
        assert_eq!(pairing.matched.len(), 1);
        assert_eq!(
            pairing.matched[0].match_strategy,
            MatchStrategy::CanonicalStopId
        );
    }

    #[test]
    fn build_pairing_falls_back_to_file_name_when_canonical_parse_fails() {
        // Both sides have an unparseable name (extra " (1)" copy suffix) — they
        // share the exact same file name so the raw-name fallback still matches.
        let left = vec![record("a/前进一路_新安公园地铁站_西_1_1350x2060_正 (1).png")];
        let right = vec![record("b/前进一路_新安公园地铁站_西_1_1350x2060_正 (1).png")];

        let pairing = build_pairing(&left, &right);
        assert_eq!(pairing.matched.len(), 1);
        assert_eq!(pairing.matched[0].match_strategy, MatchStrategy::FileName);
    }

    #[test]
    fn build_pairing_does_not_match_parseable_to_unparseable_counterpart() {
        let left = vec![record("a/前进一路_新安公园地铁站_西_01_1350x2060_A.png")];
        // Right side is unparseable so it falls back to raw file name; the keys
        // differ, so no match — matches the documented fallback contract.
        let right = vec![record("b/前进一路_新安公园地铁站_西_1_1350x2060_正 (1).png")];

        let pairing = build_pairing(&left, &right);
        assert!(pairing.matched.is_empty());
        assert_eq!(pairing.left_only.len(), 1);
        assert_eq!(pairing.right_only.len(), 1);
    }

    #[test]
    fn build_pairing_canonical_key_does_not_match_different_stop_numbers() {
        let left = vec![record("a/R_S_北_01_W_A.png")]; // stop 01
        let right = vec![record("b/R_S_北_01_W_B.png")]; // stop 02

        let pairing = build_pairing(&left, &right);
        assert!(pairing.matched.is_empty());
        assert_eq!(pairing.left_only.len(), 1);
        assert_eq!(pairing.right_only.len(), 1);
    }

    #[test]
    fn build_pairing_keeps_still_ambiguous_duplicates_unmatched_on_both_sides() {
        let left = vec![
            record("left/shared/dup.png"),
            record("archive/shared/dup.png"),
        ];
        let right = vec![
            record("right/shared/dup.png"),
            record("export/shared/dup.png"),
        ];

        let pairing = build_pairing(&left, &right);
        assert!(pairing.matched.is_empty());
        assert_eq!(pairing.left_only.len(), 2);
        assert_eq!(pairing.right_only.len(), 2);
        assert!(
            pairing
                .left_only
                .iter()
                .all(|item| item.side == UnmatchedSide::Left)
        );
        assert!(
            pairing
                .right_only
                .iter()
                .all(|item| item.side == UnmatchedSide::Right)
        );
        assert!(pairing.left_only.iter().all(
            |item| item.reason
                == "ambiguous duplicate file name group for 'dup.png' after parent directory refinement"
        ));
        assert!(pairing.right_only.iter().all(
            |item| item.reason
                == "ambiguous duplicate file name group for 'dup.png' after parent directory refinement"
        ));
    }
}
