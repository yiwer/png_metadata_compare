# PNG Directory Batch Compare Design

## Overview

Extend the existing Windows-native GUI tool so it can compare two directories instead of only two individual PNG files.

The directory mode scans both directories recursively for PNG files, pairs files by file name, resolves duplicate file-name collisions by using the immediate parent directory name, and then compares each matched pair by the existing `StopPlateMetadata` extraction and diff pipeline.

Results are shown inside the GUI as a batch report with four categories:

- `Identical`
- `Different`
- `Left Only`
- `Right Only`

Users can click any item in the report to inspect details. For matched pairs with metadata differences, the existing structured diff UI is reused.

## Goals

- Add a directory comparison mode without replacing the existing single-file mode.
- Recursively scan both selected directories for PNG files.
- Pair files primarily by file name.
- When file-name duplicates exist, refine pairing by immediate parent directory name.
- Report unmatched files as left-only or right-only items.
- Report matched files as either identical or different.
- Let users click a report item to inspect details inside the GUI.
- Reuse the current metadata extraction and diff engine for matched pairs.

## Non-Goals

- Exporting the batch report to CSV, JSON, Markdown, or other files.
- Ignoring selected metadata fields in this iteration.
- Adding deeper disambiguation beyond file name and immediate parent directory name.
- Pixel-level image comparison.
- Cross-platform packaging work.
- Performance-oriented parallel execution or caching.

## Chosen Approach

Recommended approach:

- Keep one desktop application and add a `Single File` / `Directory` mode switch.
- Build a new batch scanning and pairing layer above the existing metadata compare pipeline.
- Reuse the current per-pair diff result view for `Different` items.

Why this approach:

- It avoids duplicating the current successful single-pair compare architecture.
- It keeps the user workflow consistent.
- It allows batch items with differences to drill directly into the existing tree diff and detail panels.

Alternatives considered:

1. Separate directory-compare screen or window
   - Cleaner isolation, but duplicates too much existing result-view logic.
2. Directory scan only, then force manual handoff into single-file compare
   - Faster to build, but fails the requirement for an internal categorized report page with clickable items.

## Pairing Rules

The pairing key is defined in two stages.

### Stage 1: File Name

Files are first grouped by file name only.

- If a file name appears exactly once on the left and exactly once on the right, pair those two files directly.

### Stage 2: Immediate Parent Directory Name

If a file name appears multiple times on one or both sides, refine pairing inside that file-name group by immediate parent directory name.

- If this produces a unique pair, accept it.
- If it still does not produce a unique match, do not guess.

### Unresolved Duplicate Groups

If file name plus immediate parent directory name still cannot produce a unique pairing:

- do not hard-match by discovery order
- classify the unresolved files as unmatched
- record the reason explicitly in the report detail

Examples of unmatched reasons:

- no same-name file exists on the other side
- duplicate file name could not be uniquely resolved
- duplicate file name group exists on both sides, but parent-directory matching is still ambiguous

## Batch Result Categories

The batch report contains exactly four top-level categories.

### `Identical`

Matched pairs whose metadata comparison yields no differences.

### `Different`

Matched pairs whose metadata comparison yields at least one difference.

Important:

- Any metadata field difference counts.
- Metadata parse or extraction failure on either side also counts as `Different`.
- There is no separate `Misaligned` or `Wrong Match` category in this design.

### `Left Only`

Files from the left directory that could not be uniquely paired.

### `Right Only`

Files from the right directory that could not be uniquely paired.

## Comparison Rules for Matched Pairs

Matched pairs are compared with the existing metadata pipeline:

1. extract `StopPlateMetadata`
2. parse JSON into the current metadata model
3. compare with the existing structured diff engine
4. flatten and summarize the diff

Outcome mapping:

- zero diff entries => `Identical`
- one or more diff entries => `Different`

This includes parser and extractor errors, because those are already represented as explicit diff results.

## GUI Design

The application gains a mode switch:

- `Single File`
- `Directory`

### Directory Mode Top Bar

Show:

- left directory chooser
- right directory chooser
- `Start Scan and Compare`
- `Swap Left/Right`
- mode toggle

Rules:

- comparison cannot start until both directories are selected
- switching directories clears the previous batch result

### Batch Report Navigation

The left report pane shows four fixed sections:

- `Identical`
- `Different`
- `Left Only`
- `Right Only`

Each section displays a count.

Example:

- `Different (18)`

Each section expands to show its items.

### Report Item Labels

Each item should show:

- file name
- supporting location context, such as immediate parent directory name or relative path

For `Different`:

- include a compact summary such as `7 differences`

For `Left Only` / `Right Only`:

- include the unmatched reason

### Right Detail Pane Behavior

When the user selects an item:

- `Identical`
  - show left/right paths
  - show pairing strategy
  - show `Metadata identical`
  - optionally show a compact metadata identity summary

- `Different`
  - show left/right paths
  - show diff summary
  - reuse the existing change list, tree diff, and detail panels

- `Left Only` / `Right Only`
  - show file path
  - show file name
  - show immediate parent directory name
  - show unmatched reason
  - if metadata can still be extracted from that isolated file, showing a compact summary is allowed but not required

### Empty States

- Before running directory compare: prompt the user to select two directories.
- If all matched pairs are identical and there are no unmatched files:
  - report still renders, with all items inside `Identical`
- If no PNG files are found on one or both sides:
  - render an explicit empty batch result, not a crash or silent no-op

## New Modules

### `src/batch_scan.rs`

Responsibilities:

- recursively scan a directory for PNG files
- build per-file records
- group files by file name
- resolve unique pairs by file name, then by immediate parent directory name
- produce unmatched records with explicit reasons

### `src/batch_report.rs`

Responsibilities:

- define batch report models
- represent category membership and detail payloads
- carry matched-pair compare outcomes into the GUI

## Batch Data Model

Suggested per-file record:

- `absolute_path`
- `relative_path`
- `file_name`
- `parent_dir_name`

Suggested matched result:

- `file_name`
- `left_path`
- `right_path`
- `match_strategy`
  - `file_name`
  - `file_name + parent_dir_name`
- `compare_outcome`
  - `identical`
  - `different`
- `diff_summary`
- `diff_root`

Suggested unmatched result:

- `side`
- `file_name`
- `file_path`
- `parent_dir_name`
- `reason`

## Error Handling

Directory mode must distinguish between:

- scan-level issues
  - directory missing
  - directory unreadable
- pairing issues
  - unresolved duplicate group
  - no opposite-side match
- compare-level issues
  - metadata extraction failure
  - metadata parse failure
  - diff engine error nodes

Scan-level failures should be rendered as batch-mode result errors, not fatal crashes.

Pairing failures go to `Left Only` or `Right Only` with clear reasons.

Compare-level failures for matched pairs go to `Different`, because they are a meaningful comparison result rather than an absence of pairing.

## Future Extension Point

The design must leave room for field-ignore support later.

Not in this iteration:

- no GUI to choose ignored fields
- no stored ignore profile

But the batch result model should not hard-code the assumption that every metadata field is always compared forever.

## Testing Strategy

### Directory Scan Tests

- recursively finds PNG files in nested folders
- ignores non-PNG files
- handles empty directories

### Pairing Tests

- unique file-name match
- duplicate file-name group resolved by immediate parent directory name
- duplicate file-name group still ambiguous after parent-directory refinement
- left-only classification
- right-only classification

### Batch Compare Tests

- matched identical pair
- matched pair with at least one metadata difference
- matched pair where one side has metadata extraction or parse error, still classified as `Different`

### UI State Tests

- category counts are correct
- selecting a `Different` item opens the existing detailed diff state
- selecting a `Left Only` or `Right Only` item shows its unmatched reason

## Acceptance Criteria

- Users can switch between single-file mode and directory mode.
- Directory mode recursively scans both selected directories for PNG files.
- Matching uses file name first, then immediate parent directory name when duplicates exist.
- Unresolved matches are not guessed.
- The report shows exactly four categories:
  - `Identical`
  - `Different`
  - `Left Only`
  - `Right Only`
- Any metadata field difference causes a matched pair to appear under `Different`.
- Matched pairs with extractor or parser errors also appear under `Different`.
- Users can click report items to inspect details in the GUI.
- Existing single-file compare behavior remains available.
