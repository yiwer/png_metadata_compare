use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::batch_report::{
    MatchStrategy, MatchedPair, PairingResult, UnmatchedFile, UnmatchedSide,
};
use crate::error::CompareError;

#[derive(Clone, Debug, PartialEq, Eq)]
// Staged API for upcoming batch-compare wiring in later tasks.
#[allow(dead_code)]
pub struct BatchFileRecord {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_name: String,
    pub parent_dir_name: Option<String>,
}

// Staged API for upcoming batch-compare wiring in later tasks.
#[allow(dead_code)]
pub fn scan_png_files(root: &Path) -> Result<Vec<BatchFileRecord>, CompareError> {
    let root_absolute = if root.is_absolute() {
        root.to_path_buf()
    } else {
        root.canonicalize().map_err(|err| CompareError::FileRead {
            path: root.to_path_buf(),
            reason: err.to_string(),
        })?
    };

    let mut records = Vec::new();
    walk_png_files(&root_absolute, &root_absolute, &mut records)?;
    records.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(records)
}

// Staged API for upcoming batch-compare wiring in later tasks.
#[allow(dead_code)]
pub fn build_pairing(left: &[BatchFileRecord], right: &[BatchFileRecord]) -> PairingResult {
    let mut result = PairingResult::default();
    let mut left_by_file_name = group_by_file_name(left);
    let mut right_by_file_name = group_by_file_name(right);
    let mut file_names = BTreeSet::new();

    file_names.extend(left_by_file_name.keys().cloned());
    file_names.extend(right_by_file_name.keys().cloned());

    for file_name in file_names {
        let left_group = left_by_file_name.remove(&file_name).unwrap_or_default();
        let right_group = right_by_file_name.remove(&file_name).unwrap_or_default();

        if left_group.len() == 1 && right_group.len() == 1 {
            result.matched.push(MatchedPair {
                file_name: file_name.clone(),
                left: left_group[0].clone(),
                right: right_group[0].clone(),
                match_strategy: MatchStrategy::FileName,
            });
            continue;
        }

        if left_group.is_empty() {
            add_unmatched_files(
                &mut result.right_only,
                UnmatchedSide::Right,
                right_group,
                format!("no file named '{file_name}' found on left side"),
            );
            continue;
        }

        if right_group.is_empty() {
            add_unmatched_files(
                &mut result.left_only,
                UnmatchedSide::Left,
                left_group,
                format!("no file named '{file_name}' found on right side"),
            );
            continue;
        }

        resolve_duplicate_file_name_group(&file_name, left_group, right_group, &mut result);
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

fn group_by_file_name(records: &[BatchFileRecord]) -> HashMap<String, Vec<BatchFileRecord>> {
    let mut groups = HashMap::new();

    for record in records {
        groups
            .entry(record.file_name.clone())
            .or_insert_with(Vec::new)
            .push(record.clone());
    }

    groups
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
                    format_parent_dir_name(&parent_key)
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
                    format_parent_dir_name(&parent_key)
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
            .entry(record.parent_dir_name.clone())
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

fn format_parent_dir_name(parent_dir_name: &Option<String>) -> &str {
    parent_dir_name
        .as_deref()
        .unwrap_or("<root>")
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

        records.push(BatchFileRecord {
            absolute_path: path,
            relative_path,
            file_name,
            parent_dir_name,
        });
    }

    Ok(())
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

    use super::{build_pairing, scan_png_files, BatchFileRecord};
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
            fixture.path().join("a").join("b").join("c").join("image.png")
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
        assert!(pairing
            .left_only
            .iter()
            .all(|item| item.side == UnmatchedSide::Left));
        assert!(pairing
            .right_only
            .iter()
            .all(|item| item.side == UnmatchedSide::Right));
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
