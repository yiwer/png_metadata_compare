use std::path::PathBuf;

use thiserror::Error;

#[allow(dead_code)]
#[derive(Clone, Debug, Error)]
pub enum CompareError {
    #[error("failed to read file {path:?}: {reason}")]
    FileRead { path: PathBuf, reason: String },
    #[error("invalid PNG signature")]
    InvalidPngSignature,
    #[error("truncated PNG chunk")]
    TruncatedChunk,
    #[error("missing StopPlate metadata")]
    MissingStopPlateMetadata,
    #[error("unsupported compressed text metadata")]
    UnsupportedCompressedText,
    #[error("invalid international text metadata: {0}")]
    InvalidInternationalText(String),
    #[error("metadata is not valid UTF-8: {0}")]
    MetadataUtf8(String),
    #[error("metadata is not valid JSON: {0}")]
    MetadataJson(String),
    #[error("ambiguous business key in {path}: {key}")]
    AmbiguousBusinessKey { path: String, key: String },
}
