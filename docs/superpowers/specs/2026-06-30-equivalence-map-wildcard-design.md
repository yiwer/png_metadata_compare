# 等价映射通配值（`"*"` 哨兵）— 设计

日期：2026-06-30 · 分支：`master`

## 背景与问题

`compare-config.json` 的 `equivalence_maps` 为某条路径上的字符串值提供别名折叠：比对前把 key 折叠成 value，再做相等判断。例如 `{"Metro": "地铁站"}` 让左侧 `"Metro"` 与右侧 `"地铁站"` 视为等价。

当前 `compare-config.json:22` 已写有：

```json
"Lines[*].RouteStops[*].BuildingType": {
  ...
  "Bus": "*"
}
```

但现有实现把 `"*"` 当作**普通折叠目标**：`canonicalize(path, "Bus")` 返回字面字符串 `"*"`，随后 `scalars_are_equivalent_at` 比较 `canonicalize(left) == canonicalize(right)`。于是 `Bus` 对 `公交车` / `地铁站` / `null` / 缺失 都判为 **Modified**，并非用户期望。

用户期望：当某 key（如 `Bus`）的映射目标是 `"*"` 时，把它解释为**通配值**——只要它出现在**任意一边**，该标量就与对侧**任何值**相等，**包括 `null` / 缺失 / 空串 / 非字符串**。

## 目标

- **G1**：`equivalence_maps[path]` 中映射目标为 `"*"` 的 key 被解释为通配值。
- **G2**：通配对称——无论该值在左侧还是右侧，都生效。
- **G3**：通配匹配对侧任何值，**含 `null` / 缺失 / 空串 / 非字符串标量**。
- **G4**：保留第 22 行的现有配置写法，不新增配置字段、不改 struct。

## 非目标（YAGNI）

- 不新增独立配置段（如 `wildcard_values`）；复用 `equivalence_maps` + `"*"` 哨兵。
- 不修改代码兜底 `CompareConfig::default()`（config.rs:18）——通配只在用户的 `compare-config.json` 文件里启用。无配置文件时兜底保持最小（仅 `Metro`/`Hospital`）。
- 通配**不**复活"整项增删"：若某 `RouteStop` 仅存在于一侧，整站仍是 Added/Removed（见"作用边界"）。
- 不支持把某值真正折叠成字面量 `"*"`（本领域不会出现该需求）。

---

## 设计

### 哨兵语义

约定：`equivalence_maps[path]` 中，若某 key 的映射目标恰为字符串 `"*"`，则该 key 是该路径上的**通配值**。判定**先于**别名折叠，因此不依赖 `canonicalize` 的返回。

### 改动 A：`src/config.rs`

新增哨兵常量与判定方法，`canonicalize` 保持不变：

```rust
const WILDCARD_SENTINEL: &str = "*";

impl CompareConfig {
    /// 该路径上 `value` 是否为通配值（映射目标为 `"*"`）。
    /// 通配值与对侧任何值等价，含 null / 缺失。
    pub fn is_wildcard(&self, normalized_path: &str, value: &str) -> bool {
        self.equivalence_maps
            .get(normalized_path)
            .and_then(|m| m.get(value))
            .is_some_and(|target| target == WILDCARD_SENTINEL)
    }
}
```

同时更新 `CompareConfig` 顶部文档注释，补充 `"*"` 哨兵的说明。

### 改动 B：`src/diff.rs` — `scalars_are_equivalent_at`

在 blank 短路之后、字符串折叠比较之前插入通配短路（两向对称）：

```rust
fn scalars_are_equivalent_at(cfg: &CompareConfig, path: &str, left: &Value, right: &Value) -> bool {
    if is_blank_value(left) && is_blank_value(right) {
        return true;
    }
    let normalized = normalize_path_for_schema(path);
    // 任一侧的字符串值映射到 "*" → 通配，匹配对侧任何值（含 null）
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

同时更新该函数的文档注释。

### 关键路径验证

`BuildingType` 是 `Lines[*].RouteStops[*]` 下的 schema-known 标量。`RouteStops` 按 `Name` 业务键配对，配对成功的两项以 `(Some(obj), Some(obj))` 进入对象比对；对象比对对一侧缺失的 key 用 `null`/`[]`/`{}` 占位（diff.rs:112-117）。因此 `BuildingType` 叶子总是以 `(Some, Some)` 到达 `scalars_are_equivalent_at`。通配短路覆盖：

| 左 | 右 | 现状 | 期望（本设计后） |
|----|----|------|------------------|
| `Bus` | `地铁站` | Modified | Unchanged |
| `Bus` | `null` / 缺失 | Modified | Unchanged |
| `null` / 缺失 | `Bus` | Modified | Unchanged |
| `Bus` | 数字/布尔 | Modified | Unchanged |
| `Bus` | `Bus` | Unchanged | Unchanged |
| `Metro` | `学校`（非通配 mismatch） | Modified | Modified（回归守卫） |

### 作用边界（明确不做）

通配只在标量等价判定（`scalars_are_equivalent_at`，即 `(Some, Some)` 叶子）中生效。若某 `RouteStop` 整项仅存在于一侧，叶子以 `(None, Some)` 出现、不经过该函数，整站仍判为 Added/Removed——这是正确行为（新站点本就是新增，不该被其 `BuildingType="Bus"` 单独拉回 Unchanged）。

---

## 测试（TDD）

**`src/config.rs` 单元测试**
- `is_wildcard` 对配置了 `"*"` 的 key 返回 true；对普通别名（如 `Metro`）返回 false；对未配置 key 返回 false；对未配置路径返回 false。

**`src/diff.rs` 集成测试**（内联构造带 `"Bus":"*"` 的 config，沿用 `deserializes_user_provided_overrides` 的写法）
- `Bus` ↔ `地铁站` → Unchanged。
- `Bus`（左）↔ `null`/缺失（右）→ Unchanged；反向 `null`/缺失（左）↔ `Bus`（右）→ Unchanged。
- `Bus` ↔ `Bus` → Unchanged。
- 同一 map 中的普通别名仍正常折叠（`Metro` ↔ `地铁站` → Unchanged）。
- 非通配 mismatch 仍 Modified（回归守卫，确保通配不吞掉真实差异）。

## 影响面与风险

- `config.canonicalize` 的唯一消费者是 `scalars_are_equivalent_at`（`batch_scan.rs` 中的 `canonicalize` 是 `Path::canonicalize`，无关）。改动完全局部化于 config.rs + diff.rs 两处。
- 不改配置 schema、不改 struct、不改兜底默认值，向后兼容。已有 `compare-config.json:22` 的写法在本设计下从"无效折叠"变为"按预期通配"。
