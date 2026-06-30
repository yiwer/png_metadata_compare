use crate::config::{CompareConfig, config};
use crate::error::CompareError;
use crate::metadata::MetadataLoadResult;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffStatus {
    Unchanged,
    Modified,
    Added,
    Removed,
    Reordered,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiffNode {
    pub path: String,
    pub status: DiffStatus,
    pub left_value: Option<String>,
    pub right_value: Option<String>,
    pub summary: String,
    pub children: Vec<DiffNode>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DiffSummary {
    pub modified: usize,
    pub added: usize,
    pub removed: usize,
    pub reordered: usize,
    pub error: usize,
}

impl DiffSummary {
    pub fn total(&self) -> usize {
        self.modified + self.added + self.removed + self.reordered + self.error
    }
}

type BusinessKeyFn = fn(&Value) -> Option<String>;

struct KeyedArrayIndex<'a> {
    items: BTreeMap<String, Vec<&'a Value>>,
    errors: Vec<DiffNode>,
}

pub fn compare_metadata(left: &MetadataLoadResult, right: &MetadataLoadResult) -> DiffNode {
    compare_metadata_with_config(config(), left, right)
}

/// Same as [`compare_metadata`] but with an explicit config instead of the
/// process-wide one loaded from `compare-config.json` — lets tests pin the
/// rules they assert against.
pub fn compare_metadata_with_config(
    cfg: &CompareConfig,
    left: &MetadataLoadResult,
    right: &MetadataLoadResult,
) -> DiffNode {
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
            compare_values(cfg, "", Some(left_value), Some(right_value))
        }
    }
}

fn compare_values(
    cfg: &CompareConfig,
    path: &str,
    left: Option<&Value>,
    right: Option<&Value>,
) -> DiffNode {
    match (left, right) {
        (Some(Value::Object(left_map)), Some(Value::Object(right_map))) => {
            let mut keys = BTreeSet::new();
            keys.extend(left_map.keys().cloned());
            keys.extend(right_map.keys().cloned());
            for known in schema_keys_for_path(path) {
                keys.insert((*known).to_string());
            }
            keys.retain(|key| !is_ignored_field(cfg, &join_path(path, key)));

            let null = Value::Null;
            let empty_object = Value::Object(serde_json::Map::new());
            let empty_array = Value::Array(Vec::new());
            let placeholder_for = |peer: &Value| -> &Value {
                match peer {
                    Value::Array(_) => &empty_array,
                    Value::Object(_) => &empty_object,
                    _ => &null,
                }
            };

            let children = keys
                .into_iter()
                .map(|key| {
                    let left_value = left_map.get(&key);
                    let right_value = right_map.get(&key);
                    let (left_filled, right_filled) = match (left_value, right_value) {
                        (Some(_), Some(_)) => (left_value, right_value),
                        (Some(lv), None) => (Some(lv), Some(placeholder_for(lv))),
                        (None, Some(rv)) => (Some(placeholder_for(rv)), Some(rv)),
                        (None, None) => (Some(&null), Some(&null)),
                    };
                    compare_values(cfg, &join_path(path, &key), left_filled, right_filled)
                })
                .collect();

            aggregate_node(path, true, true, children)
        }
        (None, Some(Value::Object(right_map))) => {
            let children = right_map
                .iter()
                .filter(|(key, _)| !is_ignored_field(cfg, &join_path(path, key)))
                .map(|(key, value)| compare_values(cfg, &join_path(path, key), None, Some(value)))
                .collect();

            aggregate_node(path, left.is_some(), right.is_some(), children)
        }
        (Some(Value::Object(left_map)), None) => {
            let children = left_map
                .iter()
                .filter(|(key, _)| !is_ignored_field(cfg, &join_path(path, key)))
                .map(|(key, value)| compare_values(cfg, &join_path(path, key), Some(value), None))
                .collect();

            aggregate_node(path, left.is_some(), right.is_some(), children)
        }
        (Some(Value::Array(left_items)), Some(Value::Array(right_items))) => {
            compare_array(cfg, path, Some(left_items), Some(right_items))
        }
        (None, Some(Value::Array(right_items))) => {
            compare_array(cfg, path, None, Some(right_items))
        }
        (Some(Value::Array(left_items)), None) => compare_array(cfg, path, Some(left_items), None),
        _ => {
            let status = match (left, right) {
                (Some(left_value), Some(right_value))
                    if scalars_are_equivalent_at(cfg, path, left_value, right_value) =>
                {
                    DiffStatus::Unchanged
                }
                (None, Some(rv)) if is_blank_value(rv) => DiffStatus::Unchanged,
                (Some(lv), None) if is_blank_value(lv) => DiffStatus::Unchanged,
                (None, Some(_)) => DiffStatus::Added,
                (Some(_), None) => DiffStatus::Removed,
                (Some(_), Some(_)) => DiffStatus::Modified,
                (None, None) => DiffStatus::Unchanged,
            };

            value_node(path, status, left, right)
        }
    }
}

fn compare_array(
    cfg: &CompareConfig,
    path: &str,
    left: Option<&[Value]>,
    right: Option<&[Value]>,
) -> DiffNode {
    let left_items = left.unwrap_or(&[]);
    let right_items = right.unwrap_or(&[]);

    if let Some(key_fn) = business_key_for_path(path) {
        return compare_keyed_array(
            cfg,
            path,
            left.is_some(),
            right.is_some(),
            left_items,
            right_items,
            key_fn,
        );
    }

    let max_len = left_items.len().max(right_items.len());
    let children = (0..max_len)
        .map(|index| {
            compare_values(
                cfg,
                &join_index_path(path, index),
                left_items.get(index),
                right_items.get(index),
            )
        })
        .collect();

    aggregate_node(path, left.is_some(), right.is_some(), children)
}

fn compare_keyed_array(
    cfg: &CompareConfig,
    path: &str,
    left_present: bool,
    right_present: bool,
    left: &[Value],
    right: &[Value],
    key_fn: BusinessKeyFn,
) -> DiffNode {
    let left_index = build_key_index(path, left, key_fn, "left");
    let right_index = build_key_index(path, right, key_fn, "right");
    let mut children = Vec::new();
    children.extend(left_index.errors);
    children.extend(right_index.errors);

    let mut keys = BTreeSet::new();
    keys.extend(left_index.items.keys().cloned());
    keys.extend(right_index.items.keys().cloned());

    let empty: Vec<&Value> = Vec::new();
    for key in keys {
        let left_list = left_index.items.get(&key).unwrap_or(&empty);
        let right_list = right_index.items.get(&key).unwrap_or(&empty);
        let with_suffix = left_list.len().max(right_list.len()) > 1;
        let matched = left_list.len().min(right_list.len());

        for i in 0..matched {
            let child_path = join_keyed_occurrence_path(path, &key, i, with_suffix);
            children.push(compare_values(
                cfg,
                &child_path,
                Some(left_list[i]),
                Some(right_list[i]),
            ));
        }
        for i in matched..left_list.len() {
            let child_path = join_keyed_occurrence_path(path, &key, i, with_suffix);
            children.push(compare_values(cfg, &child_path, Some(left_list[i]), None));
        }
        for i in matched..right_list.len() {
            let child_path = join_keyed_occurrence_path(path, &key, i, with_suffix);
            children.push(compare_values(cfg, &child_path, None, Some(right_list[i])));
        }
    }

    aggregate_node(path, left_present, right_present, children)
}

fn build_key_index<'a>(
    path: &str,
    values: &'a [Value],
    key_fn: BusinessKeyFn,
    side: &str,
) -> KeyedArrayIndex<'a> {
    let mut items: BTreeMap<String, Vec<&'a Value>> = BTreeMap::new();
    let mut errors = Vec::new();

    for (position, value) in values.iter().enumerate() {
        let Some(key) = key_fn(value) else {
            errors.push(keyed_array_issue_node(
                path,
                side,
                position,
                format!("missing business key in {} at {path}[{position}]", side),
            ));
            continue;
        };
        items.entry(key).or_default().push(value);
    }

    KeyedArrayIndex { items, errors }
}

fn business_key_for_path(path: &str) -> Option<BusinessKeyFn> {
    match terminal_path_segment(path) {
        "GroupItems" => Some(|value| value.get("SequenceNo")?.as_str().map(str::to_owned)),
        // 只按线路名匹配：开往方向是会被改动的属性（如终点站更名），
        // 进键会让同名线路各自落单成 仅左/仅右；同名重复项走消费式 #N 配对。
        "Lines" => Some(|value| value.get("LineName")?.as_str().map(str::to_owned)),
        "RouteStops" => Some(|value| {
            value
                .get("Name")
                .and_then(|name| name.as_str())
                .map(str::to_owned)
        }),
        _ => None,
    }
}

fn terminal_path_segment(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or(path)
}

fn schema_keys_for_path(path: &str) -> &'static [&'static str] {
    const ROOT: &[&str] = &[
        "StopId",
        "StopName",
        "StopEngName",
        "OriName",
        "RoadName",
        "DirectionOnRoad",
        "DistrictName",
        "StreetCommitteeName",
        "QRCode",
        "HasHints",
        "Hints",
        "IsGroupPrint",
        "GroupItems",
        "IsBack",
        "FrameSize",
        "Lines",
        "RenderTime",
    ];
    const GROUP_ITEM: &[&str] = &["SequenceNo", "LineNames", "Distance", "IsCurrent"];
    const LINE: &[&str] = &[
        "LineName",
        "Direction",
        "FirstStopName",
        "LastStopName",
        "NextStop",
        "CurrentStopSequence",
        "IsStarting",
        "IsEnding",
        "HeadBusCorpName",
        "TicketType",
        "PriceDescription",
        "ServiceTimeDescription",
        "ScheduledServiceDescription",
        "LinePattern",
        "RouteStops",
    ];
    // Sequence 不参与 schema 补全也不参与比对——位置由排序展示承载，重复输出会变噪音。
    const ROUTE_STOP_VISIBLE: &[&str] = &["Name", "BuildingType", "RoadName"];

    match normalize_path_for_schema(path).as_str() {
        "" => ROOT,
        "GroupItems[*]" => GROUP_ITEM,
        "Lines[*]" => LINE,
        "Lines[*].RouteStops[*]" => ROUTE_STOP_VISIBLE,
        _ => &[],
    }
}

/// 强制忽略的字段——即使两侧都有值也不进入 diff。
/// 来源：`compare-config.json` 的 `ignored_fields`，缺省含 `Lines[*].RouteStops[*].Sequence`
/// （列表已按 Sequence 排序展示，再报 Sequence 改动是噪音）。
fn is_ignored_field(cfg: &CompareConfig, path: &str) -> bool {
    cfg.is_ignored(&normalize_path_for_schema(path))
}

fn is_blank_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        // 渲染端对"无内容"的数组字段时而写 null 时而写 []（如 GroupItems）——
        // 两种写法业务上都是"没有条目"，视为空白等价。
        Value::Array(items) => items.is_empty(),
        _ => false,
    }
}

/// Equivalence with optional per-path canonicalisation from `compare-config.json`.
/// String values matching an entry in the equivalence map are folded to their canonical
/// form before comparison; other types fall back to literal equality + blank-value
/// equivalence (null / "" / [] are interchangeable).
/// A value whose equivalence-map target is `"*"` is a wildcard: if it appears on either
/// side, the scalar matches any value on the other side, including null.
fn scalars_are_equivalent_at(cfg: &CompareConfig, path: &str, left: &Value, right: &Value) -> bool {
    if is_blank_value(left) && is_blank_value(right) {
        return true;
    }
    let normalized = normalize_path_for_schema(path);
    // 任一侧的字符串值映射到 "*" → 通配，匹配对侧任何值（含 null）。
    if let Value::String(s) = left {
        if cfg.is_wildcard(&normalized, s) {
            return true;
        }
    }
    if let Value::String(s) = right {
        if cfg.is_wildcard(&normalized, s) {
            return true;
        }
    }
    if let (Value::String(ls), Value::String(rs)) = (left, right) {
        return cfg.canonicalize(&normalized, ls) == cfg.canonicalize(&normalized, rs);
    }
    left == right
}

fn normalize_path_for_schema(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut chars = path.chars();
    while let Some(c) = chars.next() {
        if c == '[' {
            out.push_str("[*]");
            for cc in chars.by_ref() {
                if cc == ']' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn value_node(
    path: &str,
    status: DiffStatus,
    left: Option<&Value>,
    right: Option<&Value>,
) -> DiffNode {
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
        (Some(left_err), Some(right_err)) => {
            format!("Load errors: left={left_err}; right={right_err}")
        }
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

fn keyed_array_issue_node(path: &str, side: &str, position: usize, summary: String) -> DiffNode {
    DiffNode {
        path: format!("{path}.__error__.{side}[{position}]"),
        status: DiffStatus::Error,
        left_value: None,
        right_value: None,
        summary,
        children: Vec::new(),
    }
}

fn aggregate_node(
    path: &str,
    left_present: bool,
    right_present: bool,
    children: Vec<DiffNode>,
) -> DiffNode {
    let status = match (left_present, right_present) {
        (false, true) => DiffStatus::Added,
        (true, false) => DiffStatus::Removed,
        _ if children
            .iter()
            .all(|child| child.status == DiffStatus::Unchanged) =>
        {
            DiffStatus::Unchanged
        }
        _ if children
            .iter()
            .any(|child| child.status == DiffStatus::Error) =>
        {
            DiffStatus::Error
        }
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

fn join_keyed_occurrence_path(
    parent: &str,
    key: &str,
    occurrence_index: usize,
    with_suffix: bool,
) -> String {
    if with_suffix {
        let labeled = format!("{key}#{}", occurrence_index + 1);
        join_key_path(parent, &labeled)
    } else {
        join_key_path(parent, key)
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

pub fn summarize_changes(changes: &[DiffNode]) -> DiffSummary {
    let mut summary = DiffSummary::default();

    for change in changes {
        match change.status {
            DiffStatus::Unchanged => {}
            DiffStatus::Modified => summary.modified += 1,
            DiffStatus::Added => summary.added += 1,
            DiffStatus::Removed => summary.removed += 1,
            DiffStatus::Reordered => summary.reordered += 1,
            DiffStatus::Error => summary.error += 1,
        }
    }

    summary
}

fn collect_changes(node: &DiffNode, changes: &mut Vec<DiffNode>) {
    if node.status != DiffStatus::Unchanged {
        let mut flat_node = node.clone();
        flat_node.children.clear();
        changes.push(flat_node);
    }

    for child in &node.children {
        collect_changes(child, changes);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiffNode, DiffStatus, compare_metadata, compare_metadata_with_config, flatten_changes,
        summarize_changes,
    };
    use crate::config::CompareConfig;
    use crate::error::CompareError;
    use crate::metadata::MetadataLoadResult;
    use serde_json::json;

    /// Pins the bundled default config so test outcomes don't depend on the
    /// user-editable compare-config.json in the working directory.
    fn compare_with_defaults(left: &MetadataLoadResult, right: &MetadataLoadResult) -> DiffNode {
        compare_metadata_with_config(&CompareConfig::default(), left, right)
    }

    #[test]
    #[ignore]
    fn dump_real_meta_diff() {
        let left_text = std::fs::read_to_string("tmp/_meta1.json").expect("read tmp/_meta1.json");
        let right_text = std::fs::read_to_string("tmp/_meta2.json").expect("read tmp/_meta2.json");
        let strip_bom = |s: &str| s.trim_start_matches('\u{feff}').to_string();
        let left = MetadataLoadResult::Parsed(serde_json::from_str(&strip_bom(&left_text)).unwrap());
        let right = MetadataLoadResult::Parsed(serde_json::from_str(&strip_bom(&right_text)).unwrap());

        // Deliberately uses the ambient config()-backed entry point — this
        // ignored helper dumps diffs exactly as the running app would compute them.
        let diff = compare_metadata(&left, &right);
        let changes = flatten_changes(&diff);
        let summary = summarize_changes(&changes);

        println!("=== summary ===");
        println!(
            "modified={}  added={}  removed={}  reordered={}  error={}  total={}",
            summary.modified,
            summary.added,
            summary.removed,
            summary.reordered,
            summary.error,
            summary.total()
        );

        println!("\n=== flattened non-Unchanged nodes ===");
        for c in &changes {
            println!(
                "[{:?}] {}\n   left  = {}\n   right = {}",
                c.status,
                c.path,
                c.left_value.as_deref().unwrap_or("<absent>"),
                c.right_value.as_deref().unwrap_or("<absent>")
            );
        }

        println!("\n=== diff tree (only non-Unchanged subtrees) ===");
        print_tree(&diff, 0);
    }

    fn print_tree(node: &DiffNode, depth: usize) {
        if node.status == DiffStatus::Unchanged {
            return;
        }
        let indent = "  ".repeat(depth);
        let value_hint = match (&node.left_value, &node.right_value) {
            (Some(l), Some(r)) => format!(" : {l} -> {r}"),
            (Some(l), None) => format!(" : {l} -> <absent>"),
            (None, Some(r)) => format!(" : <absent> -> {r}"),
            (None, None) => String::new(),
        };
        println!(
            "{indent}{:?}  {}{}",
            node.status, node.path, value_hint
        );
        for child in &node.children {
            print_tree(child, depth + 1);
        }
    }

    #[test]
    fn marks_scalar_change_as_modified() {
        let left = MetadataLoadResult::Parsed(json!({"name": "left"}));
        let right = MetadataLoadResult::Parsed(json!({"name": "right"}));

        let diff = compare_with_defaults(&left, &right);
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
    fn marks_missing_object_field_as_modified_with_null_placeholder() {
        let left = MetadataLoadResult::Parsed(json!({}));
        let right = MetadataLoadResult::Parsed(json!({"newField": 7}));

        let diff = compare_with_defaults(&left, &right);
        let child = diff
            .children
            .iter()
            .find(|node| node.path == "newField")
            .unwrap_or_else(|| panic!("missing diff child for newField: {diff:#?}"));

        assert_eq!(child.status, DiffStatus::Modified);
        assert_eq!(child.left_value.as_deref(), Some("null"));
        assert_eq!(child.right_value.as_deref(), Some("7"));
    }

    #[test]
    fn treats_null_and_empty_array_as_equivalent() {
        let left = MetadataLoadResult::Parsed(json!({"GroupItems": null}));
        let right = MetadataLoadResult::Parsed(json!({"GroupItems": []}));

        let diff = compare_with_defaults(&left, &right);
        assert_eq!(
            diff.status,
            DiffStatus::Unchanged,
            "null vs [] 应视为等价: {diff:#?}"
        );

        let reversed = compare_with_defaults(&right, &left);
        assert_eq!(
            reversed.status,
            DiffStatus::Unchanged,
            "[] vs null 应视为等价: {reversed:#?}"
        );
    }

    #[test]
    fn keeps_null_vs_non_empty_array_as_difference() {
        let left = MetadataLoadResult::Parsed(json!({"GroupItems": null}));
        let right = MetadataLoadResult::Parsed(json!({
            "GroupItems": [{"SequenceNo": "①", "LineNames": "M102", "IsCurrent": true}]
        }));

        let diff = compare_with_defaults(&left, &right);
        let child = diff
            .children
            .iter()
            .find(|node| node.path == "GroupItems")
            .unwrap_or_else(|| panic!("missing diff child for GroupItems: {diff:#?}"));

        assert_ne!(
            child.status,
            DiffStatus::Unchanged,
            "null vs 非空数组仍应是差异: {child:#?}"
        );
    }

    #[test]
    fn keeps_added_array_items_explorable_when_array_field_was_absent_on_one_side() {
        let left = MetadataLoadResult::Parsed(json!({}));
        let right = MetadataLoadResult::Parsed(json!({
            "items": [
                {"name": "alpha"}
            ]
        }));

        let diff = compare_with_defaults(&left, &right);
        let items = diff
            .children
            .iter()
            .find(|node| node.path == "items")
            .unwrap_or_else(|| panic!("missing diff child for items: {diff:#?}"));

        assert_eq!(items.status, DiffStatus::Modified);

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
    fn keeps_removed_array_items_explorable_when_array_field_was_absent_on_one_side() {
        let left = MetadataLoadResult::Parsed(json!({
            "items": [
                {"name": "alpha"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({}));

        let diff = compare_with_defaults(&left, &right);
        let items = diff
            .children
            .iter()
            .find(|node| node.path == "items")
            .unwrap_or_else(|| panic!("missing diff child for items: {diff:#?}"));

        assert_eq!(items.status, DiffStatus::Modified);

        let index_node = items
            .children
            .iter()
            .find(|node| node.path == "items[0]")
            .unwrap_or_else(|| panic!("missing diff child for items[0]: {items:#?}"));

        assert_eq!(index_node.status, DiffStatus::Removed);
    }

    #[test]
    fn treats_absent_object_field_as_equal_to_empty_container_on_other_side() {
        let added = compare_with_defaults(
            &MetadataLoadResult::Parsed(json!({})),
            &MetadataLoadResult::Parsed(json!({"emptyObject": {}, "emptyArray": []})),
        );
        let removed = compare_with_defaults(
            &MetadataLoadResult::Parsed(json!({"emptyObject": {}, "emptyArray": []})),
            &MetadataLoadResult::Parsed(json!({})),
        );

        for (case_name, diff) in [("added", &added), ("removed", &removed)] {
            let object = diff
                .children
                .iter()
                .find(|node| node.path == "emptyObject")
                .unwrap_or_else(|| panic!("[{case_name}] missing emptyObject child: {diff:#?}"));
            let array = diff
                .children
                .iter()
                .find(|node| node.path == "emptyArray")
                .unwrap_or_else(|| panic!("[{case_name}] missing emptyArray child: {diff:#?}"));

            assert_eq!(
                object.status,
                DiffStatus::Unchanged,
                "[{case_name}] empty object paired with absence should be Unchanged"
            );
            assert_eq!(
                array.status,
                DiffStatus::Unchanged,
                "[{case_name}] empty array paired with absence should be Unchanged"
            );
        }
    }

    #[test]
    fn uses_bracketed_paths_for_array_indexes() {
        let left = MetadataLoadResult::Parsed(json!({
            "items": ["left"]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "items": ["right"]
        }));

        let diff = compare_with_defaults(&left, &right);
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

        let diff = compare_with_defaults(&left, &right);

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
    fn matches_lines_by_line_name_regardless_of_position() {
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

        let diff = compare_with_defaults(&left, &right);
        let lines = diff
            .children
            .iter()
            .find(|node| node.path == "Lines")
            .unwrap_or_else(|| panic!("missing diff child for Lines: {diff:#?}"));

        assert!(
            lines.children.iter().any(|node| {
                node.path == "Lines[B932]" && node.status == DiffStatus::Unchanged
            }),
            "B932 should match by key and be Unchanged: {lines:#?}"
        );
        assert!(
            lines.children.iter().any(|node| {
                node.path == "Lines[M375]" && node.status == DiffStatus::Modified
            }),
            "M375 should match by key and be Modified (PriceDescription differs): {lines:#?}"
        );
        assert!(
            lines
                .children
                .iter()
                .all(|node| node.status != DiffStatus::Reordered),
            "no reorder node should be emitted (algorithm no longer surfaces position changes for Lines): {lines:#?}"
        );
    }

    #[test]
    fn matches_lines_by_line_name_when_direction_differs() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{"LineName": "M208", "Direction": "旧终点站", "PriceDescription": "2"}]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{"LineName": "M208", "Direction": "新终点站", "PriceDescription": "2"}]
        }));

        let diff = compare_with_defaults(&left, &right);
        let lines = diff
            .children
            .iter()
            .find(|node| node.path == "Lines")
            .unwrap_or_else(|| panic!("missing diff child for Lines: {diff:#?}"));

        let item = lines
            .children
            .iter()
            .find(|node| node.path == "Lines[M208]")
            .unwrap_or_else(|| {
                panic!("两侧 M208 应按线路名匹配为同一项 Lines[M208]: {lines:#?}")
            });
        assert_eq!(item.status, DiffStatus::Modified);

        let direction = item
            .children
            .iter()
            .find(|node| node.path == "Lines[M208].Direction")
            .unwrap_or_else(|| panic!("missing Direction child: {item:#?}"));
        assert_eq!(
            direction.status,
            DiffStatus::Modified,
            "方向变化应表现为字段差异而非整项增删: {direction:#?}"
        );

        assert!(
            lines.children.iter().all(|node| {
                node.status != DiffStatus::Added && node.status != DiffStatus::Removed
            }),
            "不应再出现 仅左/仅右 整项: {lines:#?}"
        );
    }

    #[test]
    fn compares_suffix_named_arrays_positionally_instead_of_as_keyed_business_arrays() {
        let left = MetadataLoadResult::Parsed(json!({
            "HistoricalLines": [
                {"LineName": "B932", "Direction": "Terminal", "PriceDescription": "1"},
                {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "2"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "HistoricalLines": [
                {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "2"},
                {"LineName": "B932", "Direction": "Terminal", "PriceDescription": "1"}
            ]
        }));

        let diff = compare_with_defaults(&left, &right);
        let historical_lines = diff
            .children
            .iter()
            .find(|node| node.path == "HistoricalLines")
            .unwrap_or_else(|| panic!("missing diff child for HistoricalLines: {diff:#?}"));

        assert!(
            historical_lines
                .children
                .iter()
                .all(|node| !matches!(node.status, DiffStatus::Reordered)),
            "unexpected keyed-array reorder in HistoricalLines: {historical_lines:#?}"
        );
        assert!(
            historical_lines
                .children
                .iter()
                .all(|node| node.path.starts_with("HistoricalLines[")),
            "expected positional child paths for HistoricalLines: {historical_lines:#?}"
        );
        assert!(
            historical_lines
                .children
                .iter()
                .all(|node| !node.path.contains("HistoricalLines[M375]")),
            "unexpected business-key path in HistoricalLines: {historical_lines:#?}"
        );
    }

    #[test]
    fn marks_added_route_stop_when_name_only_exists_on_right() {
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

        let diff = compare_with_defaults(&left, &right);
        let changes = flatten_changes(&diff);

        assert!(
            changes.iter().any(|node| {
                node.path.contains("RouteStops[CurrentStop]") && node.status == DiffStatus::Added
            }),
            "expected added keyed route stop in flattened changes: {changes:#?}"
        );
    }

    #[test]
    fn route_stop_sequence_shift_is_suppressed_in_diff() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [
                    {"Sequence": 1, "Name": "Alpha"},
                    {"Sequence": 2, "Name": "Beta"},
                    {"Sequence": 3, "Name": "Gamma"}
                ]
            }]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [
                    {"Sequence": 1, "Name": "Alpha"},
                    {"Sequence": 2, "Name": "NewStop"},
                    {"Sequence": 3, "Name": "Beta"},
                    {"Sequence": 4, "Name": "Gamma"}
                ]
            }]
        }));

        let diff = compare_with_defaults(&left, &right);
        let changes = flatten_changes(&diff);

        assert!(
            changes.iter().any(|n| {
                n.path == "Lines[B932].RouteStops[NewStop]"
                    && n.status == DiffStatus::Added
            }),
            "NewStop should be Added: {changes:#?}"
        );
        assert!(
            changes
                .iter()
                .all(|n| !n.path.ends_with(".Sequence")),
            "Sequence diffs must be suppressed everywhere: {changes:#?}"
        );
        assert!(
            changes
                .iter()
                .all(|n| n.path != "Lines[B932].RouteStops[Alpha]"
                    && n.path != "Lines[B932].RouteStops[Beta]"
                    && n.path != "Lines[B932].RouteStops[Gamma]"),
            "Alpha/Beta/Gamma differ only by Sequence — none should appear in the change list: {changes:#?}"
        );
    }

    #[test]
    fn duplicate_keys_match_consumptively_with_occurrence_suffix() {
        let left = MetadataLoadResult::Parsed(json!({
            "GroupItems": [
                {"SequenceNo": "①", "LineNames": "B932"},
                {"SequenceNo": "①", "LineNames": "M375"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({"GroupItems": []}));

        let diff = compare_with_defaults(&left, &right);
        let group_items = diff
            .children
            .iter()
            .find(|node| node.path == "GroupItems")
            .unwrap_or_else(|| panic!("missing diff child for GroupItems: {diff:#?}"));

        assert!(
            group_items
                .children
                .iter()
                .all(|node| node.status != DiffStatus::Error),
            "duplicates should no longer raise an error: {group_items:#?}"
        );
        assert!(
            group_items.children.iter().any(|n| {
                n.path == "GroupItems[①#1]" && n.status == DiffStatus::Removed
            }),
            "first occurrence should be Removed with #1 suffix: {group_items:#?}"
        );
        assert!(
            group_items.children.iter().any(|n| {
                n.path == "GroupItems[①#2]" && n.status == DiffStatus::Removed
            }),
            "second occurrence should be Removed with #2 suffix: {group_items:#?}"
        );
    }

    #[test]
    fn duplicate_keys_pair_in_occurrence_order() {
        let left = MetadataLoadResult::Parsed(json!({
            "GroupItems": [
                {"SequenceNo": "①", "LineNames": "B932"},
                {"SequenceNo": "①", "LineNames": "M375"},
                {"SequenceNo": "②", "LineNames": "M197"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "GroupItems": [
                {"SequenceNo": "①", "LineNames": "B932"},
                {"SequenceNo": "②", "LineNames": "M198"}
            ]
        }));

        let diff = compare_with_defaults(&left, &right);
        let group_items = diff
            .children
            .iter()
            .find(|node| node.path == "GroupItems")
            .unwrap_or_else(|| panic!("missing diff child for GroupItems: {diff:#?}"));

        assert!(
            group_items.children.iter().any(|n| {
                n.path == "GroupItems[①#1]" && n.status == DiffStatus::Unchanged
            }),
            "①#1 (B932 vs B932) should be Unchanged: {group_items:#?}"
        );
        assert!(
            group_items.children.iter().any(|n| {
                n.path == "GroupItems[①#2]" && n.status == DiffStatus::Removed
            }),
            "①#2 (M375 only on left) should be Removed: {group_items:#?}"
        );
        assert!(
            group_items.children.iter().any(|n| {
                n.path == "GroupItems[②]" && n.status == DiffStatus::Modified
            }),
            "② is unique on each side and should be Modified (M197 -> M198) without #N suffix: {group_items:#?}"
        );
    }

    #[test]
    fn creates_error_for_missing_business_key_without_falling_back_to_position() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [
                {"Direction": "Terminal", "PriceDescription": "1"},
                {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "2"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [
                {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "3"}
            ]
        }));

        let diff = compare_with_defaults(&left, &right);
        let lines = diff
            .children
            .iter()
            .find(|node| node.path == "Lines")
            .unwrap_or_else(|| panic!("missing diff child for Lines: {diff:#?}"));

        assert!(
            lines.children.iter().any(|node| {
                node.status == DiffStatus::Error && node.summary.contains("missing business key")
            }),
            "expected explicit error for missing business key: {lines:#?}"
        );
        assert!(
            lines.children.iter().any(|node| {
                node.path.contains("Lines[M375]") && node.status == DiffStatus::Modified
            }),
            "expected valid keyed line to still compare by key: {lines:#?}"
        );
        assert!(
            lines
                .children
                .iter()
                .all(|node| !node.path.contains("Lines[0]")),
            "unexpected positional fallback in keyed array: {lines:#?}"
        );
    }

    #[test]
    fn flatten_changes_removes_child_subtrees_from_returned_nodes() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [
                {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "2"}
            ]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [
                {"LineName": "M375", "Direction": "Downtown", "PriceDescription": "3"}
            ]
        }));

        let diff = compare_with_defaults(&left, &right);
        let changes = flatten_changes(&diff);

        assert!(
            changes.iter().all(|node| node.children.is_empty()),
            "flattened changes should not retain child subtrees: {changes:#?}"
        );
        assert!(
            changes
                .iter()
                .any(|node| node.path == "Lines[M375].PriceDescription"),
            "expected leaf change in flattened list: {changes:#?}"
        );
    }

    #[test]
    fn schema_known_field_is_visible_even_when_absent_on_both_sides() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{"Sequence": 1, "Name": "Origin", "RoadName": "Main"}]
            }]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{"Sequence": 1, "Name": "Origin", "RoadName": "Main"}]
            }]
        }));

        let diff = compare_with_defaults(&left, &right);
        let route_stop = locate(
            &diff,
            "Lines[B932].RouteStops[Origin]",
        );

        let building_type = route_stop
            .children
            .iter()
            .find(|node| node.path.ends_with(".BuildingType"))
            .unwrap_or_else(|| {
                panic!("schema-known BuildingType missing from route stop: {route_stop:#?}")
            });

        assert_eq!(building_type.status, DiffStatus::Unchanged);
        assert_eq!(building_type.left_value.as_deref(), Some("null"));
        assert_eq!(building_type.right_value.as_deref(), Some("null"));
    }

    #[test]
    fn missing_schema_field_equals_explicit_null() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{"Sequence": 1, "Name": "Origin", "BuildingType": null}]
            }]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{"Sequence": 1, "Name": "Origin"}]
            }]
        }));

        let diff = compare_with_defaults(&left, &right);
        let route_stop = locate(
            &diff,
            "Lines[B932].RouteStops[Origin]",
        );

        let building_type = route_stop
            .children
            .iter()
            .find(|node| node.path.ends_with(".BuildingType"))
            .unwrap_or_else(|| panic!("BuildingType missing: {route_stop:#?}"));

        assert_eq!(
            building_type.status,
            DiffStatus::Unchanged,
            "absent on right vs explicit null on left should be Unchanged: {building_type:#?}"
        );
    }

    #[test]
    fn missing_schema_field_vs_value_is_modified_with_null_placeholder() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{"Sequence": 1, "Name": "Origin"}]
            }]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{"Sequence": 1, "Name": "Origin", "BuildingType": "地铁"}]
            }]
        }));

        let diff = compare_with_defaults(&left, &right);
        let route_stop = locate(
            &diff,
            "Lines[B932].RouteStops[Origin]",
        );

        let building_type = route_stop
            .children
            .iter()
            .find(|node| node.path.ends_with(".BuildingType"))
            .unwrap_or_else(|| panic!("BuildingType missing: {route_stop:#?}"));

        assert_eq!(building_type.status, DiffStatus::Modified);
        assert_eq!(building_type.left_value.as_deref(), Some("null"));
        assert_eq!(building_type.right_value.as_deref(), Some("\"地铁\""));
    }

    fn locate<'a>(root: &'a super::DiffNode, path: &str) -> &'a super::DiffNode {
        fn walk<'a>(
            node: &'a super::DiffNode,
            target: &str,
        ) -> Option<&'a super::DiffNode> {
            if node.path == target {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = walk(child, target) {
                    return Some(found);
                }
            }
            None
        }
        walk(root, path).unwrap_or_else(|| panic!("path {path} not found in diff: {root:#?}"))
    }

    #[test]
    fn keeps_error_node_visible_in_flattened_results() {
        let left = MetadataLoadResult::Error(CompareError::MissingStopPlateMetadata);
        let right = MetadataLoadResult::Parsed(json!({"StopName": "A"}));

        let diff = compare_with_defaults(&left, &right);
        let changes = flatten_changes(&diff);
        let error_node = changes
            .iter()
            .find(|node| node.path == "StopPlateMetadata")
            .unwrap_or_else(|| panic!("expected flattened root error node: {changes:#?}"));

        assert_eq!(error_node.status, DiffStatus::Error);
        assert!(
            error_node
                .left_value
                .as_deref()
                .is_some_and(|value| value.contains("missing StopPlate metadata")),
            "expected left error details to stay attached to flattened error node: {error_node:#?}"
        );
        assert!(error_node.children.is_empty());
    }

    #[test]
    fn default_equivalence_map_collapses_metro_and_hospital_aliases() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [
                    {"Sequence": 1, "Name": "Alpha", "BuildingType": "Metro"},
                    {"Sequence": 2, "Name": "Beta",  "BuildingType": "Hospital"}
                ]
            }]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [
                    {"Sequence": 1, "Name": "Alpha", "BuildingType": "地铁站"},
                    {"Sequence": 2, "Name": "Beta",  "BuildingType": "医院"}
                ]
            }]
        }));

        let diff = compare_with_defaults(&left, &right);
        let changes = flatten_changes(&diff);
        assert!(
            changes.is_empty(),
            "Metro↔地铁站 / Hospital↔医院 should collapse to Unchanged: {changes:#?}"
        );
    }

    #[test]
    fn unmapped_alias_still_modifies() {
        // "School" is not in the pinned default equivalence map → real diff
        // against "学校". (Don't use a real-world alias like "Airport" here:
        // users add those to compare-config.json over time.)
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932", "Direction": "Terminal",
                "RouteStops": [{"Sequence": 1, "Name": "X", "BuildingType": "School"}]
            }]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "B932", "Direction": "Terminal",
                "RouteStops": [{"Sequence": 1, "Name": "X", "BuildingType": "学校"}]
            }]
        }));
        let diff = compare_with_defaults(&left, &right);
        let changes = flatten_changes(&diff);
        assert!(
            changes.iter().any(|n| {
                n.path == "Lines[B932].RouteStops[X].BuildingType"
                    && n.status == DiffStatus::Modified
            }),
            "unmapped alias must still surface as Modified: {changes:#?}"
        );
    }

    #[test]
    fn empty_string_null_and_missing_field_are_all_equivalent() {
        let cases = [
            (json!({"a": ""}), json!({"a": null})),
            (json!({"a": ""}), json!({})),
            (json!({"a": null}), json!({})),
            (json!({"a": ""}), json!({"a": ""})),
            (json!({}), json!({})),
        ];

        for (i, (left, right)) in cases.iter().enumerate() {
            let diff = compare_with_defaults(
                &MetadataLoadResult::Parsed(left.clone()),
                &MetadataLoadResult::Parsed(right.clone()),
            );
            let changes = flatten_changes(&diff);
            assert!(
                changes.iter().all(|n| n.status == DiffStatus::Unchanged),
                "case {i}: expected no changes for {left:?} vs {right:?}, got: {changes:#?}"
            );
        }
    }

    #[test]
    fn loop_route_with_repeated_stop_names_compares_consumptively() {
        let left = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "M103",
                "Direction": "下沙总站",
                "RouteStops": [
                    {"Sequence": 1, "Name": "下沙总站"},
                    {"Sequence": 5, "Name": "田面"},
                    {"Sequence": 12, "Name": "福田水围村"},
                    {"Sequence": 20, "Name": "福田水围村"},
                    {"Sequence": 27, "Name": "田面"},
                    {"Sequence": 32, "Name": "下沙总站"}
                ]
            }]
        }));
        let right = MetadataLoadResult::Parsed(json!({
            "Lines": [{
                "LineName": "M103",
                "Direction": "下沙总站",
                "RouteStops": [
                    {"Sequence": 1, "Name": "下沙总站"},
                    {"Sequence": 5, "Name": "田面"},
                    {"Sequence": 13, "Name": "福田水围村"},
                    {"Sequence": 21, "Name": "福田水围村"},
                    {"Sequence": 28, "Name": "田面"},
                    {"Sequence": 33, "Name": "下沙总站"}
                ]
            }]
        }));

        let diff = compare_with_defaults(&left, &right);
        let changes = flatten_changes(&diff);

        assert!(
            changes.iter().all(|n| n.status != DiffStatus::Error),
            "duplicate Names in a loop route must not produce errors: {changes:#?}"
        );
        assert!(
            changes.iter().all(|n| !n.path.ends_with(".Sequence")),
            "Sequence diffs are suppressed; loop-route shift should not surface: {changes:#?}"
        );
        // Bodies otherwise identical; with Sequence suppressed nothing should remain.
        assert!(
            changes.is_empty(),
            "loop route with only Sequence shifts should produce zero diffs: {changes:#?}"
        );
    }

    /// Config with a normal alias (Metro) plus a wildcard (Bus) on BuildingType.
    fn config_with_bus_wildcard() -> CompareConfig {
        serde_json::from_value(json!({
            "equivalence_maps": {
                "Lines[*].RouteStops[*].BuildingType": { "Metro": "地铁站", "Bus": "*" }
            }
        }))
        .unwrap()
    }

    fn one_stop_line(building_type: serde_json::Value) -> serde_json::Value {
        // building_type is spliced verbatim; pass json!(null) to omit-as-null,
        // or json!("...") for a concrete value.
        json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{ "Name": "X", "BuildingType": building_type }]
            }]
        })
    }

    #[test]
    fn wildcard_value_matches_any_concrete_value_on_other_side() {
        let cfg = config_with_bus_wildcard();
        let left = MetadataLoadResult::Parsed(one_stop_line(json!("Bus")));
        let right = MetadataLoadResult::Parsed(one_stop_line(json!("地铁站")));

        let diff = compare_metadata_with_config(&cfg, &left, &right);
        let changes = flatten_changes(&diff);
        assert!(
            changes.is_empty(),
            "Bus 通配应与任意值等价（Bus vs 地铁站）: {changes:#?}"
        );
    }

    #[test]
    fn wildcard_value_matches_null_and_missing_both_directions() {
        let cfg = config_with_bus_wildcard();

        // Bus (left) vs explicit null (right)
        let a = compare_metadata_with_config(
            &cfg,
            &MetadataLoadResult::Parsed(one_stop_line(json!("Bus"))),
            &MetadataLoadResult::Parsed(one_stop_line(json!(null))),
        );
        assert!(
            flatten_changes(&a).is_empty(),
            "Bus vs null 应 Unchanged: {:#?}",
            flatten_changes(&a)
        );

        // Bus (left) vs missing BuildingType (right) — schema fills null
        let right_missing = json!({
            "Lines": [{
                "LineName": "B932",
                "Direction": "Terminal",
                "RouteStops": [{ "Name": "X" }]
            }]
        });
        let b = compare_metadata_with_config(
            &cfg,
            &MetadataLoadResult::Parsed(one_stop_line(json!("Bus"))),
            &MetadataLoadResult::Parsed(right_missing.clone()),
        );
        assert!(
            flatten_changes(&b).is_empty(),
            "Bus vs 缺失 应 Unchanged: {:#?}",
            flatten_changes(&b)
        );

        // missing (left) vs Bus (right) — reverse direction
        let c = compare_metadata_with_config(
            &cfg,
            &MetadataLoadResult::Parsed(right_missing),
            &MetadataLoadResult::Parsed(one_stop_line(json!("Bus"))),
        );
        assert!(
            flatten_changes(&c).is_empty(),
            "缺失 vs Bus 应 Unchanged（对称）: {:#?}",
            flatten_changes(&c)
        );
    }

    #[test]
    fn normal_alias_still_collapses_alongside_wildcard() {
        let cfg = config_with_bus_wildcard();
        let left = MetadataLoadResult::Parsed(one_stop_line(json!("Metro")));
        let right = MetadataLoadResult::Parsed(one_stop_line(json!("地铁站")));

        let diff = compare_metadata_with_config(&cfg, &left, &right);
        assert!(
            flatten_changes(&diff).is_empty(),
            "Metro↔地铁站 普通别名应仍折叠: {:#?}",
            flatten_changes(&diff)
        );
    }

    #[test]
    fn non_wildcard_mismatch_still_modified() {
        // Park is in neither the wildcard nor the alias map → real diff.
        let cfg = config_with_bus_wildcard();
        let left = MetadataLoadResult::Parsed(one_stop_line(json!("Park")));
        let right = MetadataLoadResult::Parsed(one_stop_line(json!("公园")));

        let diff = compare_metadata_with_config(&cfg, &left, &right);
        let node = locate(&diff, "Lines[B932].RouteStops[X].BuildingType");
        assert_eq!(
            node.status,
            DiffStatus::Modified,
            "非通配 mismatch 必须仍为 Modified（守卫：通配不吞真实差异）"
        );
    }

    #[test]
    fn wildcard_value_matches_non_string_scalar_on_other_side() {
        let cfg = config_with_bus_wildcard();
        // Bus (wildcard) on the left vs a numeric BuildingType on the right.
        let left = MetadataLoadResult::Parsed(one_stop_line(json!("Bus")));
        let right = MetadataLoadResult::Parsed(one_stop_line(json!(42)));

        let diff = compare_metadata_with_config(&cfg, &left, &right);
        assert!(
            flatten_changes(&diff).is_empty(),
            "Bus 通配应匹配非字符串标量（Bus vs 42）: {:#?}",
            flatten_changes(&diff)
        );
    }
}
