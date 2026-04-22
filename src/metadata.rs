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
    use serde_json::json;

    #[test]
    fn parses_valid_json_into_value() {
        let result = load_metadata(Ok(r#"{"name":"stop-plate","count":2}"#.to_string()));

        match result {
            MetadataLoadResult::Parsed(value) => {
                assert_eq!(value, json!({"name": "stop-plate", "count": 2}));
            }
            MetadataLoadResult::Error(err) => panic!("expected parsed metadata, got error: {err}"),
        }
    }

    #[test]
    fn converts_json_failure_to_error_result() {
        let result = load_metadata(Ok("{not-json}".to_string()));

        match result {
            MetadataLoadResult::Parsed(value) => {
                panic!("expected json parse failure, got parsed value: {value}");
            }
            MetadataLoadResult::Error(CompareError::MetadataJson(message)) => {
                assert!(!message.is_empty());
            }
            MetadataLoadResult::Error(err) => {
                panic!("expected metadata json error, got: {err}");
            }
        }
    }

    #[test]
    fn preserves_upstream_extraction_errors() {
        let result = load_metadata(Err(CompareError::MissingStopPlateMetadata));

        match result {
            MetadataLoadResult::Parsed(value) => {
                panic!("expected upstream extraction error, got parsed value: {value}");
            }
            MetadataLoadResult::Error(err) => {
                assert!(matches!(err, CompareError::MissingStopPlateMetadata));
            }
        }
    }
}
