use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

/// 用户可调的比对配置——目前覆盖两类规则：
/// 1. `ignored_fields`：路径用 `[*]` 通配（同 schema），命中即跳过比对
/// 2. `equivalence_maps`：字段值的别名表，比对前把 key 折叠成 value（大小写敏感）
///    例：`{"Metro": "地铁站"}` 会把左侧 "Metro" 与右侧 "地铁站" 视为等价
///    特例：value 为 `"*"` 时该 key 视为通配值——出现在任意一边即与对侧任何值
///    等价（含 null / 缺失）。例：`{"Bus": "*"}`。
#[derive(Debug, Clone, Deserialize)]
pub struct CompareConfig {
    #[serde(default)]
    pub ignored_fields: Vec<String>,
    #[serde(default)]
    pub equivalence_maps: HashMap<String, HashMap<String, String>>,
}

impl Default for CompareConfig {
    fn default() -> Self {
        let mut maps: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut bt: HashMap<String, String> = HashMap::new();
        bt.insert("Metro".to_string(), "地铁站".to_string());
        bt.insert("Hospital".to_string(), "医院".to_string());
        maps.insert("Lines[*].RouteStops[*].BuildingType".to_string(), bt);

        Self {
            ignored_fields: vec![
                "Lines[*].RouteStops[*].Sequence".to_string(),
                "RenderTime".to_string(),
                "QRCode".to_string(),
                "StopId".to_string(),
                "Lines[*].LinePattern".to_string(),
                "GroupItems[*].Distance".to_string(),
            ],
            equivalence_maps: maps,
        }
    }
}

/// `equivalence_maps` target meaning "match any value on the other side".
const WILDCARD_SENTINEL: &str = "*";

impl CompareConfig {
    pub fn is_ignored(&self, normalized_path: &str) -> bool {
        self.ignored_fields
            .iter()
            .any(|p| p.as_str() == normalized_path)
    }

    /// Look up a string value's canonical equivalent for a given normalized path.
    /// Returns the original if there's no rule.
    pub fn canonicalize<'a>(&'a self, normalized_path: &str, value: &'a str) -> &'a str {
        self.equivalence_maps
            .get(normalized_path)
            .and_then(|m| m.get(value).map(String::as_str))
            .unwrap_or(value)
    }

    /// Whether `value` at `normalized_path` is a wildcard — i.e. its
    /// equivalence-map target is `"*"`. A wildcard value is equivalent to any
    /// value on the other side (including null / a missing field).
    pub fn is_wildcard(&self, normalized_path: &str, value: &str) -> bool {
        self.equivalence_maps
            .get(normalized_path)
            .and_then(|m| m.get(value))
            .is_some_and(|target| target == WILDCARD_SENTINEL)
    }
}

static CONFIG: OnceLock<CompareConfig> = OnceLock::new();

pub fn config() -> &'static CompareConfig {
    CONFIG.get_or_init(load_or_default)
}

const CANDIDATE_PATHS: &[&str] = &["compare-config.json", "config/compare-config.json"];

fn load_or_default() -> CompareConfig {
    for candidate in CANDIDATE_PATHS {
        match try_load(Path::new(candidate)) {
            Ok(Some(cfg)) => return cfg,
            Ok(None) => {} // file absent — try next
            Err(err) => eprintln!("compare-config.json at {candidate}: {err}"),
        }
    }
    CompareConfig::default()
}

fn try_load(path: &Path) -> Result<Option<CompareConfig>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.to_string()),
    };
    let stripped = text.trim_start_matches('\u{feff}');
    serde_json::from_str(stripped)
        .map(Some)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::CompareConfig;
    use serde_json::json;

    #[test]
    fn default_config_ignores_route_stop_sequence() {
        let cfg = CompareConfig::default();
        assert!(cfg.is_ignored("Lines[*].RouteStops[*].Sequence"));
        assert!(cfg.is_ignored("RenderTime"));
        assert!(cfg.is_ignored("QRCode"));
        assert!(cfg.is_ignored("StopId"));
        assert!(cfg.is_ignored("Lines[*].LinePattern"));
        assert!(cfg.is_ignored("GroupItems[*].Distance"));
        assert!(!cfg.is_ignored("Lines[*].PriceDescription"));
    }

    #[test]
    fn default_config_canonicalizes_observed_building_type_aliases() {
        let cfg = CompareConfig::default();
        assert_eq!(
            cfg.canonicalize("Lines[*].RouteStops[*].BuildingType", "Metro"),
            "地铁站"
        );
        assert_eq!(
            cfg.canonicalize("Lines[*].RouteStops[*].BuildingType", "Hospital"),
            "医院"
        );
        // Already-canonical values pass through unchanged.
        assert_eq!(
            cfg.canonicalize("Lines[*].RouteStops[*].BuildingType", "地铁站"),
            "地铁站"
        );
        // Unknown values pass through unchanged.
        assert_eq!(
            cfg.canonicalize("Lines[*].RouteStops[*].BuildingType", "Airport"),
            "Airport"
        );
        // Case-sensitive: lowercase variant is not collapsed.
        assert_eq!(
            cfg.canonicalize("Lines[*].RouteStops[*].BuildingType", "metro"),
            "metro"
        );
    }

    #[test]
    fn deserializes_user_provided_overrides() {
        let raw = json!({
            "ignored_fields": ["Lines[*].PriceDescription"],
            "equivalence_maps": {
                "Lines[*].RouteStops[*].BuildingType": {
                    "Bus": "公交"
                }
            }
        });
        let cfg: CompareConfig = serde_json::from_value(raw).unwrap();
        assert!(cfg.is_ignored("Lines[*].PriceDescription"));
        assert!(!cfg.is_ignored("Lines[*].RouteStops[*].Sequence"));
        assert_eq!(
            cfg.canonicalize("Lines[*].RouteStops[*].BuildingType", "Bus"),
            "公交"
        );
    }

    #[test]
    fn missing_optional_fields_fall_back_to_empty() {
        let cfg: CompareConfig = serde_json::from_str("{}").unwrap();
        assert!(cfg.ignored_fields.is_empty());
        assert!(cfg.equivalence_maps.is_empty());
    }

    #[test]
    fn is_wildcard_true_only_for_star_target() {
        let cfg: CompareConfig = serde_json::from_value(json!({
            "equivalence_maps": {
                "Lines[*].RouteStops[*].BuildingType": { "Metro": "地铁站", "Bus": "*" }
            }
        }))
        .unwrap();
        let path = "Lines[*].RouteStops[*].BuildingType";

        // value mapped to "*" is a wildcard
        assert!(cfg.is_wildcard(path, "Bus"));
        // a normal alias target is not a wildcard
        assert!(!cfg.is_wildcard(path, "Metro"));
        // a value not present in the map is not a wildcard
        assert!(!cfg.is_wildcard(path, "Unknown"));
        // a path with no equivalence map is not a wildcard
        assert!(!cfg.is_wildcard("Some.Other.Path", "Bus"));
    }
}
