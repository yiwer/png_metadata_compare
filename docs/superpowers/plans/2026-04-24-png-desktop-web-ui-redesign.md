# PNG Desktop Web UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current `eframe` desktop UI with a Tauri-hosted Web UI that preserves single-file compare and directory batch compare while adding persistent image preview, metadata inspection, and raw JSON inspection.

**Architecture:** Keep the current Rust compare logic as the source of truth, extract it into UI-independent inspection services, then expose a thin Tauri command bridge to a React/Vite frontend. The frontend becomes a MotherDuck-inspired desktop workbench with a result rail, persistent side-by-side image preview, tabbed analysis views, and a synchronized inspector.

**Tech Stack:** Rust 2024, Tauri 2, serde/serde_json, existing in-repo compare modules, React, TypeScript, Vite, Vitest, Testing Library

---

## File Structure

Create or modify these files during implementation:

- Modify: `Cargo.toml`
- Create: `build.rs`
- Create: `tauri.conf.json`
- Create: `src/lib.rs`
- Modify: `src/main.rs`
- Create: `src/inspection.rs`
- Create: `src/desktop_api.rs`
- Modify: `src/error.rs`
- Modify: `src/diff.rs`
- Modify: `src/batch_report.rs`
- Modify: `src/batch_scan.rs`
- Modify: `src/metadata.rs`
- Create: `frontend/package.json`
- Create: `frontend/tsconfig.json`
- Create: `frontend/vite.config.ts`
- Create: `frontend/index.html`
- Create: `frontend/src/main.tsx`
- Create: `frontend/src/App.tsx`
- Create: `frontend/src/lib/api.ts`
- Create: `frontend/src/lib/types.ts`
- Create: `frontend/src/features/workbench/useWorkbench.ts`
- Create: `frontend/src/components/Toolbar.tsx`
- Create: `frontend/src/components/ResultRail.tsx`
- Create: `frontend/src/components/PreviewStrip.tsx`
- Create: `frontend/src/components/TabBar.tsx`
- Create: `frontend/src/components/DiffTree.tsx`
- Create: `frontend/src/components/MetadataTree.tsx`
- Create: `frontend/src/components/RawJsonPanel.tsx`
- Create: `frontend/src/components/ImagePane.tsx`
- Create: `frontend/src/components/InspectorPanel.tsx`
- Create: `frontend/src/components/StatusBanner.tsx`
- Create: `frontend/src/components/EmptyState.tsx`
- Create: `frontend/src/styles/tokens.css`
- Create: `frontend/src/styles/app.css`
- Create: `frontend/src/test/setup.ts`
- Modify: `.gitignore`

The file responsibilities are:

- `src/lib.rs`: crate root for shared Rust modules and testable services.
- `src/inspection.rs`: UI-independent inspection services and serializable DTOs for single compare, directory summary, and single-side inspection.
- `src/desktop_api.rs`: Tauri commands that convert string paths into service calls.
- `src/main.rs`: Tauri bootstrap with Windows no-console behavior.
- `src/error.rs`: error normalization helpers for frontend-safe payloads.
- `src/diff.rs`, `src/batch_report.rs`, `src/batch_scan.rs`: serializable domain models reused by the Web UI.
- `frontend/src/lib/types.ts`: shared frontend TypeScript models for backend payloads.
- `frontend/src/lib/api.ts`: all Tauri `invoke` wrappers in one place.
- `frontend/src/features/workbench/useWorkbench.ts`: state orchestration for mode, selection, loading, tabs, and inspector sync.
- `frontend/src/components/*`: focused presentation units for the workbench.
- `frontend/src/styles/*`: MotherDuck-inspired tokens and global layout styling.

## Task 1: Extract serializable Rust inspection services from the current GUI state

**Files:**
- Create: `src/lib.rs`
- Create: `src/inspection.rs`
- Modify: `src/error.rs`
- Modify: `src/diff.rs`
- Modify: `src/batch_report.rs`

- [ ] **Step 1: Write the failing service tests**

Create `src/inspection.rs` with these tests first:

```rust
#[cfg(test)]
mod tests {
    use super::{inspect_pair, inspect_single_side, scan_directory_summary, BatchListItemKind};
    use crate::batch_report::UnmatchedSide;
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
        assert!(payload.items.iter().any(|item| item.kind == BatchListItemKind::Different));
        assert!(payload.items.iter().any(|item| item.kind == BatchListItemKind::LeftOnly));
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test inspection::tests -- --nocapture`

Expected: FAIL because `inspect_pair`, `inspect_single_side`, `scan_directory_summary`, and DTO types do not exist

- [ ] **Step 3: Implement serializable service DTOs and extraction helpers**

Create `src/lib.rs`:

```rust
pub mod batch_report;
pub mod batch_scan;
pub mod diff;
pub mod error;
pub mod inspection;
pub mod metadata;
pub mod png_reader;
```

Update the derives in `src/diff.rs` and `src/batch_report.rs`:

```rust
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffStatus {
    Unchanged,
    Modified,
    Added,
    Removed,
    Reordered,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffNode {
    pub path: String,
    pub status: DiffStatus,
    pub left_value: Option<String>,
    pub right_value: Option<String>,
    pub summary: String,
    pub children: Vec<DiffNode>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiffSummary {
    pub modified: usize,
    pub added: usize,
    pub removed: usize,
    pub reordered: usize,
    pub error: usize,
}
```

Add frontend-safe error payloads to `src/error.rs`:

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UiError {
    pub code: &'static str,
    pub message: String,
}

impl CompareError {
    pub fn ui_code(&self) -> &'static str {
        match self {
            Self::FileRead { .. } => "file_read",
            Self::InvalidPngSignature => "invalid_png_signature",
            Self::TruncatedChunk => "truncated_chunk",
            Self::MissingStopPlateMetadata => "metadata_missing",
            Self::UnsupportedCompressedText => "unsupported_compressed_text",
            Self::InvalidInternationalText(_) => "invalid_itxt",
            Self::MetadataUtf8(_) => "metadata_utf8",
            Self::MetadataJson(_) => "metadata_json",
            Self::AmbiguousBusinessKey { .. } => "ambiguous_business_key",
        }
    }

    pub fn to_ui_error(&self) -> UiError {
        UiError {
            code: self.ui_code(),
            message: self.to_string(),
        }
    }
}
```

Create `src/inspection.rs`:

```rust
use crate::batch_report::{BatchIssue, MatchStrategy, UnmatchedSide};
use crate::batch_scan::scan_png_files_best_effort;
use crate::diff::{compare_metadata, flatten_changes, summarize_changes, DiffNode, DiffSummary};
use crate::error::UiError;
use crate::metadata::{load_metadata, MetadataLoadResult};
use crate::png_reader::extract_stop_plate_metadata_from_file;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct SideInspection {
    pub side: &'static str,
    pub file_path: String,
    pub file_name: String,
    pub raw_json: Option<String>,
    pub metadata: Option<Value>,
    pub error: Option<UiError>,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct BatchCounts {
    pub identical: usize,
    pub different: usize,
    pub left_only: usize,
    pub right_only: usize,
    pub error: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchListItem {
    pub id: String,
    pub kind: BatchListItemKind,
    pub label: String,
    pub left_path: Option<String>,
    pub right_path: Option<String>,
    pub difference_count: usize,
    pub match_strategy: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectorySummary {
    pub counts: BatchCounts,
    pub items: Vec<BatchListItem>,
}

pub fn inspect_pair(left_path: &Path, right_path: &Path) -> PairInspection {
    let left_raw = extract_stop_plate_metadata_from_file(left_path);
    let right_raw = extract_stop_plate_metadata_from_file(right_path);
    let left = side_from_raw("left", left_path, left_raw);
    let right = side_from_raw("right", right_path, right_raw);
    let left_loaded = side_to_metadata_result(&left);
    let right_loaded = side_to_metadata_result(&right);
    let diff_root = compare_metadata(&left_loaded, &right_loaded);
    let change_list = flatten_changes(&diff_root);
    let diff_summary = summarize_changes(&change_list);
    let default_selected_path = change_list
        .iter()
        .find(|node| node.left_value.is_some() || node.right_value.is_some())
        .or_else(|| change_list.first())
        .map(|node| node.path.clone());

    PairInspection {
        left,
        right,
        diff_root,
        diff_summary,
        default_selected_path,
    }
}

pub fn inspect_single_side(path: &Path, side: UnmatchedSide) -> SideInspection {
    let label = match side {
        UnmatchedSide::Left => "left",
        UnmatchedSide::Right => "right",
    };
    side_from_raw(label, path, extract_stop_plate_metadata_from_file(path))
}

pub fn scan_directory_summary(left_dir: &Path, right_dir: &Path) -> DirectorySummary {
    let left_scan = scan_png_files_best_effort(left_dir);
    let right_scan = scan_png_files_best_effort(right_dir);
    let mut counts = BatchCounts {
        identical: 0,
        different: 0,
        left_only: 0,
        right_only: 0,
        error: left_scan.issues.len() + right_scan.issues.len(),
    };
    let mut items = Vec::new();
    let mut pairing = crate::batch_scan::build_pairing(&left_scan.files, &right_scan.files);
    if right_scan.root_scan_failed {
        pairing.left_only.clear();
    }
    if left_scan.root_scan_failed {
        pairing.right_only.clear();
    }

    for pair in pairing.matched {
        let inspection = inspect_pair(&pair.left.absolute_path, &pair.right.absolute_path);
        let difference_count = inspection.diff_summary.total();
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
            label: pair.file_name.clone(),
            left_path: Some(pair.left.absolute_path.display().to_string()),
            right_path: Some(pair.right.absolute_path.display().to_string()),
            difference_count,
            match_strategy: Some(match pair.match_strategy {
                MatchStrategy::FileName => "file_name".to_string(),
                MatchStrategy::FileNameAndParentDir => "file_name_and_parent_dir".to_string(),
            }),
            message: None,
        });
    }

    for item in pairing.left_only {
        counts.left_only += 1;
        items.push(BatchListItem {
            id: format!("left-only::{}", item.file.absolute_path.display()),
            kind: BatchListItemKind::LeftOnly,
            label: item.file.file_name.clone(),
            left_path: Some(item.file.absolute_path.display().to_string()),
            right_path: None,
            difference_count: 0,
            match_strategy: None,
            message: Some(item.reason),
        });
    }

    for item in pairing.right_only {
        counts.right_only += 1;
        items.push(BatchListItem {
            id: format!("right-only::{}", item.file.absolute_path.display()),
            kind: BatchListItemKind::RightOnly,
            label: item.file.file_name.clone(),
            left_path: None,
            right_path: Some(item.file.absolute_path.display().to_string()),
            difference_count: 0,
            match_strategy: None,
            message: Some(item.reason),
        });
    }

    for issue in left_scan
        .issues
        .into_iter()
        .chain(right_scan.issues.into_iter())
    {
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

fn side_from_raw(side: &'static str, path: &Path, raw: Result<String, crate::error::CompareError>) -> SideInspection {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    match raw {
        Ok(raw_json) => match serde_json::from_str::<Value>(&raw_json) {
            Ok(metadata) => SideInspection {
                side,
                file_path: path.display().to_string(),
                file_name,
                raw_json: Some(raw_json),
                metadata: Some(metadata),
                error: None,
            },
            Err(error) => SideInspection {
                side,
                file_path: path.display().to_string(),
                file_name,
                raw_json: Some(raw_json),
                metadata: None,
                error: Some(crate::error::CompareError::MetadataJson(error.to_string()).to_ui_error()),
            },
        },
        Err(error) => SideInspection {
            side,
            file_path: path.display().to_string(),
            file_name,
            raw_json: None,
            metadata: None,
            error: Some(error.to_ui_error()),
        },
    }
}

fn side_to_metadata_result(side: &SideInspection) -> MetadataLoadResult {
    match (&side.raw_json, &side.metadata, &side.error) {
        (_, Some(metadata), _) => MetadataLoadResult::Parsed(metadata.clone()),
        (_, _, Some(error)) => MetadataLoadResult::Error(ui_error_to_compare_error(error)),
        _ => MetadataLoadResult::Error(crate::error::CompareError::MissingStopPlateMetadata),
    }
}

fn ui_error_to_compare_error(error: &UiError) -> crate::error::CompareError {
    match error.code {
        "file_read" => crate::error::CompareError::FileRead {
            path: std::path::PathBuf::from("<ui-path>"),
            reason: error.message.clone(),
        },
        "invalid_png_signature" => crate::error::CompareError::InvalidPngSignature,
        "truncated_chunk" => crate::error::CompareError::TruncatedChunk,
        "metadata_missing" => crate::error::CompareError::MissingStopPlateMetadata,
        "unsupported_compressed_text" => crate::error::CompareError::UnsupportedCompressedText,
        "invalid_itxt" => crate::error::CompareError::InvalidInternationalText(error.message.clone()),
        "metadata_utf8" => crate::error::CompareError::MetadataUtf8(error.message.clone()),
        "ambiguous_business_key" => crate::error::CompareError::AmbiguousBusinessKey {
            path: "<ui-path>".into(),
            key: error.message.clone(),
        },
        _ => crate::error::CompareError::MetadataJson(error.message.clone()),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test inspection::tests -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/lib.rs src/inspection.rs src/error.rs src/diff.rs src/batch_report.rs
git commit -m "feat: extract serializable inspection services"
```

## Task 2: Replace the native `eframe` shell with a Tauri 2 desktop shell and command bridge

**Files:**
- Modify: `Cargo.toml`
- Create: `build.rs`
- Create: `tauri.conf.json`
- Modify: `src/main.rs`
- Create: `src/desktop_api.rs`

- [ ] **Step 1: Write the failing command tests**

Create `src/desktop_api.rs` with these tests first:

```rust
#[cfg(test)]
mod tests {
    use super::{compare_single, inspect_single, scan_directory};
    use crate::batch_report::UnmatchedSide;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn compare_single_command_returns_serializable_payload() {
        let fixture = Fixture::new("compare_single");
        let left = fixture.write_png("left.png", r#"{"Title":"Left"}"#);
        let right = fixture.write_png("right.png", r#"{"Title":"Right"}"#);

        let payload = compare_single(left.display().to_string(), right.display().to_string()).unwrap();
        let json = serde_json::to_value(&payload).unwrap();

        assert_eq!(json["left"]["file_path"], left.display().to_string());
        assert_eq!(json["right"]["file_path"], right.display().to_string());
    }

    #[test]
    fn scan_directory_command_returns_counts() {
        let fixture = BatchFixture::new("scan_directory");
        fixture.write_left_png("same.png", "shared", r#"{"Title":"Same"}"#);
        fixture.write_right_png("same.png", "shared", r#"{"Title":"Same"}"#);

        let payload = scan_directory(
            fixture.left_dir.display().to_string(),
            fixture.right_dir.display().to_string(),
        )
        .unwrap();

        assert_eq!(payload.counts.identical, 1);
    }

    #[test]
    fn inspect_single_command_respects_side_argument() {
        let fixture = Fixture::new("inspect_single");
        let left = fixture.write_png("left.png", r#"{"Title":"Left"}"#);
        let payload = inspect_single(left.display().to_string(), "left".into()).unwrap();
        assert_eq!(payload.side, "left");
    }

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "png_metadata_compare_desktop_api_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn write_png(&self, name: &str, json: &str) -> PathBuf {
            let path = self.root.join(name);
            fs::write(&path, super::tests_support::png_with_metadata(json)).unwrap();
            path
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct BatchFixture {
        root: PathBuf,
        left_dir: PathBuf,
        right_dir: PathBuf,
    }

    impl BatchFixture {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "png_metadata_compare_desktop_api_batch_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            let left_dir = root.join("left");
            let right_dir = root.join("right");
            fs::create_dir_all(&left_dir).unwrap();
            fs::create_dir_all(&right_dir).unwrap();
            Self {
                root,
                left_dir,
                right_dir,
            }
        }

        fn write_left_png(&self, name: &str, dir: &str, json: &str) {
            let folder = self.left_dir.join(dir);
            fs::create_dir_all(&folder).unwrap();
            fs::write(folder.join(name), super::tests_support::png_with_metadata(json)).unwrap();
        }

        fn write_right_png(&self, name: &str, dir: &str, json: &str) {
            let folder = self.right_dir.join(dir);
            fs::create_dir_all(&folder).unwrap();
            fs::write(folder.join(name), super::tests_support::png_with_metadata(json)).unwrap();
        }
    }

    impl Drop for BatchFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test desktop_api::tests -- --nocapture`

Expected: FAIL because `compare_single`, `scan_directory`, and `inspect_single` do not exist

- [ ] **Step 3: Implement the Tauri command bridge and desktop bootstrap**

Update `Cargo.toml`:

```toml
[package]
name = "png_metadata_compare"
version = "0.1.0"
edition = "2024"
build = "build.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-opener = "2"
thiserror = "2"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

Create `build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

Create `tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "PNG Metadata Compare",
  "version": "0.1.0",
  "identifier": "com.local.png-metadata-compare",
  "build": {
    "beforeDevCommand": "npm --prefix frontend run dev",
    "beforeBuildCommand": "npm --prefix frontend run build",
    "frontendDist": "frontend/dist",
    "devUrl": "http://localhost:5173"
  },
  "app": {
    "windows": [
      {
        "title": "PNG Metadata Compare",
        "width": 1560,
        "height": 980,
        "minWidth": 1200,
        "minHeight": 760,
        "resizable": true
      }
    ]
  }
}
```

Create `src/desktop_api.rs`:

```rust
use crate::batch_report::UnmatchedSide;
use crate::inspection::{inspect_pair, inspect_single_side, scan_directory_summary, DirectorySummary, PairInspection, SideInspection};
use std::path::PathBuf;

#[tauri::command]
pub fn compare_single(left_path: String, right_path: String) -> Result<PairInspection, String> {
    Ok(inspect_pair(&PathBuf::from(left_path), &PathBuf::from(right_path)))
}

#[tauri::command]
pub fn scan_directory(left_dir: String, right_dir: String) -> Result<DirectorySummary, String> {
    Ok(scan_directory_summary(&PathBuf::from(left_dir), &PathBuf::from(right_dir)))
}

#[tauri::command]
pub fn inspect_single(path: String, side: String) -> Result<SideInspection, String> {
    let side = match side.as_str() {
        "left" => UnmatchedSide::Left,
        "right" => UnmatchedSide::Right,
        _ => return Err(format!("unsupported side: {side}")),
    };
    Ok(inspect_single_side(&PathBuf::from(path), side))
}

#[cfg(test)]
mod tests_support {
    pub fn png_with_metadata(json: &str) -> Vec<u8> {
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
```

Replace `src/main.rs` with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use png_metadata_compare::desktop_api::{compare_single, inspect_single, scan_directory};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            compare_single,
            scan_directory,
            inspect_single
        ])
        .run(tauri::generate_context!())
        .expect("failed to launch tauri desktop shell");
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test desktop_api::tests -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add Cargo.toml build.rs tauri.conf.json src/main.rs src/desktop_api.rs
git commit -m "feat: add tauri desktop shell and command bridge"
```

## Task 3: Scaffold the React/Vite frontend and MotherDuck-inspired shell layout

**Files:**
- Create: `frontend/package.json`
- Create: `frontend/tsconfig.json`
- Create: `frontend/vite.config.ts`
- Create: `frontend/index.html`
- Create: `frontend/src/main.tsx`
- Create: `frontend/src/App.tsx`
- Create: `frontend/src/styles/tokens.css`
- Create: `frontend/src/styles/app.css`
- Create: `frontend/src/test/setup.ts`
- Modify: `.gitignore`

- [ ] **Step 1: Write the failing frontend shell test**

Create `frontend/src/App.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import App from "./App";

it("renders the desktop workbench shell", () => {
  render(<App />);

  expect(screen.getByText("PNG Metadata Compare")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: /single file/i })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: /directory/i })).toBeInTheDocument();
  expect(screen.getByText("Diff")).toBeInTheDocument();
  expect(screen.getByText("Left Metadata")).toBeInTheDocument();
  expect(screen.getByText("Right Metadata")).toBeInTheDocument();
  expect(screen.getByText("Raw JSON")).toBeInTheDocument();
  expect(screen.getByText("Images")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm --prefix frontend test -- --run`

Expected: FAIL because the frontend workspace does not exist yet

- [ ] **Step 3: Create the frontend workspace and static workbench shell**

Create `frontend/package.json`:

```json
{
  "name": "png-metadata-compare-frontend",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "test": "vitest"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-opener": "^2.0.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  },
  "devDependencies": {
    "@testing-library/jest-dom": "^6.4.5",
    "@testing-library/react": "^15.0.7",
    "@types/react": "^18.3.3",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.1",
    "typescript": "^5.5.4",
    "vite": "^5.4.1",
    "vitest": "^2.0.5"
  }
}
```

Create `frontend/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["DOM", "DOM.Iterable", "ES2020"],
    "allowJs": false,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "module": "ESNext",
    "moduleResolution": "Node",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx"
  },
  "include": ["src"],
  "references": []
}
```

Create `frontend/vite.config.ts`:

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
  },
});
```

Create `frontend/src/test/setup.ts`:

```ts
import "@testing-library/jest-dom";
```

Create `frontend/index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>PNG Metadata Compare</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

Create `frontend/src/main.tsx`:

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/tokens.css";
import "./styles/app.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

Create `frontend/src/App.tsx`:

```tsx
export default function App() {
  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">
          <span className="brand__title">PNG Metadata Compare</span>
          <span className="brand__badge">Desktop Workbench</span>
        </div>
        <div className="mode-toggle">
          <button className="chip chip--active">Single File</button>
          <button className="chip">Directory</button>
        </div>
      </header>

      <div className="toolbar">
        <button className="button button--primary">Choose Left</button>
        <button className="button">Choose Right</button>
        <button className="button button--ghost">Compare</button>
      </div>

      <div className="workspace">
        <aside className="rail">
          <div className="panel-title">Results</div>
          <div className="empty-copy">Run compare to populate the result rail.</div>
        </aside>

        <main className="main">
          <section className="preview-strip">
            <article className="preview-card">
              <div className="preview-card__header">Left PNG</div>
              <div className="preview-card__body">Image Preview</div>
            </article>
            <article className="preview-card">
              <div className="preview-card__header">Right PNG</div>
              <div className="preview-card__body">Image Preview</div>
            </article>
          </section>

          <section className="tabs">
            <button className="tab tab--active">Diff</button>
            <button className="tab">Left Metadata</button>
            <button className="tab">Right Metadata</button>
            <button className="tab">Raw JSON</button>
            <button className="tab">Images</button>
          </section>

          <section className="content-grid">
            <article className="panel">
              <div className="panel__header">Main View</div>
              <div className="panel__body">Choose files or folders to begin.</div>
            </article>
            <article className="panel panel--dark">
              <div className="panel__header">Inspector</div>
              <div className="panel__body">Selected node details will appear here.</div>
            </article>
          </section>
        </main>
      </div>
    </div>
  );
}
```

Create `frontend/src/styles/tokens.css`:

```css
:root {
  --color-bg: #f4efea;
  --color-surface: #f8f8f7;
  --color-card: #ffffff;
  --color-text: #383838;
  --color-yellow: #ffde00;
  --color-blue: #6fc2ff;
  --color-red: #ff7169;
  --color-green: #22c55e;
  --border-heavy: 3px solid var(--color-text);
  --border-regular: 2px solid var(--color-text);
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --font-ui: "IBM Plex Mono", "Fira Mono", monospace;
}
```

Create `frontend/src/styles/app.css`:

```css
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: radial-gradient(circle at top left, #fff6bf 0, transparent 28%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.3), rgba(255, 255, 255, 0.3)),
    var(--color-bg);
  color: var(--color-text);
  font-family: var(--font-ui);
}

.shell {
  min-height: 100vh;
  display: grid;
  grid-template-rows: auto auto 1fr;
}

.topbar,
.toolbar,
.workspace,
.preview-strip,
.content-grid {
  gap: var(--space-4);
}

.topbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-4) var(--space-6);
  background: var(--color-yellow);
  border-bottom: var(--border-heavy);
}

.brand__badge,
.chip,
.button,
.tab {
  text-transform: uppercase;
  border: var(--border-regular);
  background: var(--color-card);
  padding: 10px 14px;
  font: inherit;
}

.chip--active,
.button--primary,
.tab--active,
.preview-card__header,
.panel__header {
  background: var(--color-blue);
}

.workspace {
  display: grid;
  grid-template-columns: 320px minmax(0, 1fr);
  min-height: 0;
}

.rail {
  border-right: var(--border-heavy);
  background: var(--color-surface);
  padding: var(--space-4);
}

.main {
  display: grid;
  grid-template-rows: 240px auto 1fr;
  min-height: 0;
}

.preview-strip,
.content-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  padding: var(--space-4);
}

.preview-card,
.panel {
  border: var(--border-heavy);
  background: var(--color-card);
}

.preview-card__header,
.panel__header {
  padding: var(--space-3) var(--space-4);
  border-bottom: var(--border-heavy);
}

.preview-card__body,
.panel__body {
  padding: var(--space-4);
  min-height: 120px;
}

.panel--dark .panel__header {
  background: var(--color-text);
  color: white;
}
```

Update `.gitignore`:

```gitignore
/target
/frontend/node_modules
/frontend/dist
/.tauri
/.superpowers
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm --prefix frontend test -- --run`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add frontend .gitignore
git commit -m "feat: scaffold desktop web workbench shell"
```

## Task 4: Add typed frontend API bindings and workbench state orchestration

**Files:**
- Create: `frontend/src/lib/types.ts`
- Create: `frontend/src/lib/api.ts`
- Create: `frontend/src/features/workbench/useWorkbench.ts`
- Modify: `frontend/src/App.tsx`

- [ ] **Step 1: Write the failing workbench state test**

Create `frontend/src/features/workbench/useWorkbench.test.tsx`:

```tsx
import { act, renderHook } from "@testing-library/react";
import { useWorkbench } from "./useWorkbench";

const mockApi = {
  compareSingle: vi.fn(),
  scanDirectory: vi.fn(),
  inspectSingle: vi.fn(),
};

it("loads a single compare result into the active inspection view", async () => {
  mockApi.compareSingle.mockResolvedValue({
    left: { side: "left", file_path: "left.png", file_name: "left.png", raw_json: "{}", metadata: {}, error: null },
    right: { side: "right", file_path: "right.png", file_name: "right.png", raw_json: "{}", metadata: {}, error: null },
    diff_root: { path: "StopPlateMetadata", status: "modified", left_value: null, right_value: null, summary: "changed", children: [] },
    diff_summary: { modified: 1, added: 0, removed: 0, reordered: 0, error: 0 },
    default_selected_path: "StopPlateMetadata"
  });

  const { result } = renderHook(() => useWorkbench(mockApi as never));

  await act(async () => {
    result.current.setMode("single");
    result.current.setLeftInput("left.png");
    result.current.setRightInput("right.png");
    await result.current.runCompare();
  });

  expect(result.current.activeInspection?.diff_summary.modified).toBe(1);
  expect(result.current.activeTab).toBe("diff");
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm --prefix frontend test -- --run useWorkbench`

Expected: FAIL because the hook and types do not exist

- [ ] **Step 3: Implement the Tauri client and synchronized workbench state**

Create `frontend/src/lib/types.ts`:

```ts
export type DiffStatus =
  | "unchanged"
  | "modified"
  | "added"
  | "removed"
  | "reordered"
  | "error";

export interface UiError {
  code: string;
  message: string;
}

export interface SideInspection {
  side: "left" | "right";
  file_path: string;
  file_name: string;
  raw_json: string | null;
  metadata: unknown | null;
  error: UiError | null;
}

export interface DiffNode {
  path: string;
  status: DiffStatus;
  left_value: string | null;
  right_value: string | null;
  summary: string;
  children: DiffNode[];
}

export interface DiffSummary {
  modified: number;
  added: number;
  removed: number;
  reordered: number;
  error: number;
}

export interface PairInspection {
  left: SideInspection;
  right: SideInspection;
  diff_root: DiffNode;
  diff_summary: DiffSummary;
  default_selected_path: string | null;
}

export type BatchListItemKind = "identical" | "different" | "left_only" | "right_only" | "error";

export interface BatchCounts {
  identical: number;
  different: number;
  left_only: number;
  right_only: number;
  error: number;
}

export interface BatchListItem {
  id: string;
  kind: BatchListItemKind;
  label: string;
  left_path: string | null;
  right_path: string | null;
  difference_count: number;
  match_strategy: string | null;
  message: string | null;
}

export interface DirectorySummary {
  counts: BatchCounts;
  items: BatchListItem[];
}
```

Create `frontend/src/lib/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { DirectorySummary, PairInspection, SideInspection } from "./types";

export interface WorkbenchApi {
  compareSingle(leftPath: string, rightPath: string): Promise<PairInspection>;
  scanDirectory(leftDir: string, rightDir: string): Promise<DirectorySummary>;
  inspectSingle(path: string, side: "left" | "right"): Promise<SideInspection>;
}

export const workbenchApi: WorkbenchApi = {
  compareSingle(leftPath, rightPath) {
    return invoke<PairInspection>("compare_single", { leftPath, rightPath });
  },
  scanDirectory(leftDir, rightDir) {
    return invoke<DirectorySummary>("scan_directory", { leftDir, rightDir });
  },
  inspectSingle(path, side) {
    return invoke<SideInspection>("inspect_single", { path, side });
  },
};
```

Create `frontend/src/features/workbench/useWorkbench.ts`:

```ts
import { useState } from "react";
import type { BatchListItem, DirectorySummary, PairInspection, SideInspection } from "../../lib/types";
import { workbenchApi, type WorkbenchApi } from "../../lib/api";

export type Mode = "single" | "directory";
export type TabKey = "diff" | "left_metadata" | "right_metadata" | "raw_json" | "images";

export function useWorkbench(api: WorkbenchApi = workbenchApi) {
  const [mode, setMode] = useState<Mode>("single");
  const [leftInput, setLeftInput] = useState("");
  const [rightInput, setRightInput] = useState("");
  const [directorySummary, setDirectorySummary] = useState<DirectorySummary | null>(null);
  const [activeItem, setActiveItem] = useState<BatchListItem | null>(null);
  const [activeInspection, setActiveInspection] = useState<PairInspection | null>(null);
  const [activeSingleSide, setActiveSingleSide] = useState<SideInspection | null>(null);
  const [activeTab, setActiveTab] = useState<TabKey>("diff");
  const [activeNodePath, setActiveNodePath] = useState<string | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [errorBanner, setErrorBanner] = useState<string | null>(null);

  async function runCompare() {
    setErrorBanner(null);
    setLoading(mode === "single" ? "Comparing PNG files..." : "Scanning directories...");

    try {
      if (mode === "single") {
        const inspection = await api.compareSingle(leftInput, rightInput);
        setDirectorySummary(null);
        setActiveItem(null);
        setActiveSingleSide(null);
        setActiveInspection(inspection);
        setActiveNodePath(inspection.default_selected_path);
        setActiveTab("diff");
      } else {
        const summary = await api.scanDirectory(leftInput, rightInput);
        setDirectorySummary(summary);
        const firstItem = summary.items[0] ?? null;
        setActiveItem(firstItem);
        if (firstItem?.left_path && firstItem.right_path) {
          const inspection = await api.compareSingle(firstItem.left_path, firstItem.right_path);
          setActiveInspection(inspection);
          setActiveSingleSide(null);
          setActiveNodePath(inspection.default_selected_path);
          setActiveTab(firstItem.kind === "different" ? "diff" : "left_metadata");
        } else if (firstItem?.left_path || firstItem?.right_path) {
          const side = firstItem.left_path ? "left" : "right";
          const payload = await api.inspectSingle(firstItem.left_path ?? firstItem.right_path!, side);
          setActiveInspection(null);
          setActiveSingleSide(payload);
          setActiveNodePath(null);
          setActiveTab(side === "left" ? "left_metadata" : "right_metadata");
        }
      }
    } catch (error) {
      setErrorBanner(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(null);
    }
  }

  async function selectResult(item: BatchListItem) {
    setActiveItem(item);
    setErrorBanner(null);
    setLoading(`Loading ${item.label}...`);
    try {
      if (item.left_path && item.right_path) {
        const inspection = await api.compareSingle(item.left_path, item.right_path);
        setActiveInspection(inspection);
        setActiveSingleSide(null);
        setActiveNodePath(inspection.default_selected_path);
        setActiveTab(item.kind === "different" ? "diff" : "left_metadata");
      } else if (item.left_path || item.right_path) {
        const side = item.left_path ? "left" : "right";
        const payload = await api.inspectSingle(item.left_path ?? item.right_path!, side);
        setActiveInspection(null);
        setActiveSingleSide(payload);
        setActiveNodePath(null);
        setActiveTab(side === "left" ? "left_metadata" : "right_metadata");
      } else {
        setActiveInspection(null);
        setActiveSingleSide(null);
        setActiveTab("raw_json");
      }
    } catch (error) {
      setErrorBanner(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(null);
    }
  }

  return {
    mode,
    leftInput,
    rightInput,
    directorySummary,
    activeItem,
    activeInspection,
    activeSingleSide,
    activeTab,
    activeNodePath,
    loading,
    errorBanner,
    setMode,
    setLeftInput,
    setRightInput,
    setActiveTab,
    setActiveNodePath,
    runCompare,
    selectResult,
  };
}
```

Update `frontend/src/App.tsx`:

```tsx
import { useWorkbench } from "./features/workbench/useWorkbench";

export default function App() {
  const workbench = useWorkbench();

  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">
          <span className="brand__title">PNG Metadata Compare</span>
          <span className="brand__badge">Desktop Workbench</span>
        </div>
        <div className="mode-toggle">
          <button
            className={workbench.mode === "single" ? "chip chip--active" : "chip"}
            onClick={() => workbench.setMode("single")}
          >
            Single File
          </button>
          <button
            className={workbench.mode === "directory" ? "chip chip--active" : "chip"}
            onClick={() => workbench.setMode("directory")}
          >
            Directory
          </button>
        </div>
      </header>

      <div className="toolbar">
        <input
          className="input"
          value={workbench.leftInput}
          onChange={(event) => workbench.setLeftInput(event.target.value)}
          placeholder={workbench.mode === "single" ? "Left PNG path" : "Left directory path"}
        />
        <input
          className="input"
          value={workbench.rightInput}
          onChange={(event) => workbench.setRightInput(event.target.value)}
          placeholder={workbench.mode === "single" ? "Right PNG path" : "Right directory path"}
        />
        <button className="button button--ghost" onClick={() => void workbench.runCompare()}>
          Compare
        </button>
      </div>

      <div className="workspace">
        <aside className="rail">
          <div className="panel-title">Results</div>
          {workbench.directorySummary ? (
            <div className="empty-copy">{workbench.directorySummary.items.length} items loaded.</div>
          ) : (
            <div className="empty-copy">Run compare to populate the result rail.</div>
          )}
        </aside>

        <main className="main">
          <section className="preview-strip">
            <article className="preview-card">
              <div className="preview-card__header">Left PNG</div>
              <div className="preview-card__body">
                {workbench.activeInspection?.left.file_name ?? workbench.activeSingleSide?.file_name ?? "Image Preview"}
              </div>
            </article>
            <article className="preview-card">
              <div className="preview-card__header">Right PNG</div>
              <div className="preview-card__body">
                {workbench.activeInspection?.right.file_name ?? "Image Preview"}
              </div>
            </article>
          </section>

          <section className="tabs">
            {[
              ["diff", "Diff"],
              ["left_metadata", "Left Metadata"],
              ["right_metadata", "Right Metadata"],
              ["raw_json", "Raw JSON"],
              ["images", "Images"],
            ].map(([key, label]) => (
              <button
                key={key}
                className={workbench.activeTab === key ? "tab tab--active" : "tab"}
                onClick={() => workbench.setActiveTab(key as never)}
              >
                {label}
              </button>
            ))}
          </section>

          <section className="content-grid">
            <article className="panel">
              <div className="panel__header">Main View</div>
              <div className="panel__body">
                {workbench.loading ?? "Results will render here after compare completes."}
              </div>
            </article>
            <article className="panel panel--dark">
              <div className="panel__header">Inspector</div>
              <div className="panel__body">{workbench.activeNodePath ?? "No node selected."}</div>
            </article>
          </section>
        </main>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm --prefix frontend test -- --run useWorkbench`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add frontend/src/lib frontend/src/features frontend/src/App.tsx
git commit -m "feat: add typed workbench state and tauri API bindings"
```

## Task 5: Render the result rail, preview strip, diff tree, metadata tree, raw JSON, and synchronized inspector

**Files:**
- Create: `frontend/src/components/Toolbar.tsx`
- Create: `frontend/src/components/ResultRail.tsx`
- Create: `frontend/src/components/PreviewStrip.tsx`
- Create: `frontend/src/components/TabBar.tsx`
- Create: `frontend/src/components/DiffTree.tsx`
- Create: `frontend/src/components/MetadataTree.tsx`
- Create: `frontend/src/components/RawJsonPanel.tsx`
- Create: `frontend/src/components/ImagePane.tsx`
- Create: `frontend/src/components/InspectorPanel.tsx`
- Modify: `frontend/src/App.tsx`

- [ ] **Step 1: Write the failing synchronization test**

Create `frontend/src/components/workbench-sync.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { PairInspection } from "../lib/types";
import { DiffTree } from "./DiffTree";
import { InspectorPanel } from "./InspectorPanel";

const inspection: PairInspection = {
  left: { side: "left", file_path: "left.png", file_name: "left.png", raw_json: "{\"Title\":\"Left\"}", metadata: { Title: "Left" }, error: null },
  right: { side: "right", file_path: "right.png", file_name: "right.png", raw_json: "{\"Title\":\"Right\"}", metadata: { Title: "Right" }, error: null },
  diff_root: {
    path: "StopPlateMetadata",
    status: "modified",
    left_value: null,
    right_value: null,
    summary: "changed",
    children: [
      {
        path: "Title",
        status: "modified",
        left_value: "\"Left\"",
        right_value: "\"Right\"",
        summary: "Title changed",
        children: [],
      },
    ],
  },
  diff_summary: { modified: 1, added: 0, removed: 0, reordered: 0, error: 0 },
  default_selected_path: "Title",
};

it("updates the inspector when a diff node is selected", async () => {
  const user = userEvent.setup();
  let activePath = inspection.default_selected_path;
  const setActivePath = vi.fn((next: string) => {
    activePath = next;
  });

  const { rerender } = render(
    <>
      <DiffTree root={inspection.diff_root} activePath={activePath} onSelect={setActivePath} />
      <InspectorPanel inspection={inspection} activePath={activePath} activeTab="diff" />
    </>,
  );

  await user.click(screen.getByRole("button", { name: /title changed/i }));
  rerender(
    <>
      <DiffTree root={inspection.diff_root} activePath="Title" onSelect={setActivePath} />
      <InspectorPanel inspection={inspection} activePath="Title" activeTab="diff" />
    </>,
  );

  expect(screen.getByText("Selected Path")).toBeInTheDocument();
  expect(screen.getByText("Title")).toBeInTheDocument();
  expect(screen.getByText("\"Left\"")).toBeInTheDocument();
  expect(screen.getByText("\"Right\"")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm --prefix frontend test -- --run workbench-sync`

Expected: FAIL because the render components do not exist

- [ ] **Step 3: Implement focused UI components and wire them into `App.tsx`**

Create `frontend/src/components/DiffTree.tsx`:

```tsx
import type { DiffNode } from "../lib/types";

interface DiffTreeProps {
  root: DiffNode;
  activePath: string | null;
  onSelect(path: string): void;
}

export function DiffTree({ root, activePath, onSelect }: DiffTreeProps) {
  return (
    <div className="tree">
      <TreeNode node={root} activePath={activePath} onSelect={onSelect} />
    </div>
  );
}

function TreeNode({
  node,
  activePath,
  onSelect,
}: {
  node: DiffNode;
  activePath: string | null;
  onSelect(path: string): void;
}) {
  return (
    <div className="tree__node">
      <button
        className={activePath === node.path ? "tree__button tree__button--active" : "tree__button"}
        onClick={() => onSelect(node.path)}
      >
        {node.summary}
      </button>
      {node.children.length > 0 ? (
        <div className="tree__children">
          {node.children.map((child) => (
            <TreeNode key={child.path} node={child} activePath={activePath} onSelect={onSelect} />
          ))}
        </div>
      ) : null}
    </div>
  );
}
```

Create `frontend/src/components/MetadataTree.tsx`:

```tsx
interface MetadataTreeProps {
  value: unknown;
  prefix?: string;
  activePath: string | null;
  onSelect(path: string, value: unknown): void;
}

export function MetadataTree({ value, prefix = "", activePath, onSelect }: MetadataTreeProps) {
  if (Array.isArray(value)) {
    return (
      <div className="json-tree">
        {value.map((entry, index) => (
          <MetadataTree
            key={`${prefix}[${index}]`}
            value={entry}
            prefix={`${prefix}[${index}]`}
            activePath={activePath}
            onSelect={onSelect}
          />
        ))}
      </div>
    );
  }

  if (value && typeof value === "object") {
    return (
      <div className="json-tree">
        {Object.entries(value as Record<string, unknown>).map(([key, child]) => {
          const path = prefix ? `${prefix}.${key}` : key;
          return (
            <div key={path} className="json-tree__branch">
              <button
                className={activePath === path ? "tree__button tree__button--active" : "tree__button"}
                onClick={() => onSelect(path, child)}
              >
                {path}
              </button>
              <MetadataTree value={child} prefix={path} activePath={activePath} onSelect={onSelect} />
            </div>
          );
        })}
      </div>
    );
  }

  const label = prefix || "value";
  return (
    <button
      className={activePath === label ? "tree__button tree__button--active" : "tree__button"}
      onClick={() => onSelect(label, value)}
    >
      {label}: {String(value)}
    </button>
  );
}
```

Create `frontend/src/components/RawJsonPanel.tsx`:

```tsx
export function RawJsonPanel({
  leftRaw,
  rightRaw,
}: {
  leftRaw: string | null | undefined;
  rightRaw: string | null | undefined;
}) {
  return (
    <div className="raw-grid">
      <pre className="code-block">{leftRaw ?? "No left JSON available."}</pre>
      <pre className="code-block">{rightRaw ?? "No right JSON available."}</pre>
    </div>
  );
}
```

Create `frontend/src/components/ImagePane.tsx`:

```tsx
function toImageSrc(path: string) {
  return path ? `asset://${encodeURIComponent(path)}` : "";
}

export function ImagePane({ leftPath, rightPath }: { leftPath?: string; rightPath?: string }) {
  return (
    <div className="raw-grid">
      <div className="image-frame">{leftPath ? <img alt="Left PNG" src={toImageSrc(leftPath)} /> : "No left image"}</div>
      <div className="image-frame">{rightPath ? <img alt="Right PNG" src={toImageSrc(rightPath)} /> : "No right image"}</div>
    </div>
  );
}
```

Create `frontend/src/components/InspectorPanel.tsx`:

```tsx
import type { PairInspection } from "../lib/types";

export function InspectorPanel({
  inspection,
  activePath,
  activeTab,
}: {
  inspection: PairInspection | null;
  activePath: string | null;
  activeTab: "diff" | "left_metadata" | "right_metadata" | "raw_json" | "images";
}) {
  const node = inspection && activePath ? findNode(inspection.diff_root, activePath) : null;

  return (
    <div className="inspector">
      <div className="inspector__label">Selected Path</div>
      <div className="inspector__value">{activePath ?? "None"}</div>
      {activeTab === "diff" && node ? (
        <>
          <div className="inspector__label">Left Value</div>
          <div className="inspector__value">{node.left_value ?? "<none>"}</div>
          <div className="inspector__label">Right Value</div>
          <div className="inspector__value">{node.right_value ?? "<none>"}</div>
        </>
      ) : null}
    </div>
  );
}

function findNode(node: PairInspection["diff_root"], path: string): PairInspection["diff_root"] | null {
  if (node.path === path) {
    return node;
  }
  for (const child of node.children) {
    const found = findNode(child, path);
    if (found) {
      return found;
    }
  }
  return null;
}
```

Update `frontend/src/App.tsx` so the main panel switches by tab:

```tsx
import { DiffTree } from "./components/DiffTree";
import { MetadataTree } from "./components/MetadataTree";
import { RawJsonPanel } from "./components/RawJsonPanel";
import { ImagePane } from "./components/ImagePane";
import { InspectorPanel } from "./components/InspectorPanel";

// inside the component body, replace the content grid section with:
<section className="content-grid">
  <article className="panel">
    <div className="panel__header">Main View</div>
    <div className="panel__body">
      {workbench.activeTab === "diff" && workbench.activeInspection ? (
        <DiffTree
          root={workbench.activeInspection.diff_root}
          activePath={workbench.activeNodePath}
          onSelect={workbench.setActiveNodePath}
        />
      ) : null}

      {workbench.activeTab === "left_metadata" && workbench.activeInspection?.left.metadata ? (
        <MetadataTree
          value={workbench.activeInspection.left.metadata}
          activePath={workbench.activeNodePath}
          onSelect={(path) => workbench.setActiveNodePath(path)}
        />
      ) : null}

      {workbench.activeTab === "right_metadata" && workbench.activeInspection?.right.metadata ? (
        <MetadataTree
          value={workbench.activeInspection.right.metadata}
          activePath={workbench.activeNodePath}
          onSelect={(path) => workbench.setActiveNodePath(path)}
        />
      ) : null}

      {workbench.activeTab === "raw_json" ? (
        <RawJsonPanel
          leftRaw={workbench.activeInspection?.left.raw_json}
          rightRaw={workbench.activeInspection?.right.raw_json}
        />
      ) : null}

      {workbench.activeTab === "images" ? (
        <ImagePane
          leftPath={workbench.activeInspection?.left.file_path ?? workbench.activeSingleSide?.file_path}
          rightPath={workbench.activeInspection?.right.file_path}
        />
      ) : null}
    </div>
  </article>

  <article className="panel panel--dark">
    <div className="panel__header">Inspector</div>
    <div className="panel__body">
      <InspectorPanel
        inspection={workbench.activeInspection}
        activePath={workbench.activeNodePath}
        activeTab={workbench.activeTab}
      />
    </div>
  </article>
</section>
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npm --prefix frontend test -- --run workbench-sync`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add frontend/src/components frontend/src/App.tsx
git commit -m "feat: render tabbed inspection views and synchronized inspector"
```

## Task 6: Add file dialogs, open-image actions, loading and error states, then verify the desktop build

**Files:**
- Create: `frontend/src/components/Toolbar.tsx`
- Create: `frontend/src/components/ResultRail.tsx`
- Create: `frontend/src/components/PreviewStrip.tsx`
- Create: `frontend/src/components/TabBar.tsx`
- Create: `frontend/src/components/StatusBanner.tsx`
- Create: `frontend/src/components/EmptyState.tsx`
- Modify: `frontend/src/App.tsx`
- Modify: `frontend/src/styles/app.css`

- [ ] **Step 1: Write the failing interaction test**

Create `frontend/src/components/toolbar-flow.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Toolbar } from "./Toolbar";

it("disables compare until both paths are present", async () => {
  const user = userEvent.setup();
  const props = {
    mode: "single" as const,
    leftInput: "",
    rightInput: "",
    onLeftInputChange: vi.fn(),
    onRightInputChange: vi.fn(),
    onPickLeft: vi.fn(),
    onPickRight: vi.fn(),
    onCompare: vi.fn(),
  };

  render(<Toolbar {...props} />);
  expect(screen.getByRole("button", { name: /compare/i })).toBeDisabled();

  await user.click(screen.getByRole("button", { name: /choose left/i }));
  expect(props.onPickLeft).toHaveBeenCalled();
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm --prefix frontend test -- --run toolbar-flow`

Expected: FAIL because `Toolbar` does not exist

- [ ] **Step 3: Implement production interactions and final app composition**

Create `frontend/src/components/Toolbar.tsx`:

```tsx
interface ToolbarProps {
  mode: "single" | "directory";
  leftInput: string;
  rightInput: string;
  onLeftInputChange(value: string): void;
  onRightInputChange(value: string): void;
  onPickLeft(): void;
  onPickRight(): void;
  onCompare(): void;
}

export function Toolbar(props: ToolbarProps) {
  const compareDisabled = !props.leftInput || !props.rightInput;

  return (
    <div className="toolbar">
      <button className="button button--primary" onClick={props.onPickLeft}>
        {props.mode === "single" ? "Choose Left PNG" : "Choose Left Folder"}
      </button>
      <input
        className="input"
        value={props.leftInput}
        onChange={(event) => props.onLeftInputChange(event.target.value)}
      />
      <button className="button" onClick={props.onPickRight}>
        {props.mode === "single" ? "Choose Right PNG" : "Choose Right Folder"}
      </button>
      <input
        className="input"
        value={props.rightInput}
        onChange={(event) => props.onRightInputChange(event.target.value)}
      />
      <button className="button button--ghost" disabled={compareDisabled} onClick={props.onCompare}>
        Compare
      </button>
    </div>
  );
}
```

Create `frontend/src/components/ResultRail.tsx`:

```tsx
import type { BatchListItem, BatchCounts } from "../lib/types";

export function ResultRail({
  counts,
  items,
  activeId,
  onSelect,
}: {
  counts: BatchCounts | null;
  items: BatchListItem[];
  activeId: string | null;
  onSelect(item: BatchListItem): void;
}) {
  return (
    <aside className="rail">
      <div className="panel-title">Results</div>
      {counts ? (
        <div className="count-grid">
          <span>Different: {counts.different}</span>
          <span>Identical: {counts.identical}</span>
          <span>Left Only: {counts.left_only}</span>
          <span>Right Only: {counts.right_only}</span>
          <span>Errors: {counts.error}</span>
        </div>
      ) : (
        <div className="empty-copy">Run compare to populate the result rail.</div>
      )}

      <div className="result-list">
        {items.map((item) => (
          <button
            key={item.id}
            className={activeId === item.id ? "result-row result-row--active" : "result-row"}
            onClick={() => onSelect(item)}
          >
            <span>{item.label}</span>
            <span>{item.kind}</span>
          </button>
        ))}
      </div>
    </aside>
  );
}
```

Create `frontend/src/components/PreviewStrip.tsx`:

```tsx
export function PreviewStrip({
  leftLabel,
  rightLabel,
  onOpenImages,
}: {
  leftLabel: string;
  rightLabel: string;
  onOpenImages(): void;
}) {
  return (
    <section className="preview-strip">
      <article className="preview-card" onClick={onOpenImages}>
        <div className="preview-card__header">Left PNG</div>
        <div className="preview-card__body">{leftLabel}</div>
      </article>
      <article className="preview-card" onClick={onOpenImages}>
        <div className="preview-card__header">Right PNG</div>
        <div className="preview-card__body">{rightLabel}</div>
      </article>
    </section>
  );
}
```

Create `frontend/src/components/TabBar.tsx`:

```tsx
const tabs = [
  ["diff", "Diff"],
  ["left_metadata", "Left Metadata"],
  ["right_metadata", "Right Metadata"],
  ["raw_json", "Raw JSON"],
  ["images", "Images"],
] as const;

export function TabBar({
  activeTab,
  onSelect,
}: {
  activeTab: "diff" | "left_metadata" | "right_metadata" | "raw_json" | "images";
  onSelect(tab: "diff" | "left_metadata" | "right_metadata" | "raw_json" | "images"): void;
}) {
  return (
    <section className="tabs">
      {tabs.map(([key, label]) => (
        <button
          key={key}
          className={activeTab === key ? "tab tab--active" : "tab"}
          onClick={() => onSelect(key)}
        >
          {label}
        </button>
      ))}
    </section>
  );
}
```

Create `frontend/src/components/StatusBanner.tsx`:

```tsx
export function StatusBanner({ loading, error }: { loading: string | null; error: string | null }) {
  if (error) {
    return <div className="status-banner status-banner--error">{error}</div>;
  }
  if (loading) {
    return <div className="status-banner">{loading}</div>;
  }
  return null;
}
```

Create `frontend/src/components/EmptyState.tsx`:

```tsx
export function EmptyState({ title, body }: { title: string; body: string }) {
  return (
    <div className="empty-state">
      <h2>{title}</h2>
      <p>{body}</p>
    </div>
  );
}
```

Update `frontend/src/App.tsx` to use dialog and opener plugins:

```tsx
import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { EmptyState } from "./components/EmptyState";
import { PreviewStrip } from "./components/PreviewStrip";
import { ResultRail } from "./components/ResultRail";
import { StatusBanner } from "./components/StatusBanner";
import { TabBar } from "./components/TabBar";
import { Toolbar } from "./components/Toolbar";

async function pickPath(directory: boolean) {
  const selected = await open({
    directory,
    multiple: false,
    filters: directory ? undefined : [{ name: "PNG", extensions: ["png"] }],
  });
  return typeof selected === "string" ? selected : "";
}

// inside App():
<StatusBanner loading={workbench.loading} error={workbench.errorBanner} />

<Toolbar
  mode={workbench.mode}
  leftInput={workbench.leftInput}
  rightInput={workbench.rightInput}
  onLeftInputChange={workbench.setLeftInput}
  onRightInputChange={workbench.setRightInput}
  onPickLeft={async () => {
    const picked = await pickPath(workbench.mode === "directory");
    if (picked) workbench.setLeftInput(picked);
  }}
  onPickRight={async () => {
    const picked = await pickPath(workbench.mode === "directory");
    if (picked) workbench.setRightInput(picked);
  }}
  onCompare={() => void workbench.runCompare()}
/>

<div className="workspace">
  <ResultRail
    counts={workbench.directorySummary?.counts ?? null}
    items={workbench.directorySummary?.items ?? []}
    activeId={workbench.activeItem?.id ?? null}
    onSelect={(item) => void workbench.selectResult(item)}
  />

  <main className="main">
    <PreviewStrip
      leftLabel={workbench.activeInspection?.left.file_name ?? workbench.activeSingleSide?.file_name ?? "No left PNG selected"}
      rightLabel={workbench.activeInspection?.right.file_name ?? "No right PNG selected"}
      onOpenImages={() => workbench.setActiveTab("images")}
    />
    <TabBar activeTab={workbench.activeTab} onSelect={workbench.setActiveTab} />

    {!workbench.activeInspection && !workbench.activeSingleSide ? (
      <EmptyState
        title="Choose inputs and run compare"
        body="The workbench will show result navigation, image previews, metadata, and raw JSON here."
      />
    ) : null}
  </main>
</div>
```

Extend `frontend/src/styles/app.css`:

```css
.status-banner {
  padding: var(--space-3) var(--space-6);
  background: var(--color-yellow);
  border-bottom: var(--border-heavy);
}

.status-banner--error {
  background: var(--color-red);
  color: white;
}

.input {
  min-height: 44px;
  padding: 0 14px;
  border: var(--border-regular);
  background: var(--color-card);
  font: inherit;
}

.count-grid,
.result-list {
  display: grid;
  gap: var(--space-3);
  margin-top: var(--space-4);
}

.result-row {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border: var(--border-regular);
  background: var(--color-card);
  font: inherit;
}

.result-row--active {
  background: var(--color-yellow);
}

.empty-state {
  margin: var(--space-4);
  padding: var(--space-6);
  border: var(--border-heavy);
  background: var(--color-card);
}

.tree__button {
  width: 100%;
  text-align: left;
  border: var(--border-regular);
  background: var(--color-card);
  padding: var(--space-3);
  font: inherit;
}

.tree__button--active {
  background: var(--color-blue);
}

.tree__children,
.json-tree,
.raw-grid {
  display: grid;
  gap: var(--space-3);
}

.code-block,
.inspector__value,
.image-frame {
  border: var(--border-regular);
  background: var(--color-surface);
  padding: var(--space-4);
}
```

- [ ] **Step 4: Run the full verification set**

Run:

```bash
cargo test
npm --prefix frontend test -- --run
npm --prefix frontend run build
cargo check
npx tauri build --debug
```

Expected:

- `cargo test`: PASS
- `npm --prefix frontend test -- --run`: PASS
- `npm --prefix frontend run build`: PASS and writes `frontend/dist`
- `cargo check`: PASS
- `npx tauri build --debug`: PASS and produces a Windows desktop build that opens without a console window

Then run:

```bash
npx tauri dev
```

Expected manual behavior:

- the app opens as a desktop window with no extra command shell
- single-file mode can compare two PNGs
- directory mode can scan two folders and populate the result rail
- selecting a result updates previews, tabs, and inspector together
- `Diff`, `Left Metadata`, `Right Metadata`, `Raw JSON`, and `Images` all render meaningful content
- unmatched items remain inspectable instead of collapsing to blank UI

- [ ] **Step 5: Commit**

Run:

```bash
git add frontend/src/components frontend/src/styles frontend/src/App.tsx
git commit -m "feat: finish desktop web workbench interactions"
```
