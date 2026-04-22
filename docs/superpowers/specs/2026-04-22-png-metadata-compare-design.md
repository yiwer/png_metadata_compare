# PNG Metadata Compare Design

## Overview

Build a Windows-native GUI tool in Rust that accepts two PNG files, extracts the `StopPlateMetadata` JSON stored in the PNG `iTXt` chunk, compares the two metadata payloads, and presents the differences in a readable, structured way.

The tool is not responsible for pixel-level image comparison. Its scope is limited to the embedded `StopPlateMetadata` metadata described in [`tmp/stop_plate_json_meta.md`](G:\Rust\png_metadata_compare\tmp\stop_plate_json_meta.md).

## Goals

- Provide a Windows desktop GUI application.
- Let the user choose two local PNG files and compare them.
- Extract `StopPlateMetadata` from PNG `iTXt` chunks without external tools.
- Show differences in a structured tree view that is easy to read.
- Also provide a flattened change list as a summary.
- Treat parse failures as displayable comparison results instead of hard-stop fatal errors.

## Non-Goals

- Comparing PNG pixel data or image rendering output.
- Comparing other PNG text chunks such as unrelated `tEXt`, `zTXt`, or `iTXt` records.
- Batch comparison, folder scanning, or drag-and-drop in the first version.
- Exporting reports in the first version.
- Cross-platform support in the first version.

## Chosen Approach

Recommended implementation stack:

- Language: Rust
- GUI framework: `egui/eframe`
- Target platform: Windows desktop only

Why this approach:

- It keeps the project single-language and aligned with the current Rust repository.
- It is fast to build for a tool-oriented interface with split panels, tree views, and detail panes.
- It avoids the complexity of a mixed Rust + C#/WPF architecture.

Alternatives considered:

1. `Rust + Slint`
   - Better for polished declarative UI, but less attractive for a custom diff-heavy tree explorer in a greenfield tool.
2. `Rust core + C#/WPF frontend`
   - Best native Windows look and mature tree controls, but adds a second language and a more complex integration boundary.

## Architecture

The application is divided into four modules.

### `png_reader`

Responsibilities:

- Read PNG bytes from disk.
- Validate the PNG signature at a basic level.
- Iterate through PNG chunks.
- Locate `iTXt` chunks whose keyword is `StopPlateMetadata`.
- Decode and return the JSON text payload, or a specific extraction error.

### `metadata_model`

Responsibilities:

- Parse extracted JSON into an internal representation.
- Preserve compatibility with future schema changes.
- Support field-level traversal for recursive diffing.

Design choice:

- Use a generic JSON tree representation as the canonical comparison model.
- Layer business-key rules for known arrays on top of that generic representation.

This avoids a brittle parser that would need immediate code changes whenever the upstream metadata adds fields.

### `diff_engine`

Responsibilities:

- Compare two metadata trees.
- Output a structured diff tree for the GUI.
- Output a flattened change list derived from the diff tree.
- Surface parser and extraction failures as diffable results.

### `ui`

Responsibilities:

- Let the user choose the left and right PNG files.
- Trigger comparison.
- Render the diff tree, summary list, and detail panel.
- Show parsing and extraction errors in normal result views instead of modal failures.

## Data Flow

1. User selects two PNG files.
2. The application reads each file and attempts to extract `StopPlateMetadata`.
3. Each extracted payload is parsed into the internal metadata model, or recorded as an error result.
4. The diff engine compares the two sides.
5. The UI renders:
   - status summary
   - change list
   - diff tree
   - selected-node detail view

## Diff Model

Each diff node contains:

- `path`: stable human-readable path such as `Lines[B932|Terminal].RouteStops[8|CurrentStop].BuildingType`
- `status`: one of `unchanged`, `modified`, `added`, `removed`, `reordered`, `error`
- `left_value`
- `right_value`
- `summary`: a concise human-readable description
- `children`

The tree is the canonical result. The change list is a flattened view of non-`unchanged` nodes.

## Comparison Rules

### Scalar values

- Compare directly.
- If values differ, mark the node as `modified`.

### Objects

- Compare by field name recursively.
- Missing keys become `added` or `removed`.

### Arrays

- Compare by business key first, not raw index.
- If the same business object exists on both sides, compare its fields recursively.
- If an item exists on only one side, mark it as `added` or `removed`.
- If the same business object exists on both sides but appears at a different position, emit `reordered`.

### Errors

Extraction or parsing problems are represented as `error` diff nodes so the UI can render them as normal comparison output.

Examples:

- PNG is unreadable
- PNG signature is invalid
- `StopPlateMetadata` is missing
- `iTXt` chunk structure is invalid
- metadata text is not valid UTF-8
- metadata JSON is invalid
- business-key matching is ambiguous

### Unknown fields

- Unknown or newly added JSON fields are still compared recursively.
- The tool must not require immediate code changes to expose newly introduced metadata fields.

## Business-Key Matching Rules

Known array matching rules:

- `GroupItems`: match by `SequenceNo`
- `Lines`: match by `LineName + Direction`; if `Direction` is empty or missing, fall back to `LineName`
- `RouteStops`: match by `Sequence + Name`; if the data is incomplete, fall back to `Name`

If the key is duplicated on one side and unique matching is impossible:

- generate an `error` node describing the ambiguity
- fall back to a conservative comparison outcome instead of silently guessing

## GUI Design

Main window layout:

### Top action bar

- Left PNG file chooser and path display
- Right PNG file chooser and path display
- `Start Compare` button
- `Swap Left/Right` button

Rules:

- Comparison cannot start until both files are selected.

### Left summary pane

Shows:

- parse status for both sides
- total number of differences
- counts for `modified`, `added`, `removed`, `reordered`, `error`
- clickable change list

Interaction:

- Selecting a change list item focuses the corresponding diff-tree node.

### Right main pane

Shows:

- structured diff tree rooted at `StopPlateMetadata`
- changed nodes highlighted
- unchanged nodes collapsed by default

Behavior:

- auto-expand paths that contain differences
- allow filters:
  - only show differences
  - show or hide reordered items
  - show or hide unchanged items
  - show or hide error nodes

### Bottom detail pane

Shows details for the currently selected node:

- full path
- diff status
- left value
- right value
- human-readable summary
- explicit error message when applicable

### Empty and success states

- Before comparison: prompt the user to choose two PNG files.
- When metadata is identical: show a clear `No differences found` result.
- When parsing fails on one or both sides: still show results in the same UI, with explicit failure details.

## Error Handling

Errors are grouped by layer.

### File layer

- file missing
- file unreadable

### PNG layer

- invalid PNG signature
- truncated or malformed chunk layout

### Metadata extraction layer

- `StopPlateMetadata` not found
- malformed `iTXt` structure
- unsupported compressed metadata if encountered

### JSON layer

- UTF-8 decode failure
- invalid JSON

### Diff layer

- ambiguous business-key match
- unsupported edge-case structure handled conservatively

All of these are rendered into the comparison result model instead of terminating the session with a blocking modal error.

## First-Version Scope Boundary

The first implementation includes:

- manual selection of two local PNG files
- extraction of `StopPlateMetadata`
- structured tree diff
- flattened change list
- detail pane
- explicit error rendering

The first implementation excludes:

- report export
- drag-and-drop
- batch or directory workflows
- image-content comparison
- non-Windows packaging work

## Suggested Source Layout

```text
src/
  main.rs
  app.rs
  error.rs
  png_reader.rs
  metadata.rs
  diff.rs
  ui/
    summary.rs
    tree.rs
    detail.rs
```

## Testing Strategy

### PNG extraction tests

- valid PNG with `StopPlateMetadata`
- PNG with no `StopPlateMetadata`
- malformed chunk structure
- invalid PNG signature

### Metadata parsing tests

- valid sample JSON
- fields with `null`
- missing optional fields
- unknown extra fields
- invalid UTF-8 or invalid JSON

### Diff engine tests

- scalar field modification
- object field add/remove
- array business-key matching
- reorder-only change
- business-item add/remove
- parser error versus valid metadata
- ambiguous business-key conflict

### UI-level verification

- selecting two files enables comparison
- completed comparison produces summary and tree state
- parse failure still produces a visible result state

Automated UI interaction coverage can remain minimal in the first version. Most verification value is in extraction, parsing, and diff-engine tests.

## Acceptance Criteria

- The app runs as a Windows native GUI application.
- A user can select two PNG files and compare them.
- The app reads `StopPlateMetadata` directly from PNG `iTXt` chunks.
- Differences are shown in a structured tree view.
- A change list summarizes the non-unchanged nodes.
- Arrays are matched by business key instead of only by index.
- Reordering is represented explicitly as order change, not disguised as unrelated field edits.
- Parse and extraction failures appear as readable comparison results.
- Unknown future metadata fields still appear in the diff output.

## Open Decisions Resolved In This Spec

- GUI type: native GUI application
- Primary diff presentation: structured tree view
- Secondary diff presentation: flattened change list
- Array matching: by business key, with reorder awareness
- Failed parsing: display as comparison result
- Metadata scope: only `StopPlateMetadata`
- Platform target: Windows desktop only
