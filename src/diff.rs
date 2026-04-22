use crate::error::CompareError;
use crate::metadata::MetadataLoadResult;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

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

type BusinessKeyFn = fn(&Value) -> Option<String>;

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

            aggregate_node(path, left.is_some(), right.is_some(), children)
        }
        (None, Some(Value::Object(right_map))) => {
            let children = right_map
                .iter()
                .map(|(key, value)| compare_values(&join_path(path, key), None, Some(value)))
                .collect();

            aggregate_node(path, left.is_some(), right.is_some(), children)
        }
        (Some(Value::Object(left_map)), None) => {
            let children = left_map
                .iter()
                .map(|(key, value)| compare_values(&join_path(path, key), Some(value), None))
                .collect();

            aggregate_node(path, left.is_some(), right.is_some(), children)
        }
        (Some(Value::Array(left_items)), Some(Value::Array(right_items))) => {
            compare_array(path, Some(left_items), Some(right_items))
        }
        (None, Some(Value::Array(right_items))) => {
            compare_array(path, None, Some(right_items))
        }
        (Some(Value::Array(left_items)), None) => {
            compare_array(path, Some(left_items), None)
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

fn compare_array(path: &str, left: Option<&[Value]>, right: Option<&[Value]>) -> DiffNode {
    let left_items = left.unwrap_or(&[]);
    let right_items = right.unwrap_or(&[]);

    if let Some(key_fn) = business_key_for_path(path) {
        return compare_keyed_array(path, left.is_some(), right.is_some(), left_items, right_items, key_fn);
    }

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

    aggregate_node(path, left.is_some(), right.is_some(), children)
}

fn compare_keyed_array(
    path: &str,
    left_present: bool,
    right_present: bool,
    left: &[Value],
    right: &[Value],
    key_fn: BusinessKeyFn,
) -> DiffNode {
    let mut children = Vec::new();
    let left_index = build_key_index(path, left, key_fn);
    let right_index = build_key_index(path, right, key_fn);

    if let Err(error) = &left_index {
        children.push(compare_error_node(
            &format!("{path}.__error__.left"),
            error.clone(),
        ));
    }
    if let Err(error) = &right_index {
        children.push(compare_error_node(
            &format!("{path}.__error__.right"),
            error.clone(),
        ));
    }

    if !children.is_empty() {
        return aggregate_node(path, left_present, right_present, children);
    }

    let left_index = left_index.expect("left_index checked above");
    let right_index = right_index.expect("right_index checked above");
    let mut keys = BTreeSet::new();
    keys.extend(left_index.keys().cloned());
    keys.extend(right_index.keys().cloned());

    for key in keys {
        let child_path = join_key_path(path, &key);
        match (left_index.get(&key), right_index.get(&key)) {
            (Some((left_pos, left_value)), Some((right_pos, right_value))) => {
                children.push(compare_values(&child_path, Some(left_value), Some(right_value)));
                if left_pos != right_pos {
                    children.push(reorder_node(&child_path, *left_pos, *right_pos));
                }
            }
            (Some((_, left_value)), None) => children.push(compare_values(&child_path, Some(left_value), None)),
            (None, Some((_, right_value))) => children.push(compare_values(&child_path, None, Some(right_value))),
            (None, None) => {}
        }
    }

    aggregate_node(path, left_present, right_present, children)
}

fn build_key_index<'a>(
    path: &str,
    values: &'a [Value],
    key_fn: BusinessKeyFn,
) -> Result<BTreeMap<String, (usize, &'a Value)>, CompareError> {
    let mut index = BTreeMap::new();
    for (position, value) in values.iter().enumerate() {
        let key = key_fn(value).unwrap_or_else(|| position.to_string());
        if index.insert(key.clone(), (position, value)).is_some() {
            return Err(CompareError::AmbiguousBusinessKey {
                path: join_path("", path),
                key,
            });
        }
    }
    Ok(index)
}

fn business_key_for_path(path: &str) -> Option<BusinessKeyFn> {
    if path.ends_with("GroupItems") {
        Some(|value| value.get("SequenceNo")?.as_str().map(str::to_owned))
    } else if path.ends_with("Lines") {
        Some(|value| {
            let line_name = value.get("LineName")?.as_str()?;
            let direction = value.get("Direction").and_then(|direction| direction.as_str());
            match direction {
                Some(direction) if !direction.is_empty() => Some(format!("{line_name}|{direction}")),
                _ => Some(line_name.to_owned()),
            }
        })
    } else if path.ends_with("RouteStops") {
        Some(|value| {
            let sequence = value.get("Sequence").and_then(|sequence| sequence.as_i64());
            let name = value.get("Name").and_then(|name| name.as_str());
            match (sequence, name) {
                (Some(sequence), Some(name)) => Some(format!("{sequence}|{name}")),
                (_, Some(name)) => Some(name.to_owned()),
                _ => None,
            }
        })
    } else {
        None
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

fn compare_error_node(path: &str, error: CompareError) -> DiffNode {
    DiffNode {
        path: join_path("", path),
        status: DiffStatus::Error,
        left_value: None,
        right_value: None,
        summary: error.to_string(),
        children: Vec::new(),
    }
}

fn reorder_node(path: &str, left_pos: usize, right_pos: usize) -> DiffNode {
    DiffNode {
        path: format!("{path}.__order__"),
        status: DiffStatus::Reordered,
        left_value: Some(left_pos.to_string()),
        right_value: Some(right_pos.to_string()),
        summary: format!("{path} reordered: {left_pos} -> {right_pos}"),
        children: Vec::new(),
    }
}

fn aggregate_node(path: &str, left_present: bool, right_present: bool, children: Vec<DiffNode>) -> DiffNode {
    let status = match (left_present, right_present) {
        (false, true) => DiffStatus::Added,
        (true, false) => DiffStatus::Removed,
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

fn join_key_path(parent: &str, key: &str) -> String {
    if parent.is_empty() {
        format!("StopPlateMetadata[{key}]")
    } else {
        format!("{parent}[{key}]")
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

pub fn flatten_changes(node: &DiffNode) -> Vec<DiffNode> {
    let mut changes = Vec::new();
    collect_changes(node, &mut changes);
    changes
}

fn collect_changes(node: &DiffNode, changes: &mut Vec<DiffNode>) {
    if node.status != DiffStatus::Unchanged {
        changes.push(node.clone());
    }

    for child in &node.children {
        collect_changes(child, changes);
    }
}

#[cfg(test)]
mod tests {
    use super::{compare_metadata, flatten_changes, DiffStatus};
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

    #[test]
    fn matches_lines_by_line_name_and_direction() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [
                {"LineName": "B932", "Direction": "Terminal", "PriceDescription": "1"},
                {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "2"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [
                {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "3"},
                {"LineName": "B932", "Direction": "Terminal", "PriceDescription": "1"}
            ]
        }));

        let diff = compare_metadata(&left, &right);
        let lines = diff
            .children
            .iter()
            .find(|node| node.path == "Lines")
            .unwrap_or_else(|| panic!("missing diff child for Lines: {diff:#?}"));

        assert!(
            lines.children.iter().any(|node| node.status == DiffStatus::Reordered),
            "expected a reorder node in Lines diff: {lines:#?}"
        );
        assert!(
            lines.children.iter().any(|node| {
                node.path.contains("M375") && node.status == DiffStatus::Modified
            }),
            "expected modified keyed line diff for M375: {lines:#?}"
        );
    }

    #[test]
    fn marks_added_route_stop_when_business_key_only_exists_on_right() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{"LineName": "B932", "Direction": "Terminal", "RouteStops": []}]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{"Sequence": 8, "Name": "CurrentStop"}]
            }]
        }));

        let diff = compare_metadata(&left, &right);
        let changes = flatten_changes(&diff);

        assert!(
            changes.iter().any(|node| {
                node.path.contains("RouteStops[8|CurrentStop]") && node.status == DiffStatus::Added
            }),
            "expected added keyed route stop in flattened changes: {changes:#?}"
        );
    }

    #[test]
    fn creates_error_for_ambiguous_business_key() {
        let left = MetadataLoadResult::Parsed(json!({
            "GroupItems": [
                {"SequenceNo": "①", "LineNames": "B932"},
                {"SequenceNo": "①", "LineNames": "M375"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({"GroupItems": []}));

        let diff = compare_metadata(&left, &right);
        let group_items = diff
            .children
            .iter()
            .find(|node| node.path == "GroupItems")
            .unwrap_or_else(|| panic!("missing diff child for GroupItems: {diff:#?}"));

        assert!(
            group_items
                .children
                .iter()
                .any(|node| node.status == DiffStatus::Error),
            "expected error node for ambiguous business key: {group_items:#?}"
        );
    }
}
