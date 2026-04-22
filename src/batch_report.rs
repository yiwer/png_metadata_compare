use crate::batch_scan::BatchFileRecord;

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
