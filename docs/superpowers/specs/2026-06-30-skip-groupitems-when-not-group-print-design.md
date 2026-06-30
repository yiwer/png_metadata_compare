# 两边都非分站打印时跳过 GroupItems — 设计

日期：2026-06-30 · 分支：`master`

## 背景与问题

站牌元数据根对象有两个相邻字段：

- `IsGroupPrint`（bool）——是否包含分站信息。
- `GroupItems`（array）——分站信息列表。

当两侧都不是分站打印（`IsGroupPrint` 非 `true`）时，`GroupItems` 往往是无关的残留/陈旧数据。此时把 `GroupItems` 的差异报出来是噪音。用户希望：**当两边 `IsGroupPrint` 都不为 `true` 时，不比较 `GroupItems`**，并且这条规则**写进代码**（不走配置文件）。

现有 `ignored_fields` 是**无条件**忽略，无法表达"依赖另一字段值的条件忽略"，所以需要在比对逻辑里加一处条件剪枝。

## 目标

- **G1**：当两侧的 `IsGroupPrint` 都不为布尔 `true`（即 `false` / `null` / 缺失都算"非分站"）时，从 diff 中移除 `GroupItems`。
- **G2**：只要**任一侧** `IsGroupPrint == true`，就照常比较 `GroupItems`。
- **G3**：硬编码在代码中，不依赖 `compare-config.json`。
- **G4**：`IsGroupPrint` 字段本身仍照常参与比对，不受影响。

## 非目标（YAGNI）

- 不做成可配置项（用户明确要求写进代码）。
- 不改 `ignored_fields` 机制（它仍是无条件忽略）。
- 不在非根层级处理 `GroupItems`（该字段只存在于根对象）。
- 不保留"可见但强制未变更"的节点——按**整段移除**实现（与忽略字段一致的"不出现"效果）。

---

## 设计

### 位置

`src/diff.rs` 的对象-对象比对分支（当前 diff.rs:87-123，`(Some(Value::Object(left_map)), Some(Value::Object(right_map)))` 分支）。该分支已持有 `left_map`、`right_map`，并在 `keys.retain(|key| !is_ignored_field(...))` 处做无条件剪枝。条件剪枝紧随其后插入，仅在**根对象**生效。

根对象的判定用 `path.is_empty()`：根比对以空路径进入（`compare_values(cfg, "", ...)`），而 `IsGroupPrint`/`GroupItems` 都是根级字段。

### 判定 helper

```rust
fn is_group_print(map: &serde_json::Map<String, Value>) -> bool {
    matches!(map.get("IsGroupPrint"), Some(Value::Bool(true)))
}
```

仅当字段存在且为布尔 `true` 时返回 `true`；`false` / `null` / 缺失一律返回 `false`（满足 G1 的"非 true 即视为非分站"）。

### 剪枝逻辑

在 `keys.retain(...)` 之后追加：

```rust
// 两侧都不是分站打印时，GroupItems 是无关残留——不参与比对。
if path.is_empty() && !is_group_print(left_map) && !is_group_print(right_map) {
    keys.remove("GroupItems");
}
```

`keys` 是 `BTreeSet<String>`，`remove("GroupItems")` 在键不存在时是无害的 no-op。移除后该键不进入子节点递归，`GroupItems` 整段不出现在 diff 树中（与忽略字段同样的效果，只是带条件）。

### 行为表

| 左 IsGroupPrint | 右 IsGroupPrint | GroupItems 不同？ | 结果 |
|---|---|---|---|
| false | false | 是 | 不出现 GroupItems 差异 |
| 缺失 / null | 缺失 / null | 是 | 不出现 GroupItems 差异 |
| true | false | 是 | 照常出现 GroupItems 差异 |
| true | true | 是 | 照常出现 GroupItems 差异 |
| false | false | 是（且别的字段也不同） | 别的字段照常报，仅 GroupItems 被移除 |

`IsGroupPrint` 字段自身在所有情形下都照常比对（如一真一假会显示 Modified）。

### 作用边界

- 仅根对象（`path.is_empty()`）。其他层级不含 `GroupItems`，逻辑不触发。
- 仅影响 `GroupItems` 这一个键；其余键的剪枝（`ignored_fields`）不变。

---

## 测试（TDD，src/diff.rs 的 `#[cfg(test)] mod tests`）

用 `compare_with_defaults`（已有 helper，固定 `CompareConfig::default()`）构造根对象。

1. **both_false_skips_group_items**：两侧 `IsGroupPrint:false`，`GroupItems` 内容不同 → `flatten_changes` 中无任何路径以 `GroupItems` 开头。
2. **both_missing_is_group_print_skips_group_items**：两侧都不含 `IsGroupPrint`，`GroupItems` 不同 → 无 `GroupItems` 差异。
3. **one_side_group_print_true_still_compares**：左 `true`、右 `false`，`GroupItems` 不同 → 出现 `GroupItems` 差异节点。
4. **both_true_still_compares**：两侧 `true`，`GroupItems` 不同 → 出现 `GroupItems` 差异节点。
5. **skip_is_scoped_to_group_items**：两侧 `false`，`GroupItems` 不同**且**另一根级字段（如 `StopName`）也不同 → `StopName` 差异照常出现，`GroupItems` 不出现。

## 影响面与风险

- 改动局部化于 `src/diff.rs`（一个文件内的 helper + 一处条件）。不改配置 schema、不改 `ignored_fields` 机制、不改其他比对路径。
- 纯增量行为：仅在"两侧都非分站打印"这一新条件下移除 `GroupItems`；其余情形与现状完全一致，向后兼容。
- 代码改动 ⇒ 需重新编译才进二进制（运行时配置不涉及此规则）。
