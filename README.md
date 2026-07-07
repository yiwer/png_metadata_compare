# PNG Metadata Compare

对比公交站牌 PNG 图片内嵌元数据的桌面工具（Tauri 2 + React）。

站牌渲染系统会把站牌的结构化数据（线路、站点、方位、票价等）以 JSON 形式写入 PNG 的
`iTXt` 块。本工具读取左右两侧图片中的这段 JSON，做结构化 diff，用于核对两个批次 /
两套渲染产物的站牌内容是否一致——而不是逐像素比较图片。

## 功能

- **单对对比**：选择左右两张 PNG，逐字段展示元数据差异，并排预览图片（支持缩放）。
- **目录批量扫描**：选择左右两个目录，递归扫描并自动配对，汇总为
  相同 / 有差异 / 仅左侧 / 仅右侧 四类；扫描带进度上报，可随时取消。
- **文件名自动配对**：识别两套命名规范并归一为同一配对键（详见 `src/pairing_key.rs`）：
  - 站牌：`路段名_站点名称_站点方位_站牌序号_尺寸_正|反` ↔
    `路段名_站点名称_站点方位_站架序号_尺寸_A|B`（`站牌序号 = (站架序号-1)*2 + 正/A→1、反/B→2`）
  - 插片：`路段名_站点名称_站点方位_尺寸_A|B|C` ↔ `路段名_站点名称_站点方位_一|二|三_序号_尺寸`
  - 配对键有意忽略 `路段名` 与 `尺寸`。
- **可配置比对规则**：忽略字段、值等价映射、`"*"` 通配值（见下文「比对配置」）。
- **单侧查看**：无配对的文件可单独查看其元数据与图片。

## 技术栈与结构

后端 Rust（Tauri 2），前端 React 18 + TypeScript + Vite；前后端通过 Tauri
command（均为 async，重活跑在阻塞线程池上，进度经 `Channel` 推送）通信。

```
src/                     Rust 后端
├── main.rs              Tauri 入口，注册 commands
├── desktop_api.rs       Tauri commands：compare_single / scan_directory /
│                        inspect_single / cancel_scan / pick_folder
├── png_reader.rs        解析 PNG iTXt 块，提取元数据 JSON
├── metadata.rs          元数据 JSON 解析
├── diff.rs              结构化 diff（忽略规则、等价映射、通配值）
├── config.rs            compare-config.json 加载与默认规则
├── pairing_key.rs       文件名 → 配对键
├── batch_scan.rs        目录递归扫描、配对、进度与取消
├── batch_report.rs      批量结果汇总
└── inspection.rs        面向 UI 的单对/单侧/目录检查入口

frontend/                React 前端（Vite + Vitest）
├── src/features/workbench/   工作台状态（useWorkbench）
├── src/components/           SelectionBar / Sidebar / UnifiedTree /
│                             DiffRail / ImageViews / WelcomePane 等
└── src/lib/                  Tauri API 封装、diff 列表、树模型、标签等

docs/superpowers/        各功能的设计 spec 与实施 plan（按日期归档）
```

## 元数据格式

从 PNG 的 `iTXt` 块按以下 keyword 顺序查找，取第一个命中者的文本作为 JSON 载荷：
`StopPlateMetadata`、`metadata`、`Metadata`、`stopMetadata`。

## 开发

依赖：Rust（edition 2024）、Node.js、（打包时）`tauri-cli`。

```bash
# 终端 1：启动前端 dev server（http://localhost:5173）
cd frontend
npm install
npm run dev

# 终端 2：启动桌面壳（debug 构建加载 devUrl）
cargo run
```

测试：

```bash
cargo test                    # Rust
cd frontend && npx vitest run # 前端
```

## 构建发布版

Tauri 以 `custom-protocol` feature（而非 cargo profile）区分 dev/生产：
不带该 feature 的构建始终加载 `devUrl`，`cargo run --release` 也不例外。

```bash
# 方式一：tauri CLI，产出 NSIS/MSI 安装包（自动启用 custom-protocol）
cargo tauri build

# 方式二：直接用 cargo 出生产可执行文件
cd frontend && npm run build && cd ..
cargo run --release --features custom-protocol
```

## 比对配置

规则内置默认值；如需覆盖，把 `compare-config.example.json` 复制为可执行文件工作目录下的
`compare-config.json`（或 `config/compare-config.json`），启动时按此顺序查找，均不存在
则回落到内置默认。

```jsonc
{
  // 命中即跳过比对的字段路径，数组用 [*] 通配
  "ignored_fields": [
    "Lines[*].RouteStops[*].Sequence",
    "RenderTime"
  ],

  // 值等价映射：比对前把 key 折叠为 value（大小写敏感）
  "equivalence_maps": {
    "Lines[*].RouteStops[*].BuildingType": {
      "Metro": "地铁站",   // 左"Metro"与右"地铁站"视为相同
      "Bus": "*"           // 特例：映射到 "*" 表示通配——"Bus" 与对侧任何值
    }                      // （含 null / 字段缺失）都视为等价
  }
}
```

仓库根目录的 `compare-config.json` 是当前实际使用的规则，随业务口径持续调整。
