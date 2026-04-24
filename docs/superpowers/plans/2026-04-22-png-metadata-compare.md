# PNG Metadata Compare Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows-native Rust GUI that loads two PNG files, extracts `StopPlateMetadata` from PNG `iTXt` chunks, compares the metadata by business-aware rules, and renders a structured diff tree plus a change summary.

**Architecture:** The app is split into a pure comparison core and a thin `egui/eframe` desktop shell. `png_reader` extracts raw metadata text, `metadata` parses it into a generic JSON representation, `diff` computes a tree of changes plus a flat summary, and `app` / `ui/*` render and navigate the result.

**Tech Stack:** Rust 2024, `eframe/egui`, `rfd`, `serde_json`, `thiserror`

---

## File Structure

Create or modify these files during implementation:

- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Create: `src/error.rs`
- Create: `src/png_reader.rs`
- Create: `src/metadata.rs`
- Create: `src/diff.rs`
- Create: `src/app.rs`
- Create: `src/ui/summary.rs`
- Create: `src/ui/tree.rs`
- Create: `src/ui/detail.rs`

The file responsibilities are:

- `src/error.rs`: shared error types for file IO, PNG extraction, metadata parsing, and diff ambiguity.
- `src/png_reader.rs`: PNG signature check, chunk iteration, and `StopPlateMetadata` extraction from `iTXt`.
- `src/metadata.rs`: metadata load result model built on `serde_json::Value`.
- `src/diff.rs`: diff node model, business-key matching, recursive compare, flattened change list.
- `src/app.rs`: application state, file-selection actions, compare trigger, filter state.
- `src/ui/summary.rs`: summary counts and clickable change list rendering.
- `src/ui/tree.rs`: tree rendering and selection updates.
- `src/ui/detail.rs`: selected-node detail pane rendering.
- `src/main.rs`: module wiring and `eframe` boot.

### Task 1: Bootstrap the crate for GUI + pure-core modules

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Create: `src/error.rs`
- Create: `src/app.rs`
- Create: `src/diff.rs`
- Create: `src/metadata.rs`
- Create: `src/png_reader.rs`
- Create: `src/ui/summary.rs`
- Create: `src/ui/tree.rs`
- Create: `src/ui/detail.rs`

- [ ] **Step 1: Write the failing smoke test for app state**

Add this test at the bottom of `src/app.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::PngMetadataCompareApp;

    #[test]
    fn compare_is_disabled_until_both_paths_are_present() {
        let mut app = PngMetadataCompareApp::default();
        assert!(!app.can_compare());

        app.left_path = Some("left.png".into());
        assert!(!app.can_compare());

        app.right_path = Some("right.png".into());
        assert!(app.can_compare());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test compare_is_disabled_until_both_paths_are_present -- --exact`

Expected: FAIL with unresolved module or missing `PngMetadataCompareApp`

- [ ] **Step 3: Add dependencies and minimal app shell**

Update `Cargo.toml`:

```toml
[package]
name = "png_metadata_compare"
version = "0.1.0"
edition = "2024"

[dependencies]
eframe = "0.33"
rfd = "0.15"
serde_json = "1"
thiserror = "2"
```

Replace `src/main.rs` with:

```rust
mod app;
mod diff;
mod error;
mod metadata;
mod png_reader;
mod ui {
    pub mod detail;
    pub mod summary;
    pub mod tree;
}

use app::PngMetadataCompareApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "PNG Metadata Compare",
        options,
        Box::new(|_cc| Ok(Box::new(PngMetadataCompareApp::default()))),
    )
}
```

Create `src/error.rs`:

```rust
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum CompareError {
    #[error("failed to read file: {path}")]
    FileRead { path: PathBuf, reason: String },
    #[error("invalid PNG signature")]
    InvalidPngSignature,
    #[error("PNG chunk layout is truncated")]
    TruncatedChunk,
    #[error("StopPlateMetadata was not found")]
    MissingStopPlateMetadata,
    #[error("unsupported compressed iTXt metadata")]
    UnsupportedCompressedText,
    #[error("invalid iTXt chunk structure: {0}")]
    InvalidInternationalText(String),
    #[error("metadata is not valid UTF-8: {0}")]
    MetadataUtf8(String),
    #[error("metadata JSON is invalid: {0}")]
    MetadataJson(String),
    #[error("ambiguous business key at {path}: {key}")]
    AmbiguousBusinessKey { path: String, key: String },
}
```

Create `src/app.rs`:

```rust
#[derive(Default)]
pub struct PngMetadataCompareApp {
    pub left_path: Option<String>,
    pub right_path: Option<String>,
}

impl PngMetadataCompareApp {
    pub fn can_compare(&self) -> bool {
        self.left_path.is_some() && self.right_path.is_some()
    }
}

impl eframe::App for PngMetadataCompareApp {
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}
}
```

Create `src/diff.rs`:

```rust
// Placeholder module. Task 4 replaces this file with the diff engine.
```

Create `src/metadata.rs`:

```rust
// Placeholder module. Task 3 replaces this file with the metadata loader.
```

Create `src/png_reader.rs`:

```rust
// Placeholder module. Task 2 replaces this file with the PNG reader.
```

Create `src/ui/summary.rs`:

```rust
pub fn draw_summary_placeholder(ui: &mut egui::Ui) {
    ui.label("Summary will appear here.");
}
```

Create `src/ui/tree.rs`:

```rust
pub fn draw_tree_placeholder(ui: &mut egui::Ui) {
    ui.label("Diff tree will appear here.");
}
```

Create `src/ui/detail.rs`:

```rust
pub fn draw_detail_placeholder(ui: &mut egui::Ui) {
    ui.label("Details will appear here.");
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test compare_is_disabled_until_both_paths_are_present -- --exact`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add Cargo.toml src/main.rs src/error.rs src/app.rs src/diff.rs src/metadata.rs src/png_reader.rs src/ui/summary.rs src/ui/tree.rs src/ui/detail.rs
git commit -m "chore: scaffold GUI application shell"
```

### Task 2: Implement PNG metadata extraction with test-generated fixtures

**Files:**
- Create: `src/png_reader.rs`
- Test: `src/png_reader.rs`

- [ ] **Step 1: Write the failing extraction tests**

Create `src/png_reader.rs` with these tests first:

```rust
#[cfg(test)]
mod tests {
    use super::extract_stop_plate_metadata;
    use crate::error::CompareError;

    fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes
    }

    fn png_with_chunks(chunks: Vec<Vec<u8>>) -> Vec<u8> {
        let mut bytes = vec![137, 80, 78, 71, 13, 10, 26, 10];
        for chunk in chunks {
            bytes.extend_from_slice(&chunk);
        }
        bytes
    }

    fn stop_plate_itxt(json: &str) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"StopPlateMetadata");
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.extend_from_slice(json.as_bytes());
        chunk(b"iTXt", &data)
    }

    #[test]
    fn extracts_stop_plate_metadata_text() {
        let png = png_with_chunks(vec![stop_plate_itxt(r#"{"StopName":"A"}"#)]);
        let text = extract_stop_plate_metadata(&png).unwrap();
        assert_eq!(text, r#"{"StopName":"A"}"#);
    }

    #[test]
    fn returns_missing_metadata_when_keyword_is_absent() {
        let png = png_with_chunks(vec![chunk(b"IDAT", &[1, 2, 3])]);
        let err = extract_stop_plate_metadata(&png).unwrap_err();
        assert!(matches!(err, CompareError::MissingStopPlateMetadata));
    }

    #[test]
    fn rejects_invalid_png_signature() {
        let err = extract_stop_plate_metadata(b"not-a-png").unwrap_err();
        assert!(matches!(err, CompareError::InvalidPngSignature));
    }

    #[test]
    fn rejects_compressed_itxt_metadata() {
        let mut data = Vec::new();
        data.extend_from_slice(b"StopPlateMetadata");
        data.push(0);
        data.push(1);
        data.push(0);
        data.push(0);
        data.push(0);
        data.extend_from_slice(br#"{"StopName":"A"}"#);
        let png = png_with_chunks(vec![chunk(b"iTXt", &data)]);

        let err = extract_stop_plate_metadata(&png).unwrap_err();
        assert!(matches!(err, CompareError::UnsupportedCompressedText));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test png_reader -- --nocapture`

Expected: FAIL because `extract_stop_plate_metadata` is not implemented

- [ ] **Step 3: Write the minimal extraction implementation**

Replace `src/png_reader.rs` with:

```rust
use crate::error::CompareError;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const STOP_PLATE_KEYWORD: &str = "StopPlateMetadata";

pub fn extract_stop_plate_metadata(bytes: &[u8]) -> Result<String, CompareError> {
    if bytes.len() < PNG_SIGNATURE.len() || &bytes[..8] != PNG_SIGNATURE {
        return Err(CompareError::InvalidPngSignature);
    }

    let mut pos = 8usize;
    while pos + 8 <= bytes.len() {
        if pos + 8 > bytes.len() {
            return Err(CompareError::TruncatedChunk);
        }

        let length = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        let chunk_type = &bytes[pos + 4..pos + 8];
        let data_start = pos + 8;
        let data_end = data_start + length;
        let crc_end = data_end + 4;

        if crc_end > bytes.len() {
            return Err(CompareError::TruncatedChunk);
        }

        if chunk_type == b"iTXt" {
            let chunk = &bytes[data_start..data_end];
            if let Some(json) = parse_stop_plate_itxt(chunk)? {
                return Ok(json);
            }
        }

        pos = crc_end;
    }

    Err(CompareError::MissingStopPlateMetadata)
}

fn parse_stop_plate_itxt(chunk: &[u8]) -> Result<Option<String>, CompareError> {
    let Some(keyword_end) = chunk.iter().position(|&b| b == 0) else {
        return Err(CompareError::InvalidInternationalText(
            "keyword terminator missing".into(),
        ));
    };

    let keyword = std::str::from_utf8(&chunk[..keyword_end])
        .map_err(|err| CompareError::InvalidInternationalText(err.to_string()))?;
    if keyword != STOP_PLATE_KEYWORD {
        return Ok(None);
    }

    if chunk.len() < keyword_end + 5 {
        return Err(CompareError::InvalidInternationalText(
            "iTXt control bytes missing".into(),
        ));
    }

    let compression_flag = chunk[keyword_end + 1];
    if compression_flag != 0 {
        return Err(CompareError::UnsupportedCompressedText);
    }

    let mut cursor = keyword_end + 3;
    while cursor < chunk.len() && chunk[cursor] != 0 {
        cursor += 1;
    }
    if cursor >= chunk.len() {
        return Err(CompareError::InvalidInternationalText(
            "language tag terminator missing".into(),
        ));
    }
    cursor += 1;

    while cursor < chunk.len() && chunk[cursor] != 0 {
        cursor += 1;
    }
    if cursor >= chunk.len() {
        return Err(CompareError::InvalidInternationalText(
            "translated keyword terminator missing".into(),
        ));
    }
    cursor += 1;

    String::from_utf8(chunk[cursor..].to_vec())
        .map(Some)
        .map_err(|err| CompareError::MetadataUtf8(err.to_string()))
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test png_reader -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/png_reader.rs
git commit -m "feat: extract StopPlateMetadata from PNG iTXt chunks"
```

### Task 3: Parse metadata into a diffable model and surface load failures

**Files:**
- Create: `src/metadata.rs`
- Test: `src/metadata.rs`

- [ ] **Step 1: Write the failing metadata loader tests**

Create `src/metadata.rs` with:

```rust
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum MetadataLoadResult {
    Parsed(Value),
    Error(crate::error::CompareError),
}

#[cfg(test)]
mod tests {
    use super::{load_metadata, MetadataLoadResult};
    use crate::error::CompareError;

    #[test]
    fn parses_valid_json_into_value() {
        let result = load_metadata(Ok(r#"{"StopName":"A","Lines":[]}"#.into()));
        match result {
            MetadataLoadResult::Parsed(value) => assert_eq!(value["StopName"], "A"),
            MetadataLoadResult::Error(err) => panic!("unexpected error: {err}"),
        }
    }

    #[test]
    fn converts_json_failure_to_error_result() {
        let result = load_metadata(Ok("{".into()));
        assert!(matches!(result, MetadataLoadResult::Error(CompareError::MetadataJson(_))));
    }

    #[test]
    fn preserves_upstream_extraction_errors() {
        let result = load_metadata(Err(CompareError::MissingStopPlateMetadata));
        assert!(matches!(
            result,
            MetadataLoadResult::Error(CompareError::MissingStopPlateMetadata)
        ));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test metadata -- --nocapture`

Expected: FAIL because `load_metadata` is missing

- [ ] **Step 3: Write the minimal metadata loader**

Replace `src/metadata.rs` with:

```rust
use crate::error::CompareError;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum MetadataLoadResult {
    Parsed(Value),
    Error(CompareError),
}

pub fn load_metadata(raw: Result<String, CompareError>) -> MetadataLoadResult {
    match raw {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(value) => MetadataLoadResult::Parsed(value),
            Err(err) => MetadataLoadResult::Error(CompareError::MetadataJson(err.to_string())),
        },
        Err(err) => MetadataLoadResult::Error(err),
    }
}

#[cfg(test)]
mod tests {
    use super::{load_metadata, MetadataLoadResult};
    use crate::error::CompareError;

    #[test]
    fn parses_valid_json_into_value() {
        let result = load_metadata(Ok(r#"{"StopName":"A","Lines":[]}"#.into()));
        match result {
            MetadataLoadResult::Parsed(value) => assert_eq!(value["StopName"], "A"),
            MetadataLoadResult::Error(err) => panic!("unexpected error: {err}"),
        }
    }

    #[test]
    fn converts_json_failure_to_error_result() {
        let result = load_metadata(Ok("{".into()));
        assert!(matches!(result, MetadataLoadResult::Error(CompareError::MetadataJson(_))));
    }

    #[test]
    fn preserves_upstream_extraction_errors() {
        let result = load_metadata(Err(CompareError::MissingStopPlateMetadata));
        assert!(matches!(
            result,
            MetadataLoadResult::Error(CompareError::MissingStopPlateMetadata)
        ));
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test metadata -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/metadata.rs
git commit -m "feat: add metadata load result model"
```

### Task 4: Build the recursive diff tree for scalars, objects, and errors

**Files:**
- Create: `src/diff.rs`
- Test: `src/diff.rs`

- [ ] **Step 1: Write the failing core diff tests**

Create `src/diff.rs` with:

```rust
use crate::metadata::MetadataLoadResult;

#[cfg(test)]
mod tests {
    use super::{compare_metadata, DiffStatus};
    use crate::error::CompareError;
    use crate::metadata::MetadataLoadResult;
    use serde_json::json;

    #[test]
    fn marks_scalar_change_as_modified() {
        let left = MetadataLoadResult::Parsed(json!({"StopName": "A"}));
        let right = MetadataLoadResult::Parsed(json!({"StopName": "B"}));

        let diff = compare_metadata(&left, &right);
        let child = diff.children.iter().find(|n| n.path == "StopName").unwrap();
        assert_eq!(child.status, DiffStatus::Modified);
        assert_eq!(child.left_value.as_deref(), Some(r#""A""#));
        assert_eq!(child.right_value.as_deref(), Some(r#""B""#));
    }

    #[test]
    fn marks_missing_field_as_added() {
        let left = MetadataLoadResult::Parsed(json!({"StopName": "A"}));
        let right = MetadataLoadResult::Parsed(json!({"StopName": "A", "FrameSize": "1050x1660"}));

        let diff = compare_metadata(&left, &right);
        let child = diff.children.iter().find(|n| n.path == "FrameSize").unwrap();
        assert_eq!(child.status, DiffStatus::Added);
    }

    #[test]
    fn turns_load_error_into_error_node() {
        let left = MetadataLoadResult::Error(CompareError::MissingStopPlateMetadata);
        let right = MetadataLoadResult::Parsed(json!({"StopName": "A"}));

        let diff = compare_metadata(&left, &right);
        assert_eq!(diff.status, DiffStatus::Error);
        assert!(diff.summary.contains("StopPlateMetadata was not found"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test diff::tests -- --nocapture`

Expected: FAIL because diff types and `compare_metadata` are missing

- [ ] **Step 3: Implement the base diff model and recursive compare**

Replace `src/diff.rs` with:

```rust
use crate::error::CompareError;
use crate::metadata::MetadataLoadResult;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStatus {
    Unchanged,
    Modified,
    Added,
    Removed,
    Reordered,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffNode {
    pub path: String,
    pub status: DiffStatus,
    pub left_value: Option<String>,
    pub right_value: Option<String>,
    pub summary: String,
    pub children: Vec<DiffNode>,
}

pub fn compare_metadata(left: &MetadataLoadResult, right: &MetadataLoadResult) -> DiffNode {
    match (left, right) {
        (MetadataLoadResult::Parsed(left), MetadataLoadResult::Parsed(right)) => {
            compare_value("", left, right)
        }
        (MetadataLoadResult::Error(err), MetadataLoadResult::Parsed(_))
        | (MetadataLoadResult::Parsed(_), MetadataLoadResult::Error(err))
        | (MetadataLoadResult::Error(err), MetadataLoadResult::Error(_)) => error_node("", err),
    }
}

fn compare_value(path: &str, left: &Value, right: &Value) -> DiffNode {
    match (left, right) {
        (Value::Object(left_obj), Value::Object(right_obj)) => {
            let mut keys = BTreeSet::new();
            keys.extend(left_obj.keys().cloned());
            keys.extend(right_obj.keys().cloned());

            let mut children = Vec::new();
            for key in keys {
                let child_path = join_path(path, &key);
                match (left_obj.get(&key), right_obj.get(&key)) {
                    (Some(l), Some(r)) => children.push(compare_value(&child_path, l, r)),
                    (None, Some(r)) => children.push(value_node(&child_path, DiffStatus::Added, None, Some(r))),
                    (Some(l), None) => children.push(value_node(&child_path, DiffStatus::Removed, Some(l), None)),
                    (None, None) => {}
                }
            }

            aggregate_node(path, children)
        }
        (Value::Array(left_arr), Value::Array(right_arr)) => compare_array(path, left_arr, right_arr),
        _ if left == right => value_node(path, DiffStatus::Unchanged, Some(left), Some(right)),
        _ => value_node(path, DiffStatus::Modified, Some(left), Some(right)),
    }
}

fn compare_array(path: &str, left: &[Value], right: &[Value]) -> DiffNode {
    let children = left
        .iter()
        .zip(right.iter())
        .enumerate()
        .map(|(index, (l, r))| compare_value(&format!("{path}[{index}]"), l, r))
        .collect::<Vec<_>>();

    aggregate_node(path, children)
}

fn aggregate_node(path: &str, children: Vec<DiffNode>) -> DiffNode {
    let status = if children.iter().any(|c| c.status == DiffStatus::Error) {
        DiffStatus::Error
    } else if children.iter().any(|c| c.status != DiffStatus::Unchanged) {
        DiffStatus::Modified
    } else {
        DiffStatus::Unchanged
    };

    DiffNode {
        path: if path.is_empty() { "StopPlateMetadata".into() } else { path.into() },
        status,
        left_value: None,
        right_value: None,
        summary: format!("{} child change(s)", children.iter().filter(|c| c.status != DiffStatus::Unchanged).count()),
        children,
    }
}

fn value_node(path: &str, status: DiffStatus, left: Option<&Value>, right: Option<&Value>) -> DiffNode {
    DiffNode {
        path: path.into(),
        status,
        left_value: left.map(compact_json),
        right_value: right.map(compact_json),
        summary: describe_change(path, status, left, right),
        children: Vec::new(),
    }
}

fn error_node(path: &str, error: &CompareError) -> DiffNode {
    DiffNode {
        path: if path.is_empty() { "StopPlateMetadata".into() } else { path.into() },
        status: DiffStatus::Error,
        left_value: None,
        right_value: None,
        summary: error.to_string(),
        children: Vec::new(),
    }
}

fn join_path(parent: &str, field: &str) -> String {
    if parent.is_empty() {
        field.into()
    } else {
        format!("{parent}.{field}")
    }
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap()
}

fn describe_change(path: &str, status: DiffStatus, left: Option<&Value>, right: Option<&Value>) -> String {
    match status {
        DiffStatus::Added => format!("{path} added: {}", right.map(compact_json).unwrap_or_default()),
        DiffStatus::Removed => format!("{path} removed: {}", left.map(compact_json).unwrap_or_default()),
        DiffStatus::Modified => format!(
            "{path} changed: {} -> {}",
            left.map(compact_json).unwrap_or_default(),
            right.map(compact_json).unwrap_or_default()
        ),
        DiffStatus::Unchanged => format!("{path} unchanged"),
        DiffStatus::Reordered => format!("{path} reordered"),
        DiffStatus::Error => format!("{path} error"),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test diff::tests -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/diff.rs
git commit -m "feat: add core recursive diff model"
```

### Task 5: Upgrade array diffing to business-key matching, reorder detection, and change-list flattening

**Files:**
- Modify: `src/diff.rs`
- Test: `src/diff.rs`

- [ ] **Step 1: Write the failing business-key and flattening tests**

Append these tests to `src/diff.rs`:

```rust
#[test]
fn matches_lines_by_line_name_and_direction() {
    let left = MetadataLoadResult::Parsed(serde_json::json!({
        "Lines": [
            {"LineName": "B932", "Direction": "Terminal", "PriceDescription": "1"},
            {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "2"}
        ]
    }));
    let right = MetadataLoadResult::Parsed(serde_json::json!({
        "Lines": [
            {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "3"},
            {"LineName": "B932", "Direction": "Terminal", "PriceDescription": "1"}
        ]
    }));

    let diff = compare_metadata(&left, &right);
    let lines = diff.children.iter().find(|n| n.path == "Lines").unwrap();
    assert!(lines.children.iter().any(|n| n.status == DiffStatus::Reordered));
    assert!(lines.children.iter().any(|n| n.path.contains("M375") && n.status == DiffStatus::Modified));
}

#[test]
fn marks_added_route_stop_when_business_key_only_exists_on_right() {
    let left = MetadataLoadResult::Parsed(serde_json::json!({
        "Lines": [{"LineName": "B932", "Direction": "Terminal", "RouteStops": []}]
    }));
    let right = MetadataLoadResult::Parsed(serde_json::json!({
        "Lines": [{
            "LineName": "B932",
            "Direction": "Terminal",
            "RouteStops": [{"Sequence": 8, "Name": "CurrentStop"}]
        }]
    }));

    let diff = compare_metadata(&left, &right);
    let changes = flatten_changes(&diff);
    assert!(changes.iter().any(|n| n.path.contains("RouteStops[8|CurrentStop]") && n.status == DiffStatus::Added));
}

#[test]
fn creates_error_for_ambiguous_business_key() {
    let left = MetadataLoadResult::Parsed(serde_json::json!({
        "GroupItems": [
            {"SequenceNo": "①", "LineNames": "B932"},
            {"SequenceNo": "①", "LineNames": "M375"}
        ]
    }));
    let right = MetadataLoadResult::Parsed(serde_json::json!({"GroupItems": []}));

    let diff = compare_metadata(&left, &right);
    let group_items = diff.children.iter().find(|n| n.path == "GroupItems").unwrap();
    assert!(group_items.children.iter().any(|n| n.status == DiffStatus::Error));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test diff::tests -- --nocapture`

Expected: FAIL because arrays are still index-based and `flatten_changes` does not exist

- [ ] **Step 3: Replace index-based array comparison with business-key comparison**

Update `src/diff.rs` with these additions and replacements:

```rust
use std::collections::{BTreeMap, BTreeSet};
```

Replace `compare_array` with:

```rust
fn compare_array(path: &str, left: &[Value], right: &[Value]) -> DiffNode {
    if let Some(key_fn) = business_key_for_path(path) {
        return compare_keyed_array(path, left, right, key_fn);
    }

    let max_len = left.len().max(right.len());
    let mut children = Vec::new();
    for index in 0..max_len {
        let child_path = format!("{path}[{index}]");
        match (left.get(index), right.get(index)) {
            (Some(l), Some(r)) => children.push(compare_value(&child_path, l, r)),
            (None, Some(r)) => children.push(value_node(&child_path, DiffStatus::Added, None, Some(r))),
            (Some(l), None) => children.push(value_node(&child_path, DiffStatus::Removed, Some(l), None)),
            (None, None) => {}
        }
    }

    aggregate_node(path, children)
}

fn compare_keyed_array(
    path: &str,
    left: &[Value],
    right: &[Value],
    key_fn: fn(&Value) -> Option<String>,
) -> DiffNode {
    let left_index = build_key_index(path, left, key_fn);
    let right_index = build_key_index(path, right, key_fn);

    if let Err(err) = &left_index {
        return error_node(path, err);
    }
    if let Err(err) = &right_index {
        return error_node(path, err);
    }

    let left_index = left_index.unwrap();
    let right_index = right_index.unwrap();

    let mut keys = BTreeSet::new();
    keys.extend(left_index.keys().cloned());
    keys.extend(right_index.keys().cloned());

    let mut children = Vec::new();
    for key in keys {
        let child_path = format!("{path}[{key}]");
        match (left_index.get(&key), right_index.get(&key)) {
            (Some((left_pos, left_value)), Some((right_pos, right_value))) => {
                let mut node = compare_value(&child_path, left_value, right_value);
                if left_pos != right_pos {
                    node.children.push(DiffNode {
                        path: format!("{child_path}.__order__"),
                        status: DiffStatus::Reordered,
                        left_value: Some(left_pos.to_string()),
                        right_value: Some(right_pos.to_string()),
                        summary: format!("{child_path} reordered: {left_pos} -> {right_pos}"),
                        children: Vec::new(),
                    });
                    if node.status == DiffStatus::Unchanged {
                        node.status = DiffStatus::Modified;
                    }
                }
                children.push(node);
            }
            (Some((_, left_value)), None) => {
                children.push(value_node(&child_path, DiffStatus::Removed, Some(left_value), None));
            }
            (None, Some((_, right_value))) => {
                children.push(value_node(&child_path, DiffStatus::Added, None, Some(right_value)));
            }
            (None, None) => {}
        }
    }

    aggregate_node(path, children)
}

fn build_key_index(
    path: &str,
    values: &[Value],
    key_fn: fn(&Value) -> Option<String>,
) -> Result<BTreeMap<String, (usize, &Value)>, CompareError> {
    let mut index = BTreeMap::new();
    for (position, value) in values.iter().enumerate() {
        let key = key_fn(value).unwrap_or_else(|| position.to_string());
        if index.insert(key.clone(), (position, value)).is_some() {
            return Err(CompareError::AmbiguousBusinessKey {
                path: path.into(),
                key,
            });
        }
    }
    Ok(index)
}

fn business_key_for_path(path: &str) -> Option<fn(&Value) -> Option<String>> {
    if path.ends_with("GroupItems") {
        Some(|value| value.get("SequenceNo")?.as_str().map(str::to_owned))
    } else if path.ends_with("Lines") {
        Some(|value| {
            let line_name = value.get("LineName")?.as_str()?;
            let direction = value.get("Direction").and_then(|v| v.as_str()).unwrap_or("");
            if direction.is_empty() {
                Some(line_name.to_owned())
            } else {
                Some(format!("{line_name}|{direction}"))
            }
        })
    } else if path.ends_with("RouteStops") {
        Some(|value| {
            let sequence = value.get("Sequence").and_then(|v| v.as_i64());
            let name = value.get("Name").and_then(|v| v.as_str());
            match (sequence, name) {
                (Some(sequence), Some(name)) => Some(format!("{sequence}|{name}")),
                (None, Some(name)) => Some(name.to_owned()),
                _ => None,
            }
        })
    } else {
        None
    }
}

pub fn flatten_changes(node: &DiffNode) -> Vec<DiffNode> {
    let mut out = Vec::new();
    collect_changes(node, &mut out);
    out
}

fn collect_changes(node: &DiffNode, out: &mut Vec<DiffNode>) {
    if node.status != DiffStatus::Unchanged {
        out.push(node.clone());
    }
    for child in &node.children {
        collect_changes(child, out);
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test diff::tests -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/diff.rs
git commit -m "feat: compare metadata arrays by business keys"
```

### Task 6: Add a pure app-state compare pipeline and summary counts

**Files:**
- Modify: `src/app.rs`
- Modify: `src/metadata.rs`
- Modify: `src/png_reader.rs`
- Modify: `src/diff.rs`

- [ ] **Step 1: Write the failing app pipeline tests**

Replace the tests in `src/app.rs` with:

```rust
#[cfg(test)]
mod tests {
    use super::PngMetadataCompareApp;
    use crate::diff::DiffStatus;
    use std::fs;

    fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
        bytes.extend_from_slice(kind);
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes
    }

    fn png_with_metadata(json: &str) -> Vec<u8> {
        let mut bytes = vec![137, 80, 78, 71, 13, 10, 26, 10];
        let mut data = Vec::new();
        data.extend_from_slice(b"StopPlateMetadata");
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.push(0);
        data.extend_from_slice(json.as_bytes());
        bytes.extend_from_slice(&chunk(b"iTXt", &data));
        bytes
    }

    #[test]
    fn compare_pipeline_builds_diff_and_counts() {
        let temp = std::env::temp_dir();
        let left = temp.join("png_metadata_compare_left.png");
        let right = temp.join("png_metadata_compare_right.png");
        fs::write(&left, png_with_metadata(r#"{"StopName":"A"}"#)).unwrap();
        fs::write(&right, png_with_metadata(r#"{"StopName":"B"}"#)).unwrap();

        let mut app = PngMetadataCompareApp::default();
        app.left_path = Some(left.display().to_string());
        app.right_path = Some(right.display().to_string());
        app.run_compare();

        let result = app.result.as_ref().unwrap();
        assert!(result.change_list.iter().any(|n| n.status == DiffStatus::Modified));
        assert_eq!(result.summary.modified, 2);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test compare_pipeline_builds_diff_and_counts -- --exact`

Expected: FAIL because `run_compare` and `result` do not exist

- [ ] **Step 3: Implement the compare pipeline and summary count model**

Update `src/png_reader.rs` to add file IO:

```rust
pub fn extract_stop_plate_metadata_from_file(path: &std::path::Path) -> Result<String, CompareError> {
    let bytes = std::fs::read(path).map_err(|err| CompareError::FileRead {
        path: path.to_path_buf(),
        reason: err.to_string(),
    })?;
    extract_stop_plate_metadata(&bytes)
}
```

Update `src/diff.rs` to add:

```rust
#[derive(Debug, Clone, Default)]
pub struct DiffSummary {
    pub modified: usize,
    pub added: usize,
    pub removed: usize,
    pub reordered: usize,
    pub error: usize,
}

impl DiffSummary {
    pub fn total(&self) -> usize {
        self.modified + self.added + self.removed + self.reordered + self.error
    }
}

pub fn summarize_changes(changes: &[DiffNode]) -> DiffSummary {
    let mut summary = DiffSummary::default();
    for change in changes {
        match change.status {
            DiffStatus::Modified => summary.modified += 1,
            DiffStatus::Added => summary.added += 1,
            DiffStatus::Removed => summary.removed += 1,
            DiffStatus::Reordered => summary.reordered += 1,
            DiffStatus::Error => summary.error += 1,
            DiffStatus::Unchanged => {}
        }
    }
    summary
}
```

Replace `src/app.rs` with:

```rust
use crate::diff::{compare_metadata, flatten_changes, summarize_changes, DiffNode, DiffSummary};
use crate::metadata::load_metadata;
use crate::png_reader::extract_stop_plate_metadata_from_file;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CompareResultView {
    pub root: DiffNode,
    pub change_list: Vec<DiffNode>,
    pub summary: DiffSummary,
    pub selected_path: Option<String>,
}

#[derive(Default)]
pub struct PngMetadataCompareApp {
    pub left_path: Option<String>,
    pub right_path: Option<String>,
    pub result: Option<CompareResultView>,
}

impl PngMetadataCompareApp {
    pub fn can_compare(&self) -> bool {
        self.left_path.is_some() && self.right_path.is_some()
    }

    pub fn run_compare(&mut self) {
        let left = self.left_path.as_ref().map(PathBuf::from).unwrap();
        let right = self.right_path.as_ref().map(PathBuf::from).unwrap();

        let left_metadata = load_metadata(extract_stop_plate_metadata_from_file(&left));
        let right_metadata = load_metadata(extract_stop_plate_metadata_from_file(&right));
        let root = compare_metadata(&left_metadata, &right_metadata);
        let change_list = flatten_changes(&root);
        let summary = summarize_changes(&change_list);

        self.result = Some(CompareResultView {
            root,
            change_list,
            summary,
            selected_path: None,
        });
    }
}

impl eframe::App for PngMetadataCompareApp {
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test compare_pipeline_builds_diff_and_counts -- --exact`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/app.rs src/png_reader.rs src/diff.rs
git commit -m "feat: wire file compare pipeline into app state"
```

### Task 7: Render the top bar, summary pane, detail pane, and empty states

**Files:**
- Modify: `src/app.rs`
- Modify: `src/ui/summary.rs`
- Modify: `src/ui/detail.rs`

- [ ] **Step 1: Write the failing summary rendering test**

Add this pure helper test to `src/ui/summary.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::summary_lines;
    use crate::diff::DiffSummary;

    #[test]
    fn formats_summary_lines_for_sidebar() {
        let summary = DiffSummary {
            modified: 3,
            added: 2,
            removed: 1,
            reordered: 4,
            error: 1,
        };

        assert_eq!(
            summary_lines(&summary),
            vec![
                "Total differences: 11".to_string(),
                "Modified: 3".to_string(),
                "Added: 2".to_string(),
                "Removed: 1".to_string(),
                "Reordered: 4".to_string(),
                "Errors: 1".to_string(),
            ]
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test formats_summary_lines_for_sidebar -- --exact`

Expected: FAIL because `summary_lines` is missing

- [ ] **Step 3: Implement sidebar/detail helpers and app layout**

Replace `src/ui/summary.rs` with:

```rust
use crate::app::CompareResultView;
use crate::diff::DiffSummary;

pub fn summary_lines(summary: &DiffSummary) -> Vec<String> {
    vec![
        format!("Total differences: {}", summary.total()),
        format!("Modified: {}", summary.modified),
        format!("Added: {}", summary.added),
        format!("Removed: {}", summary.removed),
        format!("Reordered: {}", summary.reordered),
        format!("Errors: {}", summary.error),
    ]
}

pub fn draw_summary(ui: &mut egui::Ui, result: &mut CompareResultView) {
    for line in summary_lines(&result.summary) {
        ui.label(line);
    }
    ui.separator();
    for item in &result.change_list {
        if ui.selectable_label(result.selected_path.as_deref() == Some(&item.path), &item.summary).clicked() {
            result.selected_path = Some(item.path.clone());
        }
    }
}
```

Replace `src/ui/detail.rs` with:

```rust
use crate::app::CompareResultView;
use crate::diff::DiffNode;

pub fn draw_detail(ui: &mut egui::Ui, result: &CompareResultView) {
    if let Some(selected_path) = &result.selected_path {
        if let Some(node) = find_node(&result.root, selected_path) {
            ui.heading("Detail");
            ui.label(format!("Path: {}", node.path));
            ui.label(format!("Status: {:?}", node.status));
            ui.label(format!("Summary: {}", node.summary));
            ui.separator();
            ui.label(format!("Left: {}", node.left_value.as_deref().unwrap_or("<none>")));
            ui.label(format!("Right: {}", node.right_value.as_deref().unwrap_or("<none>")));
            return;
        }
    }

    ui.heading("Detail");
    ui.label("Select a diff node to inspect values.");
}

fn find_node<'a>(node: &'a DiffNode, path: &str) -> Option<&'a DiffNode> {
    if node.path == path {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_node(child, path) {
            return Some(found);
        }
    }
    None
}
```

Replace the `update` method in `src/app.rs` with:

```rust
impl eframe::App for PngMetadataCompareApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Choose Left PNG").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("PNG", &["png"]).pick_file() {
                        self.left_path = Some(path.display().to_string());
                    }
                }
                ui.label(self.left_path.as_deref().unwrap_or("No file selected"));

                if ui.button("Choose Right PNG").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("PNG", &["png"]).pick_file() {
                        self.right_path = Some(path.display().to_string());
                    }
                }
                ui.label(self.right_path.as_deref().unwrap_or("No file selected"));

                if ui.add_enabled(self.can_compare(), egui::Button::new("Start Compare")).clicked() {
                    self.run_compare();
                }

                if ui.button("Swap Left/Right").clicked() {
                    std::mem::swap(&mut self.left_path, &mut self.right_path);
                }
            });
        });

        egui::SidePanel::left("summary").resizable(true).show(ctx, |ui| {
            ui.heading("Summary");
            if let Some(result) = &mut self.result {
                crate::ui::summary::draw_summary(ui, result);
            } else {
                ui.label("Choose two PNG files and click Start Compare.");
            }
        });

        egui::TopBottomPanel::bottom("detail").resizable(true).show(ctx, |ui| {
            if let Some(result) = &self.result {
                crate::ui::detail::draw_detail(ui, result);
            } else {
                ui.label("No comparison result yet.");
            }
        });
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test formats_summary_lines_for_sidebar -- --exact`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/app.rs src/ui/summary.rs src/ui/detail.rs
git commit -m "feat: render file chooser and result summary panes"
```

### Task 8: Render the diff tree, filters, and no-difference state

**Files:**
- Modify: `src/app.rs`
- Modify: `src/ui/tree.rs`

- [ ] **Step 1: Write the failing tree filter test**

Replace `src/ui/tree.rs` with:

```rust
use crate::diff::{DiffNode, DiffStatus};

#[derive(Debug, Clone, Copy)]
pub struct TreeFilters {
    pub only_differences: bool,
    pub show_reordered: bool,
    pub show_unchanged: bool,
    pub show_errors: bool,
}

#[cfg(test)]
mod tests {
    use super::{should_show, TreeFilters};
    use crate::diff::{DiffNode, DiffStatus};

    fn node(status: DiffStatus) -> DiffNode {
        DiffNode {
            path: "X".into(),
            status,
            left_value: None,
            right_value: None,
            summary: String::new(),
            children: Vec::new(),
        }
    }

    #[test]
    fn hides_unchanged_nodes_when_only_differences_is_enabled() {
        let filters = TreeFilters {
            only_differences: true,
            show_reordered: true,
            show_unchanged: false,
            show_errors: true,
        };

        assert!(!should_show(&node(DiffStatus::Unchanged), filters));
        assert!(should_show(&node(DiffStatus::Modified), filters));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test hides_unchanged_nodes_when_only_differences_is_enabled -- --exact`

Expected: FAIL because `should_show` is missing

- [ ] **Step 3: Implement tree rendering and filter state**

Replace `src/ui/tree.rs` with:

```rust
use crate::app::CompareResultView;
use crate::diff::{DiffNode, DiffStatus};

#[derive(Debug, Clone, Copy)]
pub struct TreeFilters {
    pub only_differences: bool,
    pub show_reordered: bool,
    pub show_unchanged: bool,
    pub show_errors: bool,
}

impl Default for TreeFilters {
    fn default() -> Self {
        Self {
            only_differences: true,
            show_reordered: true,
            show_unchanged: false,
            show_errors: true,
        }
    }
}

pub fn should_show(node: &DiffNode, filters: TreeFilters) -> bool {
    match node.status {
        DiffStatus::Unchanged => !filters.only_differences && filters.show_unchanged,
        DiffStatus::Reordered => filters.show_reordered,
        DiffStatus::Error => filters.show_errors,
        _ => true,
    }
}

pub fn draw_tree(ui: &mut egui::Ui, result: &mut CompareResultView, filters: TreeFilters) {
    draw_node(ui, &result.root, &mut result.selected_path, filters);
}

fn draw_node(
    ui: &mut egui::Ui,
    node: &DiffNode,
    selected_path: &mut Option<String>,
    filters: TreeFilters,
) {
    if !should_show(node, filters) {
        return;
    }

    let label = format!("{:?} {}", node.status, node.path);
    if node.children.is_empty() {
        if ui.selectable_label(selected_path.as_deref() == Some(&node.path), label).clicked() {
            *selected_path = Some(node.path.clone());
        }
        return;
    }

    egui::CollapsingHeader::new(label)
        .default_open(node.status != DiffStatus::Unchanged)
        .show(ui, |ui| {
            if ui.selectable_label(selected_path.as_deref() == Some(&node.path), &node.summary).clicked() {
                *selected_path = Some(node.path.clone());
            }
            for child in &node.children {
                draw_node(ui, child, selected_path, filters);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::{should_show, TreeFilters};
    use crate::diff::{DiffNode, DiffStatus};

    fn node(status: DiffStatus) -> DiffNode {
        DiffNode {
            path: "X".into(),
            status,
            left_value: None,
            right_value: None,
            summary: String::new(),
            children: Vec::new(),
        }
    }

    #[test]
    fn hides_unchanged_nodes_when_only_differences_is_enabled() {
        let filters = TreeFilters {
            only_differences: true,
            show_reordered: true,
            show_unchanged: false,
            show_errors: true,
        };

        assert!(!should_show(&node(DiffStatus::Unchanged), filters));
        assert!(should_show(&node(DiffStatus::Modified), filters));
    }
}
```

Update `src/app.rs`:

```rust
use crate::ui::tree::TreeFilters;
```

Add a field to `PngMetadataCompareApp`:

```rust
pub filters: TreeFilters,
```

Update the `Default` derive to a manual impl:

```rust
impl Default for PngMetadataCompareApp {
    fn default() -> Self {
        Self {
            left_path: None,
            right_path: None,
            result: None,
            filters: TreeFilters::default(),
        }
    }
}
```

Add the central panel in `update`:

```rust
egui::CentralPanel::default().show(ctx, |ui| {
    ui.horizontal(|ui| {
        ui.checkbox(&mut self.filters.only_differences, "Only differences");
        ui.checkbox(&mut self.filters.show_reordered, "Show reordered");
        ui.checkbox(&mut self.filters.show_unchanged, "Show unchanged");
        ui.checkbox(&mut self.filters.show_errors, "Show errors");
    });
    ui.separator();

    if let Some(result) = &mut self.result {
        if result.summary.total() == 0 {
            ui.heading("No differences found");
        } else {
            crate::ui::tree::draw_tree(ui, result, self.filters);
        }
    } else {
        ui.heading("Structured diff tree will appear here.");
    }
});
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test hides_unchanged_nodes_when_only_differences_is_enabled -- --exact`

Expected: PASS

- [ ] **Step 5: Commit**

Run:

```bash
git add src/app.rs src/ui/tree.rs
git commit -m "feat: render structured diff tree with filters"
```

### Task 9: Cover parse-failure comparisons and final regression pass

**Files:**
- Modify: `src/diff.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Write the failing regression tests for parse-failure display**

Add these tests:

In `src/diff.rs`:

```rust
#[test]
fn keeps_error_node_visible_in_flattened_results() {
    let left = MetadataLoadResult::Error(crate::error::CompareError::MissingStopPlateMetadata);
    let right = MetadataLoadResult::Parsed(serde_json::json!({"StopName": "A"}));

    let diff = compare_metadata(&left, &right);
    let changes = flatten_changes(&diff);
    assert!(changes.iter().any(|n| n.status == DiffStatus::Error));
}
```

In `src/app.rs`:

```rust
#[test]
fn compare_pipeline_surfaces_missing_metadata_as_error_result() {
    let temp = std::env::temp_dir();
    let left = temp.join("png_metadata_compare_missing_left.png");
    let right = temp.join("png_metadata_compare_missing_right.png");
    std::fs::write(&left, vec![137, 80, 78, 71, 13, 10, 26, 10]).unwrap();
    std::fs::write(
        &right,
        vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 0, 73, 69, 78, 68, 0, 0, 0, 0],
    ).unwrap();

    let mut app = PngMetadataCompareApp::default();
    app.left_path = Some(left.display().to_string());
    app.right_path = Some(right.display().to_string());
    app.run_compare();

    let result = app.result.as_ref().unwrap();
    assert!(result.change_list.iter().any(|n| n.status == DiffStatus::Error));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test keeps_error_node_visible_in_flattened_results compare_pipeline_surfaces_missing_metadata_as_error_result -- --exact`

Expected: FAIL if error nodes are not yet preserved or app result handling is brittle

- [ ] **Step 3: Make the error result path explicit and run the full suite**

If the tests fail because the root `Error` node is the only visible node, adjust `flatten_changes` to keep root-level errors:

```rust
fn collect_changes(node: &DiffNode, out: &mut Vec<DiffNode>) {
    if node.status != DiffStatus::Unchanged {
        out.push(node.clone());
    }
    for child in &node.children {
        collect_changes(child, out);
    }
}
```

If the tests fail because `run_compare` panics on missing paths, harden the path guard:

```rust
pub fn run_compare(&mut self) {
    let (Some(left), Some(right)) = (self.left_path.as_ref(), self.right_path.as_ref()) else {
        return;
    };

    let left_metadata = load_metadata(extract_stop_plate_metadata_from_file(&PathBuf::from(left)));
    let right_metadata = load_metadata(extract_stop_plate_metadata_from_file(&PathBuf::from(right)));
    let root = compare_metadata(&left_metadata, &right_metadata);
    let change_list = flatten_changes(&root);
    let summary = summarize_changes(&change_list);

    self.result = Some(CompareResultView {
        root,
        change_list,
        summary,
        selected_path: None,
    });
}
```

Then run the full suite:

Run: `cargo test`

Expected: PASS

- [ ] **Step 4: Run the desktop app manually**

Run: `cargo run`

Expected:

- the native window opens on Windows
- both file chooser buttons work
- `Start Compare` is disabled until both files are chosen
- comparing valid metadata shows a structured tree and summary list
- comparing a PNG without `StopPlateMetadata` shows an error result inside the same UI
- identical metadata shows `No differences found`

- [ ] **Step 5: Commit**

Run:

```bash
git add src/app.rs src/diff.rs
git commit -m "test: verify error results and final compare flow"
```
