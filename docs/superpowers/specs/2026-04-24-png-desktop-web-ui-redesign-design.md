# PNG Desktop Web UI Redesign Design

Date: 2026-04-24
Status: Draft for review

## Goal

Replace the current `eframe` native GUI with a desktop Web UI while preserving the existing Rust comparison logic for:

- single PNG metadata compare
- directory batch compare

The redesigned application must:

- launch on Windows without a command-line console window
- adopt a MotherDuck-inspired brutalist-minimal visual language
- support side-by-side image preview for the active compare target
- let users inspect diff results and full metadata, not only the comparison summary

## Non-Goals

- changing PNG metadata extraction semantics unless required for UI data transport
- adding metadata editing or write-back support
- adding cloud sync, multi-user features, or remote compare
- implementing advanced image annotation in the first release

## Product Direction

The application becomes a desktop inspection workbench rather than a simple diff panel.

The primary user flow is:

1. Select two files or two directories.
2. Run compare.
3. Select an active file pair or result item.
4. Review both PNG previews.
5. Switch between diff, left metadata, right metadata, raw JSON, and image-focused views.
6. Inspect exact field-level details in the inspector panel.

This structure supports both verification and debugging workflows:

- "Are these two PNGs visually the same?"
- "Which metadata fields differ?"
- "What is the exact raw metadata on each side?"
- "Why did this item fail to compare?"

## Recommended Stack

### Desktop Shell

- `Tauri 2` for the desktop host
- Windows build configured with `windows_subsystem = "windows"` so the app opens without a console window

### Core Logic

- preserve the Rust compare logic already present in the repository
- extract UI-independent service functions from the current `app.rs` orchestration path
- expose stable Tauri commands for compare operations and file metadata inspection

### Frontend

- `React + Vite + TypeScript`
- CSS driven by explicit design tokens; no dependency on a native widget toolkit

This choice minimizes logic churn in the Rust core while giving the UI enough control to match the requested design and interaction model.

## Architecture

The application is split into three layers.

### 1. Core Rust Domain Layer

Responsibilities:

- load PNG files
- extract StopPlate metadata
- compare left/right metadata trees
- summarize changes
- scan directories and build batch reports
- surface structured error states

Constraints:

- no frontend-specific presentation concerns
- all results serialized into stable DTOs for Tauri commands

### 2. Desktop Bridge Layer

Responsibilities:

- window lifecycle
- open file and folder dialogs
- call Rust domain services
- convert domain errors into frontend-safe payloads
- package the app for desktop distribution

Bridge API categories:

- app bootstrap and version info
- choose files / choose folders
- run single compare
- run directory compare
- load image preview payload
- load full metadata payload for one side

### 3. Web UI Layer

Responsibilities:

- render the workbench
- maintain active mode, active result selection, active tab, and filter state
- show loading, empty, and error states
- display previews, diff tree, metadata tree, raw JSON, and inspector content

## Information Architecture

The application uses a desktop workbench layout optimized for repeated inspection.

### Top Toolbar

Contains:

- mode switch: `Single File` / `Directory`
- left and right input selectors
- `Compare`
- `Swap`
- active filters such as `Only Differences`

Rules:

- each mode keeps its own most recent input paths
- switching modes clears current rendered results but preserves last-used inputs

### Left Summary / Result Rail

In single-file mode:

- summary counts
- quick status indicators

In directory mode:

- batch result list
- grouped or filterable item types:
  - different
  - identical
  - left only
  - right only
  - error

This rail is the navigation source for the current active compare target.

### Main Workspace

The main workspace is vertically split into two regions.

#### Persistent Image Preview Region

Always visible for the active target.

Shows:

- left PNG preview
- right PNG preview
- file name
- basic image information such as dimensions when available
- affordances to open the full image-focused view

This supports visual confirmation without forcing the user to leave the compare workflow.

#### Tabbed Analysis Region

Tabs:

- `Diff`
- `Left Metadata`
- `Right Metadata`
- `Raw JSON`
- `Images`

Purpose of each tab:

- `Diff`: compare tree, change counts, field-level differences
- `Left Metadata`: complete left-side structured metadata browser
- `Right Metadata`: complete right-side structured metadata browser
- `Raw JSON`: raw extracted metadata text for both sides, copyable
- `Images`: larger side-by-side image review area

### Inspector Panel

The inspector is synchronized with the current tab and current selected node.

In `Diff`:

- selected path
- left value
- right value
- node status
- local JSON snippet when useful

In metadata tabs:

- selected field path
- full value
- type or shape hints if useful

In error cases:

- error type
- affected side
- source path
- raw error message

## Interaction Model

The UI follows a single-source-of-truth selection model.

### Primary State

- `mode`
- `leftInput`
- `rightInput`
- `activeResult`
- `activeTab`
- `activeNodePath`
- `filters`
- `loadingState`

### Selection Rules

- clicking a result in the left rail sets `activeResult`
- `activeResult` updates image previews, tab content, and inspector together
- tabs never change the underlying data source; they only change the view of the same active result
- clicking a node in a tree or metadata browser sets `activeNodePath`
- the inspector always reflects the active result plus active node

### Preview Behavior

- previews remain visible while switching tabs
- clicking a preview opens or focuses the `Images` tab
- first release supports view and open behavior; zoom and pan are optional enhancements

### Batch Result Behavior

- `different` defaults to the `Diff` tab
- `identical` can default to `Left Metadata` or `Images`
- `left only` / `right only` open the available side metadata and a missing-side explanation
- `error` opens an error-focused inspector and raw payload view if available

## Data Contracts

The frontend should not reconstruct domain logic from partial payloads. The Tauri layer should return explicit DTOs.

### Single Compare DTO

Should include:

- left and right file references
- image preview references or loadable paths
- diff tree
- diff summary
- left metadata tree
- right metadata tree
- raw JSON text by side
- extracted issues by side
- default selected node path

### Directory Compare DTO

Should include:

- overall counts
- issue list
- result items with explicit type
- for `different` items, the same inspection payload shape as single compare
- for non-different items, enough payload to drive image and metadata views without recomputing on the client

### Error DTO

Should normalize:

- `FileMissing`
- `ReadFailure`
- `PngParseFailure`
- `MetadataMissing`
- `JsonParseFailure`
- `DirectoryScanFailure`

Each error should include side, path, user-facing summary, and raw detail text.

## Visual Design Direction

The UI should be inspired by the provided MotherDuck guides rather than copied literally.

### Visual Principles

- bold borders instead of shadows
- near-zero border radius
- high-contrast panels
- warm beige workspace background
- strong yellow and blue accents
- uppercase labels and controls where appropriate
- restrained motion with functional meaning

### Token Direction

Primary colors:

- beige background: `#F4EFEA`
- off-white surfaces: `#F8F8F7`
- white cards: `#FFFFFF`
- dark text / border: `#383838`
- primary yellow: `#FFDE00`
- primary blue: `#6FC2FF`
- error accent: `#FF7169`
- success accent: `#22C55E`

Font direction:

- use distributable substitutes that preserve the intended feel
- mono-forward UI for titles, data, and controls
- avoid default system-stack styling

### Motion

Use short transitions only when they communicate state:

- button hover
- active selection shift
- tab content change
- result-row selection
- loading or reveal transitions

Avoid decorative animation loops in core work areas.

## Empty, Loading, and Error States

### Empty States

- before compare, show clear guidance instead of blank panels
- if only one side is selected, allow inspection of that side once data is available, while compare remains disabled

### Loading States

Show inline progress states for:

- scanning directories
- reading PNGs
- extracting metadata
- building compare payloads

No blocking system-native modal should be required for normal operations.

### Error States

Errors are first-class result types, not hidden failures.

Requirements:

- show error items in the result rail
- keep partial data visible where possible
- avoid collapsing the workspace to empty content
- favor inline error cards and banner notices over disruptive popups

## Testing Strategy

### Rust Core

Preserve and extend tests for:

- single compare
- directory compare
- identical pairs
- different pairs
- left-only / right-only cases
- missing metadata
- invalid JSON
- damaged or unreadable PNGs
- directory scan failures

### Tauri Bridge

Add command-level integration tests for:

- returned DTO shape
- error normalization
- single compare payload completeness
- directory compare payload completeness

### Frontend

Add targeted UI tests for:

- mode switching clears stale rendered state
- active result selection updates all synchronized panels
- tab switching preserves active result
- diff node selection updates inspector
- non-different result types remain browsable
- error items render actionable content

Full end-to-end browser automation is optional for the first pass, but critical-path verification is required before shipping.

## Migration Plan

Implementation should proceed in phases.

### Phase 1

- introduce Tauri app shell
- move current Rust UI orchestration into reusable services
- reproduce existing compare features in a minimal Web UI

### Phase 2

- implement the MotherDuck-inspired workbench layout
- add persistent image preview
- add tabbed analysis region
- add inspector synchronization

### Phase 3

- refine loading, error, and empty states
- polish motion and keyboard flow
- package Windows build with no console window

## Risks and Mitigations

### Risk: UI migration changes compare semantics

Mitigation:

- keep comparison logic in Rust
- define explicit DTOs
- preserve existing tests and extend them before UI cutover

### Risk: directory mode payloads become too heavy

Mitigation:

- return summary-first list payloads
- lazily load full inspection payload for the selected result when needed if performance requires it

### Risk: image preview loading adds latency

Mitigation:

- load previews only for the active result
- cache current and recently viewed previews in memory on the frontend where useful

### Risk: design system drift during implementation

Mitigation:

- centralize color, spacing, border, and motion tokens
- build shared UI primitives for result rows, tabs, panels, and inspector cards

## Acceptance Criteria

The redesign is successful when:

- the app launches on Windows without a console window
- single-file compare still works end to end
- directory batch compare still works end to end
- users can inspect both PNG previews for the active result
- users can inspect diff, left metadata, right metadata, and raw JSON
- error and missing-side cases are inspectable rather than opaque
- the interface reflects the requested brutalist MotherDuck-inspired direction
