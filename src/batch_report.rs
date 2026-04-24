use crate::batch_scan::BatchFileRecord;
use crate::diff::{DiffNode, DiffSummary};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchStrategy {
    FileName,
    FileNameAndParentDir,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchedPair {
    pub file_name: String,
    pub left: BatchFileRecord,
    pub right: BatchFileRecord,
    pub match_strategy: MatchStrategy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnmatchedSide {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnmatchedFile {
    pub side: UnmatchedSide,
    pub file: BatchFileRecord,
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PairingResult {
    pub matched: Vec<MatchedPair>,
    pub left_only: Vec<UnmatchedFile>,
    pub right_only: Vec<UnmatchedFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdenticalPairResult {
    pub pair: MatchedPair,
}

#[derive(Clone, Debug)]
pub struct DifferentPairResult {
    pub pair: MatchedPair,
    pub diff_root: DiffNode,
    pub change_list: Vec<DiffNode>,
    pub summary: DiffSummary,
    pub selected_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BatchIssue {
    ScanFailure {
        side: UnmatchedSide,
        path: PathBuf,
        reason: String,
    },
}

#[derive(Clone, Debug, Default)]
pub struct BatchCompareReport {
    pub issues: Vec<BatchIssue>,
    pub identical: Vec<IdenticalPairResult>,
    pub different: Vec<DifferentPairResult>,
    pub left_only: Vec<UnmatchedFile>,
    pub right_only: Vec<UnmatchedFile>,
}

#[derive(Clone, Debug)]
pub enum MatchedPairCompareResult {
    Identical {
        pair: MatchedPair,
    },
    Different {
        pair: MatchedPair,
        diff_root: DiffNode,
        change_list: Vec<DiffNode>,
        summary: DiffSummary,
    },
}

impl MatchedPairCompareResult {
    pub fn identical(pair: MatchedPair) -> Self {
        Self::Identical { pair }
    }

    pub fn different(
        pair: MatchedPair,
        diff_root: DiffNode,
        change_list: Vec<DiffNode>,
        summary: DiffSummary,
    ) -> Self {
        Self::Different {
            pair,
            diff_root,
            change_list,
            summary,
        }
    }
}

pub fn build_batch_results(
    matched: Vec<MatchedPairCompareResult>,
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

    for matched_result in matched {
        match matched_result {
            MatchedPairCompareResult::Identical { pair } => {
                report.identical.push(IdenticalPairResult { pair });
            }
            MatchedPairCompareResult::Different {
                pair,
                diff_root,
                change_list,
                summary,
            } => {
                report.different.push(DifferentPairResult {
                    pair,
                    diff_root,
                    change_list,
                    summary,
                    selected_path: None,
                });
            }
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::{
        MatchStrategy, MatchedPair, MatchedPairCompareResult, UnmatchedFile, UnmatchedSide,
    };
    use crate::batch_report::{BatchIssue, build_batch_results};
    use crate::batch_scan::BatchFileRecord;
    use crate::diff::{DiffNode, DiffStatus, DiffSummary};
    use std::path::PathBuf;

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

    fn pair(file_name: &str) -> MatchedPair {
        MatchedPair {
            file_name: file_name.to_string(),
            left: record(&format!("left/{file_name}")),
            right: record(&format!("right/{file_name}")),
            match_strategy: MatchStrategy::FileName,
        }
    }

    fn diff_root(path: &str, status: DiffStatus) -> DiffNode {
        DiffNode {
            path: path.to_string(),
            status,
            left_value: None,
            right_value: None,
            summary: "test".to_string(),
            children: Vec::new(),
        }
    }

    fn assert_summary_eq(actual: &DiffSummary, expected: &DiffSummary) {
        assert_eq!(actual.modified, expected.modified);
        assert_eq!(actual.added, expected.added);
        assert_eq!(actual.removed, expected.removed);
        assert_eq!(actual.reordered, expected.reordered);
        assert_eq!(actual.error, expected.error);
    }

    #[test]
    fn keeps_identical_results_in_identical_bucket() {
        let matched = vec![MatchedPairCompareResult::identical(pair("same.png"))];

        let report = build_batch_results(matched, vec![], vec![], vec![]);

        assert_eq!(report.identical.len(), 1);
        assert!(report.different.is_empty());
        assert_eq!(report.identical[0].pair.file_name, "same.png");
    }

    #[test]
    fn keeps_different_payload_and_sets_selected_path_to_none() {
        let expected_diff_root = diff_root("changed", DiffStatus::Modified);
        let expected_change_list = vec![diff_root("changed.field", DiffStatus::Modified)];
        let expected_summary = DiffSummary {
            modified: 1,
            ..DiffSummary::default()
        };
        let matched = vec![MatchedPairCompareResult::different(
            pair("changed.png"),
            expected_diff_root.clone(),
            expected_change_list.clone(),
            expected_summary.clone(),
        )];

        let report = build_batch_results(matched, vec![], vec![], vec![]);

        assert_eq!(report.different.len(), 1);
        assert!(report.identical.is_empty());
        assert_eq!(report.different[0].pair.file_name, "changed.png");
        assert_eq!(report.different[0].diff_root, expected_diff_root);
        assert_eq!(report.different[0].change_list, expected_change_list);
        assert_summary_eq(&report.different[0].summary, &expected_summary);
        assert_eq!(report.different[0].selected_path, None);
    }

    #[test]
    fn keeps_scan_issues_outside_result_categories() {
        let left_only = vec![UnmatchedFile {
            side: UnmatchedSide::Left,
            file: record("left/left-only.png"),
            reason: "missing on right".to_string(),
        }];
        let right_only = vec![UnmatchedFile {
            side: UnmatchedSide::Right,
            file: record("right/right-only.png"),
            reason: "missing on left".to_string(),
        }];
        let issues = vec![BatchIssue::ScanFailure {
            side: UnmatchedSide::Left,
            path: PathBuf::from("left/source"),
            reason: "permission denied".to_string(),
        }];

        let report = build_batch_results(
            vec![],
            left_only.clone(),
            right_only.clone(),
            issues.clone(),
        );

        assert_eq!(report.issues, issues);
        assert_eq!(report.left_only, left_only);
        assert_eq!(report.right_only, right_only);
        assert!(report.identical.is_empty());
        assert!(report.different.is_empty());
    }
}
