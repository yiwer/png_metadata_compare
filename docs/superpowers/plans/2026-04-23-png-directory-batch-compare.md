# PNG Directory Batch Compare Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the existing Windows-native PNG metadata compare tool with a directory mode that recursively scans two directories, pairs PNG files by file name and then parent directory name, compares matched pairs, and shows a categorized clickable batch report inside the GUI.

**Architecture:** The existing single-file compare pipeline remains intact and reusable. A new batch layer scans directories, resolves matches, classifies results into four categories, and stores enough per-pair diff state to drive the existing tree/detail UI for `Different` items. Scan-level failures are rendered as non-fatal batch issues outside the four required result categories.

**Tech Stack:** Rust 2024, `eframe/egui`, `rfd`, `serde_json`, existing in-repo PNG reader and diff engine

---

## File Structure

Create or modify these files during implementation:

- Modify: `src/app.rs`
- Modify: `src/main.rs`
- Create: `src/batch_scan.rs`
- Create: `src/batch_report.rs`
- Modify: `src/ui/summary.rs`
- Modify: `src/ui/detail.rs`
- Modify: `src/ui/tree.rs`

The file responsibilities are:

- `src/batch_scan.rs`: recursive PNG discovery, file records, grouping, pairing by file name and parent directory name, unmatched-reason generation.
- `src/batch_report.rs`: batch result models, category grouping, matched-pair outcome representation, detail payloads for GUI selection.
- `src/app.rs`: mode switch, directory selection state, batch compare execution, report selection state, integration with existing diff UI.
- `src/ui/summary.rs`: report navigation for the four batch categories and item selection.
- `src/ui/detail.rs`: detail rendering for identical items, different items, and unmatched items.
- `src/ui/tree.rs`: continue rendering the structured diff tree for selected `Different` items.
- `src/main.rs`: test-only crate-root wrappers if exact test-command compatibility is needed.

### Task 1: Add recursive PNG scanning and file record modeling

**Files:**
- Create: `src/batch_scan.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write the failing scan tests**

Create `src/batch_scan.rs` with these tests first:

```rust
#[cfg(test)]
mod tests {
    use super::scan_png_files;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn recursively_finds_png_files_and_ignores_other_files() {
        let root = unique_test_dir("scan");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("a.png"), b"png").unwrap();
        fs::write(root.join("nested").join("b.PNG"), b"png").unwrap();
        fs::write(root.join("nested").join("note.txt"), b"text").unwrap();

        let files = scan_png_files(&root).unwrap();
        let names = files
            .iter()
            .map(|file| file.file_name.clone())
            .collect::<Vec<_>>();

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a.png".to_string()));
        assert!(names.contains(&"b.PNG".to_string()));
    }

    #[test]
    fn captures_parent_directory_name_for_nested_file() {
        let root = unique_test_dir("parent");
        fs::create_dir_all(root.join("line_a")).unwrap();
        fs::write(root.join("line_a").join("stop.png"), b"png").unwrap();

        let files = scan_png_files(&root).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name, "stop.png");
        assert_eq!(files[0].parent_dir_name.as_deref(), Some("line_a"));
    }

    fn unique_test_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "png_metadata_compare_batch_scan_{label}_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test batch_scan -- --nocapture`

Expected: FAIL because `scan_png_files` does not exist

- [ ] **Step 3: Write minimal scanning implementation**

Replace `src/batch_scan.rs` with:

```rust
use crate::error::CompareError;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchFileRecord {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_name: String,
    pub parent_dir_name: Option<String>,
}

pub fn scan_png_files(root: &Path) -> Result<Vec<BatchFileRecord>, CompareError> {
    let mut files = Vec::new();
    scan_dir(root, root, &mut files)?;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}

fn scan_dir(root: &Path, current: &Path, files: &mut Vec<BatchFileRecord>) -> Result<(), CompareError> {
    let entries = std::fs::read_dir(current).map_err(|err| CompareError::FileRead {
        path: current.to_path_buf(),
        reason: err.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|err| CompareError::FileRead {
            path: current.to_path_buf(),
            reason: err.to_string(),
        })?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir(root, &path, files)?;
            continue;
        }

        let is_png = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("png"))
            .unwrap_or(false);
        if !is_png {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_path_buf();
        let parent_dir_name = path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_owned);

        files.push(BatchFileRecord {
            absolute_path: path,
            relative_path,
            file_name,
            parent_dir_name,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::scan_png_files;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn recursively_finds_png_files_and_ignores_other_files() {
        let root = unique_test_dir("scan");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("a.png"), b"png").unwrap();
        fs::write(root.join("nested").join("b.PNG"), b"png").unwrap();
        fs::write(root.join("nested").join("note.txt"), b"text").unwrap();

        let files = scan_png_files(&root).unwrap();
        let names = files
            .iter()
            .map(|file| file.file_name.clone())
            .collect::<Vec<_>>();

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a.png".to_string()));
        assert!(names.contains(&"b.PNG".to_string()));
    }

    #[test]
    fn captures_parent_directory_name_for_nested_file() {
        let root = unique_test_dir("parent");
        fs::create_dir_all(root.join("line_a")).unwrap();
        fs::write(root.join("line_a").join("stop.png"), b"png").unwrap();

        let files = scan_png_files(&root).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name, "stop.png");
        assert_eq!(files[0].parent_dir_name.as_deref(), Some("line_a"));
    }

    fn unique_test_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "png_metadata_compare_batch_scan_{label}_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test batch_scan -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/batch_scan.rs
git commit -m "feat: add recursive PNG directory scanning"
```

### Task 2: Implement directory pairing and unmatched classification

**Files:**
- Modify: `src/batch_scan.rs`
- Create: `src/batch_report.rs`

- [ ] **Step 1: Write the failing pairing tests**

Append these tests to `src/batch_scan.rs`:

```rust
#[test]
fn pairs_unique_file_names_directly() {
    let left = vec![record("left/a.png", "a.png", Some("left"))];
    let right = vec![record("right/a.png", "a.png", Some("right"))];

    let result = build_pairing(&left, &right);

    assert_eq!(result.matched.len(), 1);
    assert!(result.left_only.is_empty());
    assert!(result.right_only.is_empty());
}

#[test]
fn resolves_duplicate_file_names_by_parent_directory_name() {
    let left = vec![
        record("left/alpha/stop.png", "stop.png", Some("alpha")),
        record("left/beta/stop.png", "stop.png", Some("beta")),
    ];
    let right = vec![
        record("right/alpha/stop.png", "stop.png", Some("alpha")),
        record("right/beta/stop.png", "stop.png", Some("beta")),
    ];

    let result = build_pairing(&left, &right);
    assert_eq!(result.matched.len(), 2);
}

#[test]
fn leaves_files_unmatched_when_duplicate_group_is_still_ambiguous() {
    let left = vec![
        record("left/a/stop.png", "stop.png", Some("dup")),
        record("left/b/stop.png", "stop.png", Some("dup")),
    ];
    let right = vec![record("right/c/stop.png", "stop.png", Some("dup"))];

    let result = build_pairing(&left, &right);
    assert!(result.matched.is_empty());
    assert_eq!(result.left_only.len(), 2);
    assert_eq!(result.right_only.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test build_pairing -- --nocapture`

Expected: FAIL because `build_pairing` and `record` do not exist

- [ ] **Step 3: Add pairing and report models**

Create `src/batch_report.rs`:

```rust
use crate::batch_scan::BatchFileRecord;
use crate::diff::{DiffNode, DiffSummary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchStrategy {
    FileName,
    FileNameAndParentDir,
}

#[derive(Debug, Clone)]
pub struct MatchedPair {
    pub file_name: String,
    pub left: BatchFileRecord,
    pub right: BatchFileRecord,
    pub match_strategy: MatchStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnmatchedSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct UnmatchedFile {
    pub side: UnmatchedSide,
    pub file: BatchFileRecord,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct PairingResult {
    pub matched: Vec<MatchedPair>,
    pub left_only: Vec<UnmatchedFile>,
    pub right_only: Vec<UnmatchedFile>,
}

#[derive(Debug, Clone)]
pub enum BatchCompareOutcome {
    Identical,
    Different {
        diff_root: DiffNode,
        diff_summary: DiffSummary,
    },
}
```

Update `src/main.rs` module declarations:

```rust
mod batch_report;
mod batch_scan;
```

Replace `src/batch_scan.rs` with:

```rust
use crate::batch_report::{MatchStrategy, MatchedPair, PairingResult, UnmatchedFile, UnmatchedSide};
use crate::error::CompareError;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchFileRecord {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_name: String,
    pub parent_dir_name: Option<String>,
}

pub fn scan_png_files(root: &Path) -> Result<Vec<BatchFileRecord>, CompareError> {
    let mut files = Vec::new();
    scan_dir(root, root, &mut files)?;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}

pub fn build_pairing(left: &[BatchFileRecord], right: &[BatchFileRecord]) -> PairingResult {
    let mut result = PairingResult::default();
    let mut left_groups: BTreeMap<&str, Vec<&BatchFileRecord>> = BTreeMap::new();
    let mut right_groups: BTreeMap<&str, Vec<&BatchFileRecord>> = BTreeMap::new();

    for file in left {
        left_groups.entry(&file.file_name).or_default().push(file);
    }
    for file in right {
        right_groups.entry(&file.file_name).or_default().push(file);
    }

    let mut keys = left_groups.keys().cloned().collect::<Vec<_>>();
    for key in right_groups.keys() {
        if !keys.contains(key) {
            keys.push(key);
        }
    }
    keys.sort();

    for key in keys {
        let left_group = left_groups.get(key).cloned().unwrap_or_default();
        let right_group = right_groups.get(key).cloned().unwrap_or_default();

        if left_group.len() == 1 && right_group.len() == 1 {
            result.matched.push(MatchedPair {
                file_name: key.to_string(),
                left: left_group[0].clone(),
                right: right_group[0].clone(),
                match_strategy: MatchStrategy::FileName,
            });
            continue;
        }

        if left_group.is_empty() {
            for file in right_group {
                result.right_only.push(unmatched(UnmatchedSide::Right, file, "no same-name file on left"));
            }
            continue;
        }

        if right_group.is_empty() {
            for file in left_group {
                result.left_only.push(unmatched(UnmatchedSide::Left, file, "no same-name file on right"));
            }
            continue;
        }

        let mut left_by_parent: BTreeMap<Option<&str>, Vec<&BatchFileRecord>> = BTreeMap::new();
        let mut right_by_parent: BTreeMap<Option<&str>, Vec<&BatchFileRecord>> = BTreeMap::new();
        for file in &left_group {
            left_by_parent.entry(file.parent_dir_name.as_deref()).or_default().push(*file);
        }
        for file in &right_group {
            right_by_parent.entry(file.parent_dir_name.as_deref()).or_default().push(*file);
        }

        let mut parents = left_by_parent.keys().cloned().collect::<Vec<_>>();
        for parent in right_by_parent.keys() {
            if !parents.contains(parent) {
                parents.push(*parent);
            }
        }
        parents.sort();

        let mut matched_any = false;
        let mut ambiguous = false;

        for parent in parents {
            let left_items = left_by_parent.get(&parent).cloned().unwrap_or_default();
            let right_items = right_by_parent.get(&parent).cloned().unwrap_or_default();

            match (left_items.len(), right_items.len()) {
                (1, 1) => {
                    matched_any = true;
                    result.matched.push(MatchedPair {
                        file_name: key.to_string(),
                        left: left_items[0].clone(),
                        right: right_items[0].clone(),
                        match_strategy: MatchStrategy::FileNameAndParentDir,
                    });
                }
                (0, 0) => {}
                _ => {
                    ambiguous = true;
                }
            }
        }

        if ambiguous || !matched_any || result
            .matched
            .iter()
            .filter(|pair| pair.file_name == key)
            .count()
            != left_group.len().min(right_group.len())
        {
            for file in left_group {
                if !result.matched.iter().any(|pair| pair.left.absolute_path == file.absolute_path) {
                    result.left_only.push(unmatched(
                        UnmatchedSide::Left,
                        file,
                        "duplicate file-name group could not be uniquely resolved",
                    ));
                }
            }
            for file in right_group {
                if !result.matched.iter().any(|pair| pair.right.absolute_path == file.absolute_path) {
                    result.right_only.push(unmatched(
                        UnmatchedSide::Right,
                        file,
                        "duplicate file-name group could not be uniquely resolved",
                    ));
                }
            }
        }
    }

    result
}

fn unmatched(side: UnmatchedSide, file: &BatchFileRecord, reason: &str) -> UnmatchedFile {
    UnmatchedFile {
        side,
        file: file.clone(),
        reason: reason.to_string(),
    }
}

fn scan_dir(root: &Path, current: &Path, files: &mut Vec<BatchFileRecord>) -> Result<(), CompareError> {
    let entries = std::fs::read_dir(current).map_err(|err| CompareError::FileRead {
        path: current.to_path_buf(),
        reason: err.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|err| CompareError::FileRead {
            path: current.to_path_buf(),
            reason: err.to_string(),
        })?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir(root, &path, files)?;
            continue;
        }

        let is_png = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("png"))
            .unwrap_or(false);
        if !is_png {
            continue;
        }

        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default().to_string();
        let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let parent_dir_name = path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_owned);

        files.push(BatchFileRecord {
            absolute_path: path,
            relative_path,
            file_name,
            parent_dir_name,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_pairing, scan_png_files, BatchFileRecord};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn recursively_finds_png_files_and_ignores_other_files() {
        let root = unique_test_dir("scan");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::write(root.join("a.png"), b"png").unwrap();
        fs::write(root.join("nested").join("b.PNG"), b"png").unwrap();
        fs::write(root.join("nested").join("note.txt"), b"text").unwrap();

        let files = scan_png_files(&root).unwrap();
        let names = files.iter().map(|file| file.file_name.clone()).collect::<Vec<_>>();

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a.png".to_string()));
        assert!(names.contains(&"b.PNG".to_string()));
    }

    #[test]
    fn captures_parent_directory_name_for_nested_file() {
        let root = unique_test_dir("parent");
        fs::create_dir_all(root.join("line_a")).unwrap();
        fs::write(root.join("line_a").join("stop.png"), b"png").unwrap();

        let files = scan_png_files(&root).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name, "stop.png");
        assert_eq!(files[0].parent_dir_name.as_deref(), Some("line_a"));
    }

    #[test]
    fn pairs_unique_file_names_directly() {
        let left = vec![record("left/a.png", "a.png", Some("left"))];
        let right = vec![record("right/a.png", "a.png", Some("right"))];

        let result = build_pairing(&left, &right);

        assert_eq!(result.matched.len(), 1);
        assert!(result.left_only.is_empty());
        assert!(result.right_only.is_empty());
    }

    #[test]
    fn resolves_duplicate_file_names_by_parent_directory_name() {
        let left = vec![
            record("left/alpha/stop.png", "stop.png", Some("alpha")),
            record("left/beta/stop.png", "stop.png", Some("beta")),
        ];
        let right = vec![
            record("right/alpha/stop.png", "stop.png", Some("alpha")),
            record("right/beta/stop.png", "stop.png", Some("beta")),
        ];

        let result = build_pairing(&left, &right);
        assert_eq!(result.matched.len(), 2);
    }

    #[test]
    fn leaves_files_unmatched_when_duplicate_group_is_still_ambiguous() {
        let left = vec![
            record("left/a/stop.png", "stop.png", Some("dup")),
            record("left/b/stop.png", "stop.png", Some("dup")),
        ];
        let right = vec![record("right/c/stop.png", "stop.png", Some("dup"))];

        let result = build_pairing(&left, &right);
        assert!(result.matched.is_empty());
        assert_eq!(result.left_only.len(), 2);
        assert_eq!(result.right_only.len(), 1);
    }

    fn record(path: &str, file_name: &str, parent: Option<&str>) -> BatchFileRecord {
        BatchFileRecord {
            absolute_path: PathBuf::from(path),
            relative_path: PathBuf::from(path),
            file_name: file_name.to_string(),
            parent_dir_name: parent.map(str::to_owned),
        }
    }

    fn unique_test_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "png_metadata_compare_batch_scan_{label}_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test build_pairing -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/batch_scan.rs src/batch_report.rs
git commit -m "feat: add directory pairing and unmatched classification"
```

### Task 3: Build batch compare outcomes and batch issue models

**Files:**
- Modify: `src/batch_report.rs`
- Modify: `src/batch_scan.rs`
- Test: `src/batch_report.rs`

- [ ] **Step 1: Write the failing batch compare tests**

Append these tests to `src/batch_report.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::{build_batch_results, BatchIssue, DifferentPairResult, MatchStrategy};
    use crate::batch_scan::BatchFileRecord;
    use crate::diff::{DiffNode, DiffStatus, DiffSummary};
    use std::path::PathBuf;

    #[test]
    fn classifies_matched_pair_with_no_differences_as_identical() {
        let pair = matched_pair("stop.png");
        let result = build_batch_results(
            vec![(pair, fake_compare_result(false))],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(result.identical.len(), 1);
        assert!(result.different.is_empty());
    }

    #[test]
    fn classifies_matched_pair_with_differences_as_different() {
        let pair = matched_pair("stop.png");
        let result = build_batch_results(
            vec![(pair, fake_compare_result(true))],
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(result.different.len(), 1);
        assert!(result.identical.is_empty());
        assert_eq!(result.different[0].summary.total(), 1);
    }

    #[test]
    fn keeps_scan_issue_outside_result_categories() {
        let result = build_batch_results(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![BatchIssue::ScanFailure {
                side: "left".to_string(),
                path: PathBuf::from("left"),
                reason: "directory unreadable".to_string(),
            }],
        );

        assert_eq!(result.issues.len(), 1);
        assert!(result.identical.is_empty());
        assert!(result.different.is_empty());
        assert!(result.left_only.is_empty());
        assert!(result.right_only.is_empty());
    }

    fn matched_pair(name: &str) -> crate::batch_report::MatchedPair {
        crate::batch_report::MatchedPair {
            file_name: name.to_string(),
            left: BatchFileRecord {
                absolute_path: PathBuf::from(format!("left/{name}")),
                relative_path: PathBuf::from(name),
                file_name: name.to_string(),
                parent_dir_name: Some("left".into()),
            },
            right: BatchFileRecord {
                absolute_path: PathBuf::from(format!("right/{name}")),
                relative_path: PathBuf::from(name),
                file_name: name.to_string(),
                parent_dir_name: Some("right".into()),
            },
            match_strategy: MatchStrategy::FileName,
        }
    }

    fn fake_compare_result(has_diff: bool) -> DifferentPairResult {
        let root = DiffNode {
            path: "StopPlateMetadata".into(),
            status: if has_diff { DiffStatus::Modified } else { DiffStatus::Unchanged },
            left_value: None,
            right_value: None,
            summary: String::new(),
            children: Vec::new(),
        };
        let mut summary = DiffSummary::default();
        if has_diff {
            summary.modified = 1;
        }
        DifferentPairResult {
            pair: matched_pair("stop.png"),
            diff_root: root,
            change_list: Vec::new(),
            summary,
            selected_path: None,
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test build_batch_results -- --nocapture`

Expected: FAIL because batch result builders and categories do not exist

- [ ] **Step 3: Implement batch result classification**

Replace `src/batch_report.rs` with:

```rust
use crate::batch_scan::BatchFileRecord;
use crate::diff::{DiffNode, DiffSummary};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchStrategy {
    FileName,
    FileNameAndParentDir,
}

#[derive(Debug, Clone)]
pub struct MatchedPair {
    pub file_name: String,
    pub left: BatchFileRecord,
    pub right: BatchFileRecord,
    pub match_strategy: MatchStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnmatchedSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct UnmatchedFile {
    pub side: UnmatchedSide,
    pub file: BatchFileRecord,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct IdenticalPairResult {
    pub pair: MatchedPair,
}

#[derive(Debug, Clone)]
pub struct DifferentPairResult {
    pub pair: MatchedPair,
    pub diff_root: DiffNode,
    pub change_list: Vec<DiffNode>,
    pub summary: DiffSummary,
    pub selected_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum BatchIssue {
    ScanFailure {
        side: String,
        path: PathBuf,
        reason: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct PairingResult {
    pub matched: Vec<MatchedPair>,
    pub left_only: Vec<UnmatchedFile>,
    pub right_only: Vec<UnmatchedFile>,
}

#[derive(Debug, Clone, Default)]
pub struct BatchCompareReport {
    pub issues: Vec<BatchIssue>,
    pub identical: Vec<IdenticalPairResult>,
    pub different: Vec<DifferentPairResult>,
    pub left_only: Vec<UnmatchedFile>,
    pub right_only: Vec<UnmatchedFile>,
}

pub fn build_batch_results(
    matched: Vec<(MatchedPair, DifferentPairResult)>,
    left_only: Vec<UnmatchedFile>,
    right_only: Vec<UnmatchedFile>,
    issues: Vec<BatchIssue>,
) -> BatchCompareReport {
    let mut report = BatchCompareReport {
        issues,
        identical: Vec::new(),
        different: Vec::new(),
        left_only,
        right_only,
    };

    for (pair, mut different_result) in matched {
        if different_result.summary.total() == 0 {
            report.identical.push(IdenticalPairResult { pair });
        } else {
            different_result.pair = pair;
            report.different.push(different_result);
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::build_batch_results;
    use crate::batch_report::{BatchIssue, DifferentPairResult, MatchStrategy};
    use crate::batch_scan::BatchFileRecord;
    use crate::diff::{DiffNode, DiffStatus, DiffSummary};
    use std::path::PathBuf;

    #[test]
    fn classifies_matched_pair_with_no_differences_as_identical() {
        let pair = matched_pair("stop.png");
        let result = build_batch_results(vec![(pair, fake_compare_result(false))], Vec::new(), Vec::new(), Vec::new());

        assert_eq!(result.identical.len(), 1);
        assert!(result.different.is_empty());
    }

    #[test]
    fn classifies_matched_pair_with_differences_as_different() {
        let pair = matched_pair("stop.png");
        let result = build_batch_results(vec![(pair, fake_compare_result(true))], Vec::new(), Vec::new(), Vec::new());

        assert_eq!(result.different.len(), 1);
        assert!(result.identical.is_empty());
        assert_eq!(result.different[0].summary.total(), 1);
    }

    #[test]
    fn keeps_scan_issue_outside_result_categories() {
        let result = build_batch_results(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![BatchIssue::ScanFailure {
                side: "left".to_string(),
                path: PathBuf::from("left"),
                reason: "directory unreadable".to_string(),
            }],
        );

        assert_eq!(result.issues.len(), 1);
        assert!(result.identical.is_empty());
        assert!(result.different.is_empty());
        assert!(result.left_only.is_empty());
        assert!(result.right_only.is_empty());
    }

    fn matched_pair(name: &str) -> crate::batch_report::MatchedPair {
        crate::batch_report::MatchedPair {
            file_name: name.to_string(),
            left: BatchFileRecord {
                absolute_path: PathBuf::from(format!("left/{name}")),
                relative_path: PathBuf::from(name),
                file_name: name.to_string(),
                parent_dir_name: Some("left".into()),
            },
            right: BatchFileRecord {
                absolute_path: PathBuf::from(format!("right/{name}")),
                relative_path: PathBuf::from(name),
                file_name: name.to_string(),
                parent_dir_name: Some("right".into()),
            },
            match_strategy: MatchStrategy::FileName,
        }
    }

    fn fake_compare_result(has_diff: bool) -> DifferentPairResult {
        let root = DiffNode {
            path: "StopPlateMetadata".into(),
            status: if has_diff { DiffStatus::Modified } else { DiffStatus::Unchanged },
            left_value: None,
            right_value: None,
            summary: String::new(),
            children: Vec::new(),
        };
        let mut summary = DiffSummary::default();
        if has_diff {
            summary.modified = 1;
        }
        DifferentPairResult {
            pair: matched_pair("stop.png"),
            diff_root: root,
            change_list: Vec::new(),
            summary,
            selected_path: None,
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test build_batch_results -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/batch_report.rs
git commit -m "feat: classify batch compare outcomes"
```

### Task 4: Add app-level directory mode and execute batch comparisons

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs`
- Modify: `src/batch_scan.rs`
- Modify: `src/batch_report.rs`

- [ ] **Step 1: Write the failing directory-mode app tests**

Append these tests to `src/app.rs`:

```rust
#[test]
fn directory_mode_classifies_identical_and_different_pairs() {
    let fixture = BatchDirFixture::new(
        &[("same.png", "common", r#"{"StopName":"A"}"#), ("diff.png", "common", r#"{"StopName":"B"}"#)],
        &[("same.png", "common", r#"{"StopName":"A"}"#), ("diff.png", "common", r#"{"StopName":"C"}"#)],
    );

    let mut app = PngMetadataCompareApp::default();
    app.mode = AppMode::Directory;
    app.left_dir = Some(fixture.left_dir.display().to_string());
    app.right_dir = Some(fixture.right_dir.display().to_string());
    app.run_directory_compare();

    let report = app.batch_report.as_ref().unwrap();
    assert_eq!(report.identical.len(), 1);
    assert_eq!(report.different.len(), 1);
}

#[test]
fn directory_mode_reports_unmatched_files() {
    let fixture = BatchDirFixture::new(
        &[("left_only.png", "common", r#"{"StopName":"A"}"#)],
        &[("right_only.png", "common", r#"{"StopName":"B"}"#)],
    );

    let mut app = PngMetadataCompareApp::default();
    app.mode = AppMode::Directory;
    app.left_dir = Some(fixture.left_dir.display().to_string());
    app.right_dir = Some(fixture.right_dir.display().to_string());
    app.run_directory_compare();

    let report = app.batch_report.as_ref().unwrap();
    assert_eq!(report.left_only.len(), 1);
    assert_eq!(report.right_only.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test directory_mode_ -- --nocapture`

Expected: FAIL because `AppMode`, directory state, fixture, or `run_directory_compare` do not exist

- [ ] **Step 3: Implement directory-mode state and compare execution**

Update `src/app.rs` to add:

```rust
use crate::batch_report::{build_batch_results, BatchCompareReport, BatchIssue, DifferentPairResult};
use crate::batch_scan::build_pairing;
use crate::batch_scan::scan_png_files;
```

Add these types near the top:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    SingleFile,
    Directory,
}

#[derive(Debug, Clone)]
pub enum BatchSelection {
    Identical(usize),
    Different(usize),
    LeftOnly(usize),
    RightOnly(usize),
}
```

Extend app state:

```rust
pub left_dir: Option<String>,
pub right_dir: Option<String>,
pub batch_report: Option<BatchCompareReport>,
pub batch_selection: Option<BatchSelection>,
pub mode: AppMode,
```

Initialize them in `Default`.

Add:

```rust
pub fn run_directory_compare(&mut self) {
    let (Some(left_dir), Some(right_dir)) = (self.left_dir.as_deref(), self.right_dir.as_deref()) else {
        self.batch_report = None;
        self.batch_selection = None;
        return;
    };

    let mut issues = Vec::new();
    let left_files = match scan_png_files(Path::new(left_dir)) {
        Ok(files) => files,
        Err(error) => {
            issues.push(BatchIssue::ScanFailure {
                side: "left".to_string(),
                path: Path::new(left_dir).to_path_buf(),
                reason: error.to_string(),
            });
            Vec::new()
        }
    };
    let right_files = match scan_png_files(Path::new(right_dir)) {
        Ok(files) => files,
        Err(error) => {
            issues.push(BatchIssue::ScanFailure {
                side: "right".to_string(),
                path: Path::new(right_dir).to_path_buf(),
                reason: error.to_string(),
            });
            Vec::new()
        }
    };

    let pairing = build_pairing(&left_files, &right_files);
    let matched = pairing
        .matched
        .into_iter()
        .map(|pair| {
            let left_metadata = load_metadata(extract_stop_plate_metadata_from_file(&pair.left.absolute_path));
            let right_metadata = load_metadata(extract_stop_plate_metadata_from_file(&pair.right.absolute_path));
            let diff_root = compare_metadata(&left_metadata, &right_metadata);
            let changes = flatten_changes(&diff_root);
            let summary = summarize_changes(&changes);
            let selected_path = default_selected_path(&changes);
            (
                pair.clone(),
                DifferentPairResult {
                    pair,
                    diff_root,
                    change_list: changes,
                    summary,
                    selected_path,
                },
            )
        })
        .collect::<Vec<_>>();

    let report = build_batch_results(matched, pairing.left_only, pairing.right_only, issues);
    self.batch_selection = if !report.different.is_empty() {
        Some(BatchSelection::Different(0))
    } else if !report.identical.is_empty() {
        Some(BatchSelection::Identical(0))
    } else if !report.left_only.is_empty() {
        Some(BatchSelection::LeftOnly(0))
    } else if !report.right_only.is_empty() {
        Some(BatchSelection::RightOnly(0))
    } else {
        None
    };
    self.batch_report = Some(report);
}
```

Add a test-only directory fixture to `src/app.rs`:

```rust
#[cfg(test)]
struct BatchDirFixture {
    left_dir: std::path::PathBuf,
    right_dir: std::path::PathBuf,
}

#[cfg(test)]
impl BatchDirFixture {
    fn new(left_files: &[(&str, &str, &str)], right_files: &[(&str, &str, &str)]) -> Self {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("png_metadata_compare_batch_fixture_{}_{}", std::process::id(), unique));
        let left_dir = root.join("left");
        let right_dir = root.join("right");
        fs::create_dir_all(&left_dir).unwrap();
        fs::create_dir_all(&right_dir).unwrap();

        for (name, parent, json) in left_files {
            write_png(&left_dir, parent, name, json);
        }
        for (name, parent, json) in right_files {
            write_png(&right_dir, parent, name, json);
        }

        Self { left_dir, right_dir }
    }
}

#[cfg(test)]
impl Drop for BatchDirFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(
            self.left_dir.parent().expect("fixture root parent should exist")
        );
    }
}

#[cfg(test)]
fn write_png(root: &std::path::Path, parent: &str, name: &str, json: &str) {
    let dir = root.join(parent);
    std::fs::create_dir_all(&dir).unwrap();
    let mut bytes = Vec::from(b\"\\x89PNG\\r\\n\\x1a\\n\".as_slice());
    bytes.extend(png_chunk(*b\"iTXt\", stop_plate_itxt_data(json)));
    bytes.extend(png_chunk(*b\"IEND\", Vec::new()));
    std::fs::write(dir.join(name), bytes).unwrap();
}

#[cfg(test)]
fn stop_plate_itxt_data(json: &str) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b\"StopPlateMetadata\");
    data.push(0);
    data.push(0);
    data.push(0);
    data.push(0);
    data.push(0);
    data.extend_from_slice(json.as_bytes());
    data
}

#[cfg(test)]
fn png_chunk(kind: [u8; 4], data: Vec<u8>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&kind);
    bytes.extend_from_slice(&data);
    bytes.extend_from_slice(&png_chunk_crc(kind, &data).to_be_bytes());
    bytes
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test directory_mode_ -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/batch_scan.rs src/batch_report.rs
git commit -m "feat: execute directory batch comparisons"
```

### Task 5: Add batch-mode summary navigation and detail rendering

**Files:**
- Modify: `src/ui/summary.rs`
- Modify: `src/ui/detail.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Write the failing summary navigation test**

Append this test to `src/ui/summary.rs`:

```rust
#[test]
fn batch_section_labels_include_counts() {
    let labels = batch_section_labels(3, 2, 1, 4);
    assert_eq!(
        labels,
        vec![
            "Identical (3)".to_string(),
            "Different (2)".to_string(),
            "Left Only (1)".to_string(),
            "Right Only (4)".to_string(),
        ]
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test batch_section_labels_include_counts -- --nocapture`

Expected: FAIL because helper does not exist

- [ ] **Step 3: Implement batch report navigation and detail rendering**

Update `src/ui/summary.rs` with:

```rust
use crate::app::{BatchSelection, CompareResultView, PngMetadataCompareApp};
use crate::batch_report::BatchCompareReport;
use crate::diff::DiffSummary;

pub fn batch_section_labels(identical: usize, different: usize, left_only: usize, right_only: usize) -> Vec<String> {
    vec![
        format!("Identical ({identical})"),
        format!("Different ({different})"),
        format!("Left Only ({left_only})"),
        format!("Right Only ({right_only})"),
    ]
}

pub fn draw_batch_summary(ui: &mut eframe::egui::Ui, report: &BatchCompareReport, selection: &mut Option<BatchSelection>) {
    for label in batch_section_labels(
        report.identical.len(),
        report.different.len(),
        report.left_only.len(),
        report.right_only.len(),
    ) {
        ui.label(label);
    }

    ui.separator();

    for (index, item) in report.identical.iter().enumerate() {
        if ui.selectable_label(matches!(selection, Some(BatchSelection::Identical(i)) if *i == index), format!("{} :: Metadata identical", item.pair.file_name)).clicked() {
            *selection = Some(BatchSelection::Identical(index));
        }
    }
    for (index, item) in report.different.iter().enumerate() {
        if ui.selectable_label(matches!(selection, Some(BatchSelection::Different(i)) if *i == index), format!("{} :: {} differences", item.pair.file_name, item.summary.total())).clicked() {
            *selection = Some(BatchSelection::Different(index));
        }
    }
    for (index, item) in report.left_only.iter().enumerate() {
        if ui.selectable_label(matches!(selection, Some(BatchSelection::LeftOnly(i)) if *i == index), format!("{} :: {}", item.file.file_name, item.reason)).clicked() {
            *selection = Some(BatchSelection::LeftOnly(index));
        }
    }
    for (index, item) in report.right_only.iter().enumerate() {
        if ui.selectable_label(matches!(selection, Some(BatchSelection::RightOnly(i)) if *i == index), format!("{} :: {}", item.file.file_name, item.reason)).clicked() {
            *selection = Some(BatchSelection::RightOnly(index));
        }
    }
}
```

Update `src/ui/detail.rs` with a batch detail renderer:

```rust
use crate::app::BatchSelection;
use crate::batch_report::BatchCompareReport;
```

Add:

```rust
pub fn draw_batch_detail(ui: &mut eframe::egui::Ui, report: Option<&BatchCompareReport>, selection: Option<&BatchSelection>) {
    let Some(report) = report else {
        ui.label("Run directory compare to inspect a batch item.");
        return;
    };
    let Some(selection) = selection else {
        ui.label("Select a batch item to inspect details.");
        return;
    };

    match selection {
        BatchSelection::Identical(index) => {
            let item = &report.identical[*index];
            ui.label(format!("File: {}", item.pair.file_name));
            ui.label(format!("Left: {}", item.pair.left.absolute_path.display()));
            ui.label(format!("Right: {}", item.pair.right.absolute_path.display()));
            ui.label("Metadata identical");
        }
        BatchSelection::Different(index) => {
            let item = &report.different[*index];
            ui.label(format!("File: {}", item.pair.file_name));
            ui.label(format!("Left: {}", item.pair.left.absolute_path.display()));
            ui.label(format!("Right: {}", item.pair.right.absolute_path.display()));
            ui.label(format!("Differences: {}", item.summary.total()));
        }
        BatchSelection::LeftOnly(index) => {
            let item = &report.left_only[*index];
            ui.label(format!("File: {}", item.file.file_name));
            ui.label(format!("Path: {}", item.file.absolute_path.display()));
            ui.label(format!("Reason: {}", item.reason));
        }
        BatchSelection::RightOnly(index) => {
            let item = &report.right_only[*index];
            ui.label(format!("File: {}", item.file.file_name));
            ui.label(format!("Path: {}", item.file.absolute_path.display()));
            ui.label(format!("Reason: {}", item.reason));
        }
    }
}
```

Update `src/app.rs` so summary/detail panes switch behavior based on `mode`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test batch_section_labels_include_counts -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/ui/summary.rs src/ui/detail.rs
git commit -m "feat: add batch report navigation and details"
```

### Task 6: Integrate directory mode into the main UI and reuse the current diff view for `Different`

**Files:**
- Modify: `src/app.rs`
- Modify: `src/ui/tree.rs`
- Modify: `src/ui/detail.rs`

- [ ] **Step 1: Write the failing mode-switch test**

Append this test to `src/app.rs`:

```rust
#[test]
fn directory_mode_prefers_batch_selection_over_single_file_result() {
    let mut app = PngMetadataCompareApp::default();
    app.mode = AppMode::Directory;
    assert!(matches!(app.mode, AppMode::Directory));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test directory_mode_prefers_batch_selection_over_single_file_result -- --nocapture`

Expected: FAIL if mode wiring is incomplete

- [ ] **Step 3: Wire directory-mode GUI flow**

Update `src/app.rs`:

- Top bar:
  - add radio/selectable mode switch
  - show file pickers in single-file mode
  - show directory pickers in directory mode
  - call `run_compare()` in single-file mode
  - call `run_directory_compare()` in directory mode

- Summary pane:
  - single-file mode uses current summary rendering
  - directory mode uses `draw_batch_summary`

- Bottom detail pane:
  - single-file mode uses current detail renderer
  - directory mode uses `draw_batch_detail` for `Identical`, `Left Only`, `Right Only`
  - if selection is `Different`, continue showing current per-pair detail

- Central panel:
  - single-file mode behaves as before
  - directory mode:
    - empty prompt before compare
    - if selected batch item is `Different`, render the existing diff tree for that pair
    - if selected batch item is not `Different`, render a simple placeholder like `No diff tree for this item type`

Update `src/ui/tree.rs` to add:

```rust
pub fn draw_diff_tree_for_batch_item(
    ui: &mut eframe::egui::Ui,
    root: &crate::diff::DiffNode,
    selected_path: &mut Option<String>,
    filters: &TreeFilters,
) {
    if !should_show(root, filters) {
        ui.label("No diff nodes match the current filters.");
        return;
    }

    eframe::egui::ScrollArea::vertical().show(ui, |ui| {
        draw_node(ui, root, selected_path, filters);
    });
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test directory_mode_prefers_batch_selection_over_single_file_result -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/ui/tree.rs src/ui/detail.rs
git commit -m "feat: integrate directory mode into the GUI"
```

### Task 7: Final regression pass for identical, different, left-only, and right-only categories

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs` (only if exact-command compatibility shim is required)

- [ ] **Step 1: Write the failing batch-regression tests**

Append these tests to `src/app.rs`:

```rust
#[test]
fn directory_mode_selects_different_item_for_diff_tree_when_available() {
    let fixture = BatchDirFixture::new(
        &[("same.png", "x", r#"{"StopName":"A"}"#), ("diff.png", "x", r#"{"StopName":"B"}"#)],
        &[("same.png", "x", r#"{"StopName":"A"}"#), ("diff.png", "x", r#"{"StopName":"C"}"#)],
    );

    let mut app = PngMetadataCompareApp::default();
    app.mode = AppMode::Directory;
    app.left_dir = Some(fixture.left_dir.display().to_string());
    app.right_dir = Some(fixture.right_dir.display().to_string());
    app.run_directory_compare();

    assert!(matches!(app.batch_selection, Some(BatchSelection::Different(0))));
}

#[test]
fn directory_mode_keeps_identical_results_when_no_differences_exist() {
    let fixture = BatchDirFixture::new(
        &[("same.png", "x", r#"{"StopName":"A"}"#)],
        &[("same.png", "x", r#"{"StopName":"A"}"#)],
    );

    let mut app = PngMetadataCompareApp::default();
    app.mode = AppMode::Directory;
    app.left_dir = Some(fixture.left_dir.display().to_string());
    app.right_dir = Some(fixture.right_dir.display().to_string());
    app.run_directory_compare();

    let report = app.batch_report.as_ref().unwrap();
    assert_eq!(report.identical.len(), 1);
    assert!(report.different.is_empty());
    assert!(matches!(app.batch_selection, Some(BatchSelection::Identical(0))));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test directory_mode_ -- --nocapture`

Expected: FAIL if selection defaults or category wiring are incomplete

- [ ] **Step 3: Run the full suite and desktop app**

If tests fail, fix the minimal issue in `src/app.rs` and rerun.

Run:

```bash
cargo test
```

Expected: PASS

Run:

```bash
cargo run
```

Expected:

- the app opens
- users can switch between `Single File` and `Directory`
- directory mode allows selecting two folders
- identical pairs appear under `Identical`
- metadata-different pairs appear under `Different`
- unmatched files appear under `Left Only` / `Right Only`
- clicking a `Different` item shows the diff tree and detail view

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "test: verify directory batch compare flow"
```
