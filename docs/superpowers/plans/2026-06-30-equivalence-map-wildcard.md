# Equivalence-Map Wildcard (`"*"`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make an `equivalence_maps` entry whose target is `"*"` (e.g. `"Bus": "*"`) behave as a wildcard — on either side it matches any value on the other side, including null/missing.

**Architecture:** Add a `CompareConfig::is_wildcard(path, value)` predicate in `src/config.rs` that returns true when the value's equivalence-map target equals the `"*"` sentinel. Short-circuit `scalars_are_equivalent_at` in `src/diff.rs` to return `true` when either side's string value is a wildcard. No new config fields, no struct changes, no change to the code-side `CompareConfig::default()` fallback.

**Tech Stack:** Rust, `serde_json::Value`, std `HashMap`. Library crate `png_metadata_compare` (`src/lib.rs`); unit tests live in `#[cfg(test)] mod tests` inside each module and run via `cargo test --lib`.

## Global Constraints

- Reuse `equivalence_maps`; the wildcard sentinel is the literal string `"*"`. No new config keys, no `struct` field changes.
- Do **not** modify `CompareConfig::default()` (config.rs:18) — wildcard activates only via the user's `compare-config.json`.
- Wildcard is **symmetric** (left or right) and matches **any** opposite value, **including null / missing / empty string / non-string scalars**.
- Wildcard applies only inside scalar equivalence (`scalars_are_equivalent_at`, i.e. `(Some, Some)` leaves). It must **not** resurrect an entirely added/removed parent object.
- Commit message footer (every commit):
  `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`

---

### Task 1: `CompareConfig::is_wildcard` predicate

**Files:**
- Modify: `src/config.rs` (add module const + method; update doc comment on `CompareConfig`)
- Test: `src/config.rs` (`#[cfg(test)] mod tests`, already present)

**Interfaces:**
- Consumes: existing `pub equivalence_maps: HashMap<String, HashMap<String, String>>` field.
- Produces: `pub fn is_wildcard(&self, normalized_path: &str, value: &str) -> bool` — true iff `equivalence_maps[normalized_path][value] == "*"`. Also a module-level `const WILDCARD_SENTINEL: &str = "*";`.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `src/config.rs` (after the existing `missing_optional_fields_fall_back_to_empty` test, before the closing `}` of `mod tests`). The module already has `use super::CompareConfig;` and `use serde_json::json;`.

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib is_wildcard_true_only_for_star_target`
Expected: FAIL — compile error `no method named `is_wildcard` found for reference `&CompareConfig``.

- [ ] **Step 3: Write minimal implementation**

In `src/config.rs`, add the module-level constant immediately before `impl CompareConfig {` (currently line 39):

```rust
/// `equivalence_maps` target meaning "match any value on the other side".
const WILDCARD_SENTINEL: &str = "*";
```

Then add the method inside `impl CompareConfig`, immediately after the existing `canonicalize` method (after its closing `}`, currently line 53):

```rust
    /// Whether `value` at `normalized_path` is a wildcard — i.e. its
    /// equivalence-map target is `"*"`. A wildcard value is equivalent to any
    /// value on the other side (including null / a missing field).
    pub fn is_wildcard(&self, normalized_path: &str, value: &str) -> bool {
        self.equivalence_maps
            .get(normalized_path)
            .and_then(|m| m.get(value))
            .is_some_and(|target| target == WILDCARD_SENTINEL)
    }
```

Also update the `CompareConfig` doc comment (currently lines 6-9) by appending one line after the `equivalence_maps` bullet, so the final block reads:

```rust
/// 用户可调的比对配置——目前覆盖两类规则：
/// 1. `ignored_fields`：路径用 `[*]` 通配（同 schema），命中即跳过比对
/// 2. `equivalence_maps`：字段值的别名表，比对前把 key 折叠成 value（大小写敏感）
///    例：`{"Metro": "地铁站"}` 会把左侧 "Metro" 与右侧 "地铁站" 视为等价
///    特例：value 为 `"*"` 时该 key 视为通配值——出现在任意一边即与对侧任何值
///    等价（含 null / 缺失）。例：`{"Bus": "*"}`。
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib is_wildcard_true_only_for_star_target`
Expected: PASS (`test result: ok. 1 passed`).

- [ ] **Step 5: Commit**

```bash
git add src/config.rs
git commit -m "$(cat <<'EOF'
feat(config): add is_wildcard predicate for "*" sentinel

equivalence_maps target "*" marks a value as a wildcard. is_wildcard
reports it; canonicalize is unchanged.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Wildcard short-circuit in `scalars_are_equivalent_at`

**Files:**
- Modify: `src/diff.rs:370-379` (`scalars_are_equivalent_at`; update its doc comment at 366-369)
- Test: `src/diff.rs` (`#[cfg(test)] mod tests`, already present)

**Interfaces:**
- Consumes: `CompareConfig::is_wildcard` (Task 1); existing `compare_metadata_with_config`, `flatten_changes`, and the test-local `locate` helper.
- Produces: no new public API; behavior change only.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `src/diff.rs` (before the final closing `}` of `mod tests`, after `loop_route_with_repeated_stop_names_compares_consumptively`). The module already imports `compare_metadata_with_config`, `flatten_changes`, `DiffStatus`, `CompareConfig`, `MetadataLoadResult`, and `json!`. It also defines a `locate(root, path)` helper.

```rust
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
        // School is in neither the wildcard nor the alias map → real diff.
        let cfg = config_with_bus_wildcard();
        let left = MetadataLoadResult::Parsed(one_stop_line(json!("School")));
        let right = MetadataLoadResult::Parsed(one_stop_line(json!("学校")));

        let diff = compare_metadata_with_config(&cfg, &left, &right);
        let node = locate(&diff, "Lines[B932].RouteStops[X].BuildingType");
        assert_eq!(
            node.status,
            DiffStatus::Modified,
            "非通配 mismatch 必须仍为 Modified（守卫：通配不吞真实差异）"
        );
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib wildcard_value_matches_any_concrete_value_on_other_side wildcard_value_matches_null_and_missing_both_directions`
Expected: FAIL — assertions fire because `Bus` vs `地铁站`/null currently canonicalizes `Bus`→`"*"` and reports `Modified` (`changes` non-empty). `normal_alias_still_collapses_alongside_wildcard` and `non_wildcard_mismatch_still_modified` already pass; the two wildcard tests are the red ones.

- [ ] **Step 3: Write minimal implementation**

In `src/diff.rs`, replace the body of `scalars_are_equivalent_at` (currently lines 370-379) so the wildcard short-circuit runs after the blank-vs-blank check and before the string-folding compare:

```rust
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
```

Also update the function's doc comment (currently lines 366-369) by appending one sentence:

```rust
/// Equivalence with optional per-path canonicalisation from `compare-config.json`.
/// String values matching an entry in the equivalence map are folded to their canonical
/// form before comparison; other types fall back to literal equality + blank-value
/// equivalence (null / "" / [] are interchangeable).
/// A value whose equivalence-map target is `"*"` is a wildcard: if it appears on either
/// side, the scalar matches any value on the other side, including null.
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib wildcard_value_matches_any_concrete_value_on_other_side wildcard_value_matches_null_and_missing_both_directions normal_alias_still_collapses_alongside_wildcard non_wildcard_mismatch_still_modified`
Expected: PASS (4 passed).

- [ ] **Step 5: Run the full library test suite (no regressions)**

Run: `cargo test --lib`
Expected: PASS — all pre-existing `config::tests` and `diff::tests` still pass (the `#[ignore]`d `dump_real_meta_diff` stays ignored).

- [ ] **Step 6: Commit**

```bash
git add src/diff.rs
git commit -m "$(cat <<'EOF'
feat(diff): "*" equivalence-map values match anything, incl null

scalars_are_equivalent_at short-circuits to equal when either side's
string value is a wildcard (is_wildcard). Symmetric; covers null and
missing (schema-filled) fields. Closes the gap where "Bus":"*" folded
to the literal "*" and reported spurious Modified diffs.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review

**Spec coverage:**
- G1 (target `"*"` → wildcard): Task 1 `is_wildcard`.
- G2 (symmetric): Task 2 checks `left` and `right` independently; test `wildcard_value_matches_null_and_missing_both_directions` covers both directions.
- G3 (matches any incl null/missing/non-string): Task 2 short-circuit precedes the string-only fold and returns before the `left == right` fallback; null/missing covered by tests, non-string scalars covered by the same `return true` path (any opposite `Value`).
- G4 (no new config field / struct change): Tasks touch only `config.rs` (const + method) and `diff.rs` (function body). No struct edits.
- Non-goal "don't modify `Default`": no task edits `CompareConfig::default()`.
- Non-goal "don't resurrect added/removed parent": unchanged — short-circuit lives only in `scalars_are_equivalent_at`, reached only on `(Some, Some)` leaves; no test forces otherwise and no code path outside that function is touched.

**Placeholder scan:** No TBD/TODO/"handle edge cases"; every code step shows full code and exact `cargo test` commands with expected outcomes.

**Type consistency:** `is_wildcard(&self, normalized_path: &str, value: &str) -> bool` defined in Task 1, consumed identically in Task 2. `WILDCARD_SENTINEL: &str` used only within `is_wildcard`. Test helpers `config_with_bus_wildcard`/`one_stop_line` defined and used within Task 2. `locate` is the pre-existing diff-test helper (diff.rs:1346).
