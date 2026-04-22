use crate::batch_scan::BatchFileRecord;
use crate::diff::{DiffNode, DiffSummary};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

pub type MatchedPairCompareResult = (MatchedPair, DiffNode, Vec<DiffNode>, DiffSummary);

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

    for (pair, diff_root, change_list, summary) in matched {
        if summary.total() == 0 {
            report.identical.push(IdenticalPairResult { pair });
        } else {
            report.different.push(DifferentPairResult {
                pair,
                diff_root,
                change_list,
                summary,
                selected_path: None,
            });
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::{MatchStrategy, MatchedPair, UnmatchedFile, UnmatchedSide};
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

    #[test]
    fn classifies_zero_total_summary_as_identical() {
        let matched = vec![(
            pair("same.png"),
            diff_root("same", DiffStatus::Unchanged),
            vec![],
            DiffSummary::default(),
        )];

        let report = build_batch_results(matched, vec![], vec![], vec![]);

        assert_eq!(report.identical.len(), 1);
        assert!(report.different.is_empty());
        assert_eq!(report.identical[0].pair.file_name, "same.png");
    }

    #[test]
    fn classifies_non_zero_total_summary_as_different() {
        let matched = vec![(
            pair("changed.png"),
            diff_root("changed", DiffStatus::Modified),
            vec![diff_root("changed.field", DiffStatus::Modified)],
            DiffSummary {
                modified: 1,
                ..DiffSummary::default()
            },
        )];

        let report = build_batch_results(matched, vec![], vec![], vec![]);

        assert_eq!(report.different.len(), 1);
        assert!(report.identical.is_empty());
        assert_eq!(report.different[0].pair.file_name, "changed.png");
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

        let report =
            build_batch_results(vec![], left_only.clone(), right_only.clone(), issues.clone());

        assert_eq!(report.issues, issues);
        assert_eq!(report.left_only, left_only);
        assert_eq!(report.right_only, right_only);
        assert!(report.identical.is_empty());
        assert!(report.different.is_empty());
    }
}
