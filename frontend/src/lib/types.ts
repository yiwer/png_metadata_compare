export type JsonValue =
  | string
  | number
  | boolean
  | null
  | { [key: string]: JsonValue }
  | JsonValue[];

export type Side = 'left' | 'right';

export type DiffStatus =
  | 'unchanged'
  | 'modified'
  | 'added'
  | 'removed'
  | 'reordered'
  | 'error';

export interface UiError {
  code: string;
  message: string;
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

export interface SideInspection {
  side: Side;
  file_path: string;
  file_name: string;
  raw_json: string | null;
  metadata: JsonValue | null;
  error: UiError | null;
}

export interface PairInspection {
  left: SideInspection;
  right: SideInspection;
  diff_root: DiffNode;
  diff_summary: DiffSummary;
  default_selected_path: string | null;
}

export type MatchStrategy = 'file_name' | 'file_name_and_parent_dir';

export type BatchListItemKind =
  | 'identical'
  | 'different'
  | 'left_only'
  | 'right_only'
  | 'error';

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
  match_strategy: MatchStrategy | null;
  message: string | null;
}

export interface DirectorySummary {
  counts: BatchCounts;
  items: BatchListItem[];
}

export type WorkbenchMode = 'single' | 'directory';

export type AnalysisTab =
  | 'diff'
  | 'left-metadata'
  | 'right-metadata'
  | 'raw-json'
  | 'images';
