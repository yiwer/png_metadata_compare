# PNG 元数据对比工具 — 全面重设计 (Design Spec)

日期：2026-05-05
范围：前端 UI/UX 与展示层全面重构；后端 API 基本不动。

## 0 · 设计目标

应用核心：
- **1v1**：查看两张 PNG 的元数据树结构哪里不一样
- **目录 vs 目录**：哪边有哪边没有，哪些是 metadata 对不上的

本次重设计要解决：
1. 字段以英文显示，业务方读起来心智负担重 → 全面中文化（业务直白风）
2. 必须两边都填才能看 metadata → 任意一边填即可看
3. 现有树形不够立体、值不够清晰
4. 视觉规范零散 → 统一为高对比工程感系统

## 1 · 字段中文映射

### 顶层 StopPlateMetadata

| 字段 | 中文标签 | 值规则 |
|---|---|---|
| `StopId` | 站点编号 | 数字直显；null → `—` |
| `StopName` | 中文站名 | 字符串 |
| `StopEngName` | 英文站名 | 字符串 |
| `OriName` | 原站名 | 字符串；null → `—` |
| `RoadName` | 所在道路 | 字符串 |
| `DirectionOnRoad` | 在道路的方位 | 直显（东侧/西侧/南侧/北侧） |
| `DistrictName` | 所属行政区 | 字符串 |
| `StreetCommitteeName` | 所属街道 | 字符串 |
| `QRCode` | 二维码地址 | 字符串 |
| `HasHints` | 含温馨提示 | true → `是` ; false → `否` |
| `Hints` | 温馨提示内容 | 字符串；null → `—`；当 `HasHints=否` 时可折叠隐藏 |
| `IsGroupPrint` | 含分站信息 | true → `是` ; false → `否` |
| `GroupItems` | 分站列表 | 数组分组 |
| `IsBack` | 站牌朝向 | true → `反面` ; false → `正面` |
| `FrameSize` | 站架尺寸 | `1050×1660`（宽×高 mm）|
| `Lines` | 停靠线路 | 数组分组 |
| `RenderTime` | 渲染时间 | 字符串直显 |

### GroupItems[i]（数组元素摘要标签：`分站 ${SequenceNo}` → `分站 ①`）

| 字段 | 中文标签 | 值规则 |
|---|---|---|
| `SequenceNo` | 分站序号 | 直显（① / ② …） |
| `LineNames` | 该分站线路 | 字符串 |
| `Distance` | 距当前站 | `${n} 米` |
| `IsCurrent` | 是否当前站 | true → `是（当前）` ; false → `否` |

### Lines[i]（数组元素摘要标签：`线路 ${i+1} · ${LineName}` → `线路 1 · B932`）

| 字段 | 中文标签 | 值规则 |
|---|---|---|
| `LineName` | 线路名称 | 字符串 |
| `Direction` | 开往方向 | 字符串 |
| `FirstStopName` | 线路起点站 | 字符串 |
| `LastStopName` | 线路终点站 | 字符串 |
| `NextStop` | 下一站 | 字符串；null → `—（已到终点）` |
| `CurrentStopSequence` | 当前站站序 | 数字直显 |
| `IsStarting` | 当前站是起点 | `是` / `否` |
| `IsEnding` | 当前站是终点 | `是` / `否` |
| `HeadBusCorpName` | 运营企业 | 字符串 |
| `TicketType` | 票制类型 | 直显（一票制/分段收费） |
| `PriceDescription` | 票价描述 | 字符串 |
| `ServiceTimeDescription` | 服务时间 | 字符串 |
| `ScheduledServiceDescription` | 发车时刻表 | 字符串；null → `—（非定时班车）` |
| `LinePattern` | 线路模式 | 直显（单边/双边） |
| `RouteStops` | 途经站点 | 数组分组 |

### RouteStops[i]（数组元素摘要标签：`#${Sequence} · ${Name}` → `#3 · 翻身地铁站`）

| 字段 | 中文标签 | 值规则 |
|---|---|---|
| `Name` | 站点名称 | 字符串 |
| `Sequence` | 站点序号 | 数字直显 |
| `BuildingType` | 附近设施 | 见下方枚举；null/空 → `—` |
| `RoadName` | 所在道路 | 字符串 |

`BuildingType` 枚举：`地铁` → `地铁换乘`；`公交` → `公交换乘`；`医院` → `医院`；`长途` → `长途客运`；其他 → 直显。

### 通用值规则

- `null` → `—`（em dash，颜色为 `--text-tertiary`）
- 空字符串 → `(空)`
- 未在 schema 中的字段（向后兼容）→ 标签用原英文键名，附小灰色徽章 `未识别`
- 布尔值（除上述语境化）默认 → `是` / `否`

### 数组元素摘要标签生成器（伪代码）

```ts
function arrayItemLabel(arrayPath: string, index: number, item: JsonValue): string {
  switch (arrayPath) {
    case 'GroupItems':
      return `分站 ${item?.SequenceNo ?? index + 1}`;
    case 'Lines':
      return `线路 ${index + 1} · ${item?.LineName ?? '?'}`;
    case 'Lines[*].RouteStops':
      return `#${item?.Sequence ?? index + 1} · ${item?.Name ?? '?'}`;
    default:
      return `项 ${index + 1}`;
  }
}
```

## 2 · 信息架构与流程

### 模式开关

顶部 `单文件 | 目录` 二选一。切换模式清空槽位与结果。
- 单文件：槽位接受 PNG 文件
- 目录：槽位接受目录路径

### 槽位状态机

| 状态 | 触发 | 视觉 |
|---|---|---|
| 空 | 初始 | 虚线边框 + 拖入提示 + 浏览按钮 |
| 拖入中 | dragover | 边框变 `--accent` + 高亮填充 |
| 已填充 | 文件/目录已选 | 实线边框 + 文件名 + 副信息（路径/大小/元数据字节）+ 清除按钮 |
| 错误 | 文件无效 / 无元数据 | 红色边框 + 错误徽章 |

整个窗口都接受拖放（不限于槽位区域）；落入哪个槽位由当时高亮的目标决定（默认按位置：左半屏 → 左槽，右半屏 → 右槽）。

**类型与模式不匹配的处理**：拖入文件类型与当前模式不符时（如单文件模式拖入文件夹，或目录模式拖入 PNG），自动切换到匹配的模式并清空原槽位（行为提示：顶部冒泡 toast `已切换到目录模式`）。

### 槽位条折叠

两个槽位都已填充且首次分析（`compare_single` / `scan_directory`）成功返回后，槽位条从 ~80px 自动收缩为 40px 单行摘要：

```
左 · ...01_v1.png   ⇄   右 · ...01_v1.png   [▼ 展开]
```

点击 `▼` 重新展开。

### 渲染分支

| 槽位状态 | 视图 |
|---|---|
| `0 / 0` | 居中欢迎 + 拖放区 + 快捷键提示 |
| `1 / 0` 或 `0 / 1` | **SoloTree**（仅渲染该侧元数据，无差异样式） |
| `1 / 1` 单文件 | **MirrorTree**（左右镜像同步对比） |
| 目录 + 目录 | **DirectoryList**（统计 + 筛选 + 列表） |

### 目录 → 子页导航

列表行点击：
| 状态 | 进入 |
|---|---|
| 不一致 | MirrorTree |
| 仅左 | SoloTree（左） |
| 仅右 | SoloTree（右） |
| 一致 | MirrorTree（无差异色） |
| 错误 | 错误详情面板 |

子页顶栏：`← 返回目录` 按钮 + 进度标签 `${index} / ${totalDifferent} 处不一致`；`[` / `]` 在不一致项之间穿越。

### 顶栏（持久化）

```
[图标] PNG⌁Compare  │  [单文件|目录]  │  ← 返回目录   ┄ 文件名/进度 ┄   [─][□][✕]
```

### 控件条（槽位条与内容区之间）

- 视图切换：`树` / `JSON` / `图片`
  - 1v1 + 两边都有：三个全可用
  - solo：`图片` 可用；`JSON` 仅显示有的那一边
- `高亮差异` 开关（仅 1v1）
- `仅看不同` 开关（仅 1v1）
- 状态摘要（仅 1v1）：`4 处不同 · 1 仅右 · 0 仅左 · 0 顺序不同`
- 排序下拉（仅目录列表）：差异数量降序 / 文件名 / 修改时间

### 快捷键

| 键 | 行为 |
|---|---|
| `Ctrl+O` | 打开左侧选择 |
| `Ctrl+Shift+O` | 打开右侧 |
| `Ctrl+Enter` | 触发对比/扫描 |
| `[` / `]` | 上一/下一不一致项 |
| `Esc` | 返回目录 / 清空槽位 |
| `1` / `2` / `3` | 树 / JSON / 图片 |
| `D` | 切换差异高亮 |
| `Shift+Enter` | 1v1 内：跳到下一处不同 |

## 3 · 树形渲染规则

### 节点类型

| 类型 | 触发 | 渲染 |
|---|---|---|
| 叶子（string/number/boolean/null） | 值不是 object/array | 两栏行：`键 │ 值` |
| 对象分组 | object | 小标题（暖琥珀色 + uppercase 字距）+ 嵌套缩进 |
| 数组分组 | array | 小标题 + 数量徽章 `(18 项)` |
| 数组元素 | array 内的 object | 子小标题（用摘要标签生成器） |

### 视觉示意

```
█ STOP_INFO 站点信息
│ 中文站名         翻身地铁站
│ 英文站名         Fanshen Metro Station
│ 所在道路         创业一路
│ 在道路的方位      北侧
│ 站架尺寸         1050×1660 mm
│
█ DISPATCH 分站列表 (2 项)
│ ╲ 分站 ①
│ │ 分站序号       ①
│ │ 该分站线路     M197
│ │ 距当前站       150 米
│ │ 是否当前站     否
│ ╲ 分站 ② [当前]
│ │ ...
│
█ LINES 停靠线路 (5 项)
│ ╲ 线路 1 · B932
│ │ 线路名称       B932
│ │ 开往方向       福城万达广场
│ │ 下一站         尚都花园
│ │ 票价描述       一票制 1元
│ │ ╲ 途经站点 (18 项)         ← 默认折叠
│ │ │ ▶ 共 18 站
```

### 缩进 / 导引线

- 每级缩进 16px
- 嵌套小标题左侧 1px 实线 + 上方 4px 圆角拐角，形成"分组容器"
- 叶子两栏：键 130px（顶层）/ 110px（嵌套）+ 值 1fr，gap 16px

### 折叠

- 顶层分组：默认全部展开
- 嵌套数组（如 `途经站点`、`分站列表`）：默认 **折叠**，标题右侧 `▶ 共 N 项`
- 折叠状态在当前会话内记忆，不持久化

### 分组标题悬停动作

每个分组小标题悬停时显示行尾按钮：
- 折叠/展开 `▼` / `▶`
- `复制 JSON 子树`
- 1v1 模式额外：`只看本组差异`（隐藏本组之外的所有内容并禁用顶级 `仅看不同`）

### Solo 渲染（单边）

- 不显示差异背景色 / 徽章 / 同步滚动
- 顶部状态条：`仅查看 ${side} · ${文件名}`
- 视图保持完整两栏 + 小标题结构（不塌缩为单列）

## 4 · 1v1 镜像对比规则

### 同步滚动

- 双 pane 共用滚动控制器，左滚 → 右镜像滚，反之亦然
- 对齐单位：**逻辑路径**（同一字段两边永远在同一垂直线）
- 一侧分组在另一侧不存在 → 缺失侧插入"占位行"以保对齐

### 行级状态视觉

| 状态 | 左 pane | 右 pane | 说明 |
|---|---|---|---|
| `unchanged` | 正常 | 正常 | 无背景，正常字色 |
| `modified` | 黄色行底 + 值前 `←` | 黄色行底 + 值前 `→` | 两边同时染色，值字色保持完整对比度 |
| `added`（仅右） | 占位行（暗灰 `—`） | 绿色行底 | 缺失侧不染色，只灰色占位 |
| `removed`（仅左） | 红色行底 | 占位行 | 同上反向 |
| `reordered` | 蓝色 4px 左边框 | 同 | 元素相同但顺序不同 |

**关键约束**：值字符永远全亮（`--text-primary`），差异感来自背景色与边框，不靠淡化值文本。

### 占位行

```
│ — — — —     仅另一侧存在
```

灰色虚线 + 注解（`仅另一侧存在`），高度等于对侧实际行高。

### 控件条（1v1 专属）

```
[树|JSON|图片]   [✓ 高亮差异]  [≡ 仅看不同]   [↓ 跳到下一处不同]   [复制差异为 Markdown]
```

- `仅看不同`：隐藏所有 `unchanged` 行；保留小标题作锚点；其下若全 unchanged 则整组隐藏
- `仅看不同` 开启时，被默认折叠的嵌套数组若内部有差异，自动展开到差异行
- `跳到下一处不同` / `Shift+Enter`：滚动到下一个非 `unchanged` 行
- `复制差异为 Markdown`：把差异列表导出为多行 `站架尺寸: 1050×1660 → 1200×1800` 文本

### 摘要条

```
4 处不同 · 1 仅右 · 0 仅左 · 0 顺序不同
```

数字可点击 → 滚动到该类的第一处。

## 5 · 目录概览规则

### 顶部统计条

```
[23] 不一致     [5] 仅左     [3] 仅右     [142] 一致     · 173 总计
```

数字大字号着色，标签小字。点击 → 等同于点击对应筛选芯片。

### 筛选芯片

```
[全部 173]  [不一致 23]  [仅左 5]  [仅右 3]  [一致 142]  [错误 0]
```

单选；扫描完成后默认聚焦在 **`不一致`**（用户扫描的目的就是找问题）。

### 列表行

```
●  翻身地铁站_北_01_v1.png         [12 处不同]    ›
●  翻身地铁站_南_01_v1.png         [3 处不同]     ›
●  观澜湖_东_02_v1.png              [仅左侧]       ›
●  福永_北_01_v1.png                [仅右侧]       ›
●  坂田_西_03_v1.png                [7 处不同]     ›
○  沙井_南_02_v1.png                [一致]         ›   ← 半透明 0.55
```

- 左色点：`不一致 黄`、`仅左 红`、`仅右 绿`、`一致 灰`
- 文件名：mono 字体
- hover 显示左/右路径浮层提示
- 整行可点击；右键菜单：`复制路径` / `在资源管理器中显示`

### 自动分组

文件总数 > 50 且筛选 = `全部` 时，列表按状态分组：

```
▼ 不一致 (23)
   ●  ...
▼ 仅左 (5)
▼ 仅右 (3)
▶ 一致 (142)              ← 默认折叠
```

### 排序

- 按差异数量降序（默认）
- 按文件名 A-Z
- 按修改时间

### 错误项

`扫描错误`：单独 `错误 N` 芯片；行内显示错误码徽章（`E_NO_METADATA` / `E_TRUNCATED` / `E_INVALID_JSON`）；点击 → 错误详情面板（不进入对比视图）。

### 进度条

控件条下方 1px 条形进度条，扫描完成后淡出消失：`扫描中… 87 / 173`。

### 空状态 / 单槽位

- 两个目录槽都未填：居中欢迎，提示拖入两个目录
- 仅一个目录槽填了：列表显示该目录所有 PNG，全部标 `仅左侧 / 仅右侧`（取决于哪个槽位有值）。复用单边查看逻辑——点击进入 SoloTree。

## 6 · 视觉系统

### 调色板

```css
/* 表面 */
--bg-page:        #000000;
--bg-elevated:    #0a0a0a;
--bg-overlay:     #141414;

/* 文字 */
--text-primary:   #ffffff;
--text-secondary: #b8c0d0;
--text-tertiary:  #6c7480;
--text-disabled:  #3a3f4a;

/* 描边 */
--border-subtle:  #1a1a1a;
--border-default: #2a2a2a;
--border-emph:    #444a55;
--border-focus:   #5b8dff;

/* 状态（差异） */
--mod-bg:    #3a2a00;  --mod-text: #ffd23f;  --mod-arrow: #ffd23f;
--add-bg:    #0d2d18;  --add-text: #5eed90;  --add-emph:  #5eed90;
--rem-bg:    #330808;  --rem-text: #ff6b6b;  --rem-emph:  #ff6b6b;
--reord-bord:#5b8dff;
--err-bg:    #2a0d2a;  --err-text: #ff7ce0;

/* 分组 */
--group-head: #ffb547;
--group-rule: rgba(255,181,71,0.15);

/* 强调 */
--accent:    #5b8dff;
--accent-bg: rgba(91,141,255,0.15);
```

### 字体

```css
--font-ui:   "Inter","PingFang SC","Microsoft YaHei",system-ui,-apple-system,sans-serif;
--font-mono: "JetBrains Mono","SF Mono","Cascadia Code",ui-monospace,Consolas,monospace;
```

- 键、值、文件名、JSON、路径 → mono
- 小标题、按钮、徽章、UI 标签 → sans
- 数字（统计、徽章数量）→ mono（位宽对齐）

### 字号 / 行高

| Token | 值 | 用途 |
|---|---|---|
| `--fs-xs` | 11px / 1.4 | 标签 / 徽章 / 控件 |
| `--fs-sm` | 12px / 1.5 | 主体（树行） |
| `--fs-md` | 13px / 1.5 | 顶栏 / 标题文字 |
| `--fs-lg` | 14px / 1.4 | 区块标题 |
| `--fs-xl` | 18px / 1.3 | 统计大字 |
| `--fs-2xl` | 24px / 1.2 | 欢迎标题 |

### 间距 / 圆角

```css
--sp-1: 4px;  --sp-2: 8px;  --sp-3: 12px;
--sp-4: 16px; --sp-5: 24px; --sp-6: 32px;

--r-sm: 4px;  /* 徽章 / 芯片 / 输入 */
--r-md: 6px;  /* 卡片 / 槽位 / 浮层 */
--r-lg: 10px; /* 大容器 / 模态 */
```

### 状态徽章

```
.badge {
  font: 500 11px/1.4 var(--font-ui);
  letter-spacing: 0.02em;
  padding: 1px 8px;
  border-radius: var(--r-sm);
}
```

色板见上 `--mod-*` / `--add-*` / `--rem-*` / `--err-*` 配套（背景使用色 + 16% alpha，文字使用色 100%）。

### 分组小标题

```
.group-head {
  font: 500 11px/1.4 var(--font-ui);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--group-head);
  padding: 6px 10px;
  background: rgba(255,181,71,0.06);
  border-left: 2px solid var(--group-head);
  border-radius: 2px;
  margin: 12px 0 4px;
}
.group-head--nested {
  font-size: 10px;
  background: transparent;
  color: rgba(255,181,71,0.75);
  border-left-color: rgba(255,181,71,0.4);
  margin: 8px 0 2px;
}
```

### 树行键值

```
.kv {
  display: grid;
  grid-template-columns: 130px 1fr;
  column-gap: var(--sp-4);
  padding: 4px 10px;
  border-bottom: 1px solid var(--border-subtle);
  font: 12px/1.5 var(--font-mono);
}
.kv .key   { color: var(--text-secondary); }
.kv .value { color: var(--text-primary); word-break: break-all; }
.kv:hover  { background: var(--bg-overlay); }
```

### 焦点

```css
:focus-visible { outline: 2px solid var(--border-focus); outline-offset: 1px; }
```

### 滚动条（深色）

```css
::-webkit-scrollbar { width: 10px; height: 10px; }
::-webkit-scrollbar-thumb { background: #2a2a2a; border-radius: 5px; }
::-webkit-scrollbar-thumb:hover { background: #444a55; }
```

### 动效

- 槽位条折叠/展开：200ms `ease-out`
- 行 hover 背景：80ms `linear`
- 视图切换：无过渡（瞬切）
- 加载态：1px 顶部进度条 + 低饱和脉动占位卡

### 显式不做

- 无阴影
- 无渐变（除徽章半透明背景）
- 无圆角放大
- 无 emoji 装饰

## 7 · 实施轮廓

### 前端文件

新增 / 重写：

- `frontend/src/lib/labels.ts` — 字段中文映射表 + 数组元素摘要标签生成器
- `frontend/src/lib/treeModel.ts` — `JsonValue` × 标签 → `TreeNode[]`（叶子/对象组/数组组/数组元素）+ 折叠状态 + 状态映射 + 占位行
- `frontend/src/components/Slot.tsx`
- `frontend/src/components/SlotBar.tsx`（含折叠）
- `frontend/src/components/MirrorTree.tsx`（取代 `MetadataTree` + `DiffTree`）
- `frontend/src/components/SoloTree.tsx`
- `frontend/src/components/DirectoryList.tsx`（重写 `DirectoryOverview`）
- `frontend/src/components/StatusBadge.tsx`
- `frontend/src/components/GroupHead.tsx`
- `frontend/src/styles/tokens.css`（彻底重写为 §6 调色板）
- `frontend/src/styles/app.css`（仅保留全局重置）

调整：

- `frontend/src/features/workbench/useWorkbench.ts` — 新增 `view: 'solo'` 分支；目录单边项跳转到 solo；快捷键 hook
- `frontend/src/lib/api.ts` — 已有 `inspectSingle`，复用
- 拖放：浏览器 `drop` + Tauri `tauri://drag-drop` 事件双通道（确保拖入 OS 文件管理器与拖入应用窗口都生效）

### 后端

无 schema 变更。错误码已分类清晰，前端按 §5 规则映射文案。

### 数据流

```
原始 metadata (JsonValue)
    ↓
labels.ts: fieldLabel(path) + arrayItemLabel(path, item)
    ↓
treeModel.buildTree(value, labels) → TreeNode[]
    ↓
[1v1]                                     [solo]
diffTree(left, right) → DiffNode[]
buildMirror(treeL, treeR, diff)           buildSolo(tree)
    ↓                                     ↓
MirrorTree                                SoloTree
```

### 测试

升级：

- `useWorkbench.test.tsx` — 增加 solo / 单槽位 / 模式切换状态机断言
- `App.test.tsx` — 端到端 smoke：拖入一个文件 → solo；拖入第二个 → 镜像

新增：

- `labels.test.ts` — 中文映射表全字段覆盖；摘要标签生成器各数组路径
- `treeModel.test.ts` — null / 空数组 / 未识别字段 / 深嵌套
- `MirrorTree.test.tsx` — 占位行对齐、同步滚动、`仅看不同` 切换、`仅看不同` 自动展开嵌套数组

### 删除 / 退役

合入新组件后删除：

- `frontend/src/components/DiffTree.tsx`
- `frontend/src/components/MetadataTree.tsx`
- `frontend/src/components/EmptyState.tsx`
- `frontend/src/components/StatusBanner.tsx`
- `frontend/src/components/DiffStrip.tsx`
- `frontend/src/components/FileCard.tsx`
- `frontend/src/components/ViewModeStrip.tsx`（合入控件条）

保留并改造样式：
- `ImagePane.tsx`
- `RawJsonPanel.tsx`

### 实施顺序（建议）

1. `labels.ts` + `treeModel.ts` + 单测
2. `tokens.css` 全替换 + 共享原子组件（`StatusBadge`、`GroupHead`）
3. `SoloTree`
4. `MirrorTree`（基于 `SoloTree` 升级）
5. `Slot` + `SlotBar` + 顶栏
6. `DirectoryList` 重写
7. 快捷键 + `仅看不同` + `复制差异为 Markdown` 等增强
8. 删除旧组件 / 旧样式 / 测试清理

## 8 · 关键决策回顾

| 决策点 | 选择 | 理由 |
|---|---|---|
| 中文标签风格 | 业务直白（5-8 字） | 用户偏好；嵌套层级中读起来自然 |
| 单 PNG 查看形态 | 拖入即看 | 取消"必须两边都填"的硬约束；统一目录单边项的体验 |
| 树形展示 | 两栏键值 + 缩进导引 + 全量小标题 | 结构感最强；扫描节奏稳定 |
| 1v1 布局 | 左右镜像 + 同步滚动 | 核心心智匹配"哪里不一样" |
| 目录概览 | 单列表 + 筛选芯片 | 长文件名友好；扫描快 |
| 工作台外壳 | 顶部槽位条（可折叠） | 镜像视图需要宽度，边栏浪费 |
| 视觉基调 | 黑底极致高对比 | 工程感；值字符不被淡化 |

## 9 · 范围之外

- 国际化：本设计假定 UI 文案中文。若未来需要英文/多语言，需要把映射表和值规则抽成 i18n 表（不在本次范围）
- 后端 API 变更：不在本次范围
- 元数据 schema 演进（新字段、新枚举值）：通过"未识别字段"机制兜底；schema 升级时再扩 labels 表
- 持久化用户偏好（折叠状态、视图模式等）：本次仅会话级，不写入磁盘
