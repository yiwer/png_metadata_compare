use std::path::PathBuf;

use serde::Serialize;
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

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiError {
    pub code: &'static str,
    pub message: String,
}

#[allow(dead_code)]
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
