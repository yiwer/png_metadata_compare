use crate::error::CompareError;
use crate::metadata::MetadataLoadResult;
use serde_json::Value;
use std::collections::BTreeSet;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffStatus {
    Unchanged,
    Modified,
    Added,
    Removed,
    Reordered,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffNode {
    pub path: String,
    pub status: DiffStatus,
    pub left_value: Option<String>,
    pub right_value: Option<String>,
    pub summary: String,
    pub children: Vec<DiffNode>,
}

pub fn compare_metadata(left: &MetadataLoadResult, right: &MetadataLoadResult) -> DiffNode {
    match (left, right) {
        (MetadataLoadResult::Error(left_err), MetadataLoadResult::Error(right_err)) => {
            error_node("", Some(left_err), Some(right_err))
        }
        (MetadataLoadResult::Error(left_err), MetadataLoadResult::Parsed(_)) => {
            error_node("", Some(left_err), None)
        }
        (MetadataLoadResult::Parsed(_), MetadataLoadResult::Error(right_err)) => {
            error_node("", None, Some(right_err))
        }
        (MetadataLoadResult::Parsed(left_value), MetadataLoadResult::Parsed(right_value)) => {
            compare_values("", Some(left_value), Some(right_value))
        }
    }
}

fn compare_values(path: &str, left: Option<&Value>, right: Option<&Value>) -> DiffNode {
    match (left, right) {
        (Some(Value::Object(left_map)), Some(Value::Object(right_map))) => {
            let mut keys = BTreeSet::new();
            keys.extend(left_map.keys().cloned());
            keys.extend(right_map.keys().cloned());

            let children = keys
                .into_iter()
                .map(|key| compare_values(&join_path(path, &key), left_map.get(&key), right_map.get(&key)))
                .collect();

            aggregate_node(path, left, right, children)
        }
        (None, Some(Value::Object(right_map))) => {
            let children = right_map
                .iter()
                .map(|(key, value)| compare_values(&join_path(path, key), None, Some(value)))
                .collect();

            aggregate_node(path, left, right, children)
        }
        (Some(Value::Object(left_map)), None) => {
            let children = left_map
                .iter()
                .map(|(key, value)| compare_values(&join_path(path, key), Some(value), None))
                .collect();

            aggregate_node(path, left, right, children)
        }
        (Some(Value::Array(left_items)), Some(Value::Array(right_items))) => {
            let max_len = left_items.len().max(right_items.len());
            let children = (0..max_len)
                .map(|index| {
                    compare_values(
                        &join_index_path(path, index),
                        left_items.get(index),
                        right_items.get(index),
                    )
                })
                .collect();

            aggregate_node(path, left, right, children)
        }
        (None, Some(Value::Array(right_items))) => {
            let children = right_items
                .iter()
                .enumerate()
                .map(|(index, value)| compare_values(&join_index_path(path, index), None, Some(value)))
                .collect();

            aggregate_node(path, left, right, children)
        }
        (Some(Value::Array(left_items)), None) => {
            let children = left_items
                .iter()
                .enumerate()
                .map(|(index, value)| compare_values(&join_index_path(path, index), Some(value), None))
                .collect();

            aggregate_node(path, left, right, children)
        }
        _ => {
            let status = match (left, right) {
                (Some(left_value), Some(right_value)) if left_value == right_value => DiffStatus::Unchanged,
                (None, Some(_)) => DiffStatus::Added,
                (Some(_), None) => DiffStatus::Removed,
                (Some(_), Some(_)) => DiffStatus::Modified,
                (None, None) => DiffStatus::Unchanged,
            };

            value_node(path, status, left, right)
        }
    }
}

fn value_node(path: &str, status: DiffStatus, left: Option<&Value>, right: Option<&Value>) -> DiffNode {
    DiffNode {
        path: join_path("", path),
        status: status.clone(),
        left_value: compact_json(left),
        right_value: compact_json(right),
        summary: describe_change(path, &status),
        children: Vec::new(),
    }
}

fn error_node(path: &str, left: Option<&CompareError>, right: Option<&CompareError>) -> DiffNode {
    let summary = match (left, right) {
        (Some(left_err), Some(right_err)) => format!("Load errors: left={left_err}; right={right_err}"),
        (Some(left_err), None) => format!("Load error on left: {left_err}"),
        (None, Some(right_err)) => format!("Load error on right: {right_err}"),
        (None, None) => "Metadata load error".to_string(),
    };

    DiffNode {
        path: join_path("", path),
        status: DiffStatus::Error,
        left_value: left.map(ToString::to_string),
        right_value: right.map(ToString::to_string),
        summary,
        children: Vec::new(),
    }
}

fn aggregate_node(path: &str, left: Option<&Value>, right: Option<&Value>, children: Vec<DiffNode>) -> DiffNode {
    let status = match (left, right) {
        (None, Some(_)) => DiffStatus::Added,
        (Some(_), None) => DiffStatus::Removed,
        _ if children.iter().all(|child| child.status == DiffStatus::Unchanged) => DiffStatus::Unchanged,
        _ if children.iter().any(|child| child.status == DiffStatus::Error) => DiffStatus::Error,
        _ => DiffStatus::Modified,
    };

    DiffNode {
        path: join_path("", path),
        status: status.clone(),
        left_value: None,
        right_value: None,
        summary: describe_change(path, &status),
        children,
    }
}

fn join_path(parent: &str, segment: &str) -> String {
    if parent.is_empty() {
        if segment.is_empty() {
            "StopPlateMetadata".to_string()
        } else {
            segment.to_string()
        }
    } else if segment.is_empty() {
        parent.to_string()
    } else {
        format!("{parent}.{segment}")
    }
}

fn join_index_path(parent: &str, index: usize) -> String {
    if parent.is_empty() {
        format!("StopPlateMetadata[{index}]")
    } else {
        format!("{parent}[{index}]")
    }
}

fn compact_json(value: Option<&Value>) -> Option<String> {
    value.and_then(|value| serde_json::to_string(value).ok())
}

fn describe_change(path: &str, status: &DiffStatus) -> String {
    let label = if path.is_empty() {
        "StopPlateMetadata"
    } else {
        path
    };

    match status {
        DiffStatus::Unchanged => format!("{label} unchanged"),
        DiffStatus::Modified => format!("{label} modified"),
        DiffStatus::Added => format!("{label} added"),
        DiffStatus::Removed => format!("{label} removed"),
        DiffStatus::Reordered => format!("{label} reordered"),
        DiffStatus::Error => format!("{label} has an error"),
    }
}

#[cfg(test)]
mod tests {
    use super::{compare_metadata, DiffStatus};
    use crate::error::CompareError;
    use crate::metadata::MetadataLoadResult;
    use serde_json::json;

    #[test]
    fn marks_scalar_change_as_modified() {
        let left = MetadataLoadResult::Parsed(json!({"name": "left"}));
        let right = MetadataLoadResult::Parsed(json!({"name": "right"}));

        let diff = compare_metadata(&left, &right);
        let child = diff
            .children
            .iter()
            .find(|node| node.path == "name")
            .unwrap_or_else(|| panic!("missing diff child for name: {diff:#?}"));

        assert_eq!(child.status, DiffStatus::Modified);
        assert_eq!(child.left_value.as_deref(), Some("\"left\""));
        assert_eq!(child.right_value.as_deref(), Some("\"right\""));
    }

    #[test]
    fn marks_missing_field_as_added() {
        let left = MetadataLoadResult::Parsed(json!({}));
        let right = MetadataLoadResult::Parsed(json!({"newField": 7}));

        let diff = compare_metadata(&left, &right);
        let child = diff
            .children
            .iter()
            .find(|node| node.path == "newField")
            .unwrap_or_else(|| panic!("missing diff child for newField: {diff:#?}"));

        assert_eq!(child.status, DiffStatus::Added);
        assert_eq!(child.left_value, None);
        assert_eq!(child.right_value.as_deref(), Some("7"));
    }

    #[test]
    fn keeps_added_compound_values_explorable() {
        let left = MetadataLoadResult::Parsed(json!({}));
        let right = MetadataLoadResult::Parsed(json!({
            "items": [
                {"name": "alpha"}
            ]
        }));

        let diff = compare_metadata(&left, &right);
        let items = diff
            .children
            .iter()
            .find(|node| node.path == "items")
            .unwrap_or_else(|| panic!("missing diff child for items: {diff:#?}"));

        assert_eq!(items.status, DiffStatus::Added);
        assert!(items.left_value.is_none());
        assert!(items.right_value.is_none());

        let index_node = items
            .children
            .iter()
            .find(|node| node.path == "items[0]")
            .unwrap_or_else(|| panic!("missing diff child for items[0]: {items:#?}"));

        assert_eq!(index_node.status, DiffStatus::Added);

        let leaf = index_node
            .children
            .iter()
            .find(|node| node.path == "items[0].name")
            .unwrap_or_else(|| panic!("missing diff child for items[0].name: {index_node:#?}"));

        assert_eq!(leaf.status, DiffStatus::Added);
        assert_eq!(leaf.left_value, None);
        assert_eq!(leaf.right_value.as_deref(), Some("\"alpha\""));
    }

    #[test]
    fn preserves_removed_compound_parent_status() {
        let left = MetadataLoadResult::Parsed(json!({
            "items": [
                {"name": "alpha"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({}));

        let diff = compare_metadata(&left, &right);
        let items = diff
            .children
            .iter()
            .find(|node| node.path == "items")
            .unwrap_or_else(|| panic!("missing diff child for items: {diff:#?}"));

        assert_eq!(items.status, DiffStatus::Removed);

        let index_node = items
            .children
            .iter()
            .find(|node| node.path == "items[0]")
            .unwrap_or_else(|| panic!("missing diff child for items[0]: {items:#?}"));

        assert_eq!(index_node.status, DiffStatus::Removed);
    }

    #[test]
    fn preserves_empty_added_and_removed_compound_parent_statuses() {
        let added = compare_metadata(
            &MetadataLoadResult::Parsed(json!({})),
            &MetadataLoadResult::Parsed(json!({"emptyObject": {}, "emptyArray": []})),
        );
        let removed = compare_metadata(
            &MetadataLoadResult::Parsed(json!({"emptyObject": {}, "emptyArray": []})),
            &MetadataLoadResult::Parsed(json!({})),
        );

        let added_object = added
            .children
            .iter()
            .find(|node| node.path == "emptyObject")
            .unwrap_or_else(|| panic!("missing diff child for emptyObject: {added:#?}"));
        let added_array = added
            .children
            .iter()
            .find(|node| node.path == "emptyArray")
            .unwrap_or_else(|| panic!("missing diff child for emptyArray: {added:#?}"));
        let removed_object = removed
            .children
            .iter()
            .find(|node| node.path == "emptyObject")
            .unwrap_or_else(|| panic!("missing diff child for emptyObject: {removed:#?}"));
        let removed_array = removed
            .children
            .iter()
            .find(|node| node.path == "emptyArray")
            .unwrap_or_else(|| panic!("missing diff child for emptyArray: {removed:#?}"));

        assert_eq!(added_object.status, DiffStatus::Added);
        assert_eq!(added_array.status, DiffStatus::Added);
        assert_eq!(removed_object.status, DiffStatus::Removed);
        assert_eq!(removed_array.status, DiffStatus::Removed);
        assert!(added_object.children.is_empty());
        assert!(added_array.children.is_empty());
        assert!(removed_object.children.is_empty());
        assert!(removed_array.children.is_empty());
    }

    #[test]
    fn uses_bracketed_paths_for_array_indexes() {
        let left = MetadataLoadResult::Parsed(json!({
            "items": ["left"]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "items": ["right"]
        }));

        let diff = compare_metadata(&left, &right);
        let items = diff
            .children
            .iter()
            .find(|node| node.path == "items")
            .unwrap_or_else(|| panic!("missing diff child for items: {diff:#?}"));

        let index_node = items
            .children
            .iter()
            .find(|node| node.path == "items[0]")
            .unwrap_or_else(|| panic!("missing diff child for items[0]: {items:#?}"));

        assert_eq!(index_node.status, DiffStatus::Modified);
        assert_eq!(index_node.left_value.as_deref(), Some("\"left\""));
        assert_eq!(index_node.right_value.as_deref(), Some("\"right\""));
        assert!(
            items.children.iter().all(|node| node.path != "items.0"),
            "unexpected dotted array index path in children: {items:#?}"
        );
    }

    #[test]
    fn turns_load_error_into_error_node() {
        let left = MetadataLoadResult::Error(CompareError::MissingStopPlateMetadata);
        let right = MetadataLoadResult::Parsed(json!({"name": "ok"}));

        let diff = compare_metadata(&left, &right);

        assert_eq!(diff.path, "StopPlateMetadata");
        assert_eq!(diff.status, DiffStatus::Error);
        assert!(
            diff.summary.contains("MissingStopPlateMetadata")
                || diff.summary.contains("missing StopPlate metadata"),
            "unexpected summary: {}",
            diff.summary
        );
    }
}
