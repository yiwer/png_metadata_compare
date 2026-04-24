# MOTHERDUCK DESIGN SYSTEM GUIDE — COMPLETE EDITION

## 1. OVERVIEW

| 属性                | 值                                                           |
| ------------------- | ------------------------------------------------------------ |
| Brand Name          | MotherDuck                                                   |
| Tagline             | "Infrastructure for Answers"                                 |
| Product Description | The cloud data warehouse built for answers, in SQL or natural language. Fast, serverless analytics powered by DuckDB. |
| Design Language     | Brutalist-minimalist：粗边框、无圆角、无阴影、高色彩对比、等宽字体 |
| Tech Stack Hint     | Styled-components (CSS-in-JS), React, Mobile-first responsive |

**核心设计原则（Agent 决策参考）：**
- 所有主要组件使用 `border-radius: 0px`（除 nav 菜单项为 `2px`）
- 用粗边框（2–3px solid）代替阴影实现层次感
- 颜色大量对比：黄色 + 蓝色 + 黑色 + 米白底
- 字体全局使用等宽字体 `Aeonik Mono`，UI 标签类使用 `Aeonik`，辅助/小字使用 `Inter`
- CTA 文字、导航、按钮全部 `text-transform: uppercase`

---

## 2. COLOR PALETTE（完整版）

### 2.1 核心品牌色

| Token名称                | HEX       | RGB                  | 用途                                                  |
| ------------------------ | --------- | -------------------- | ----------------------------------------------------- |
| `color-yellow-primary`   | `#FFDE00` | `rgb(255, 222, 0)`   | Banner背景、Badge、页面section强调块                  |
| `color-yellow-secondary` | `#F9EE3E` | `rgb(249, 238, 62)`  | 次级高亮、强调装饰                                    |
| `color-blue-primary`     | `#6FC2FF` | `rgb(111, 194, 255)` | 主CTA按钮背景、卡片header背景、数据工程师标签、图表色 |
| `color-teal-accent`      | `#53DBC9` | `rgb(83, 219, 201)`  | 数据科学家标签（角色颜色标识）                        |
| `color-red-accent`       | `#FF7169` | `rgb(255, 113, 105)` | 错误、警告、警示元素                                  |

### 2.2 中性/背景色

| Token名称              | HEX                        | RGB                  | 用途                              |
| ---------------------- | -------------------------- | -------------------- | --------------------------------- |
| `color-bg-beige`       | `#F4EFEA`                  | `rgb(244, 239, 234)` | **全局页面背景**、Fixed导航栏背景 |
| `color-bg-offwhite`    | `#F8F8F7`                  | `rgb(248, 248, 247)` | AI Widget 主体背景、次级背景区域  |
| `color-bg-offwhite-70` | `rgba(248, 248, 247, 0.7)` | —                    | 搜索输入框背景（半透明）          |
| `color-bg-white`       | `#FFFFFF`                  | `rgb(255, 255, 255)` | 卡片背景、内容区白色层            |
| `color-bg-light-gray`  | `#F1F1F1`                  | `rgb(241, 241, 241)` | 进度条track、分隔元素             |
| `color-bg-light-smoke` | `#FAF9F5`                  | `rgb(250, 249, 245)` | 极浅背景变体                      |

### 2.3 文字色

| Token名称               | HEX       | RGB                  | 用途                                    |
| ----------------------- | --------- | -------------------- | --------------------------------------- |
| `color-text-dark`       | `#383838` | `rgb(56, 56, 56)`    | **主要文字色**、按钮文字、边框颜色      |
| `color-text-black`      | `#000000` | `rgb(0, 0, 0)`       | 高对比度标题、卡片内文字                |
| `color-text-white`      | `#FFFFFF` | `rgb(255, 255, 255)` | 深色背景上的文字（Footer、深色toolbar） |
| `color-text-gray`       | `#818181` | `rgb(129, 129, 129)` | 辅助信息、eyebrow 标签（如"FEATURES"）  |
| `color-text-mid-gray`   | `#C0C0C0` | `rgb(192, 192, 192)` | 禁用状态文字/图标                       |
| `color-text-light-gray` | `#A1A1A1` | `rgb(161, 161, 161)` | 占位符、元信息                          |

### 2.4 功能色 / 状态色

| Token名称            | HEX       | RGB                  | 用途                                                   |
| -------------------- | --------- | -------------------- | ------------------------------------------------------ |
| `color-success`      | `#22C55E` | `rgb(34, 197, 94)`   | 完成步骤 checkmark 背景（18×18px 方块）                |
| `color-success-bg`   | `#E8F5E9` | `rgb(232, 245, 233)` | 成功状态卡片背景                                       |
| `color-dark-surface` | `#383838` | `rgb(56, 56, 56)`    | Footer背景、AI Widget toolbar、CTA容器背景、深色模式面 |
| `color-ai-badge`     | `#5A5A5A` | `rgb(90, 90, 90)`    | "AI" 内嵌标签背景（在深色toolbar中）                   |

### 2.5 数据可视化图表专用色（用于折线图/柱状图序列）

按优先级排列（第1系列→第N系列）：

| 序号 | HEX       | RGB                  | 建议语义       |
| ---- | --------- | -------------------- | -------------- |
| 1    | `#54B4DE` | `rgb(84, 180, 222)`  | 主数据系列蓝   |
| 2    | `#38C1B0` | `rgb(56, 193, 176)`  | 次数据系列青绿 |
| 3    | `#B291DE` | `rgb(178, 145, 222)` | 第三系列紫     |
| 4    | `#B3C419` | `rgb(179, 196, 25)`  | 第四系列黄绿   |
| 5    | `#E1C427` | `rgb(225, 196, 39)`  | 第五系列金黄   |
| 6    | `#84A6BC` | `rgb(132, 166, 188)` | 第六系列钢蓝   |
| 7    | `#7597EE` | `rgb(117, 151, 238)` | 第七系列蓝紫   |
| 8    | `#F38E84` | `rgb(243, 142, 132)` | 第八系列珊瑚红 |
| 9    | `#F5B161` | `rgb(245, 177, 97)`  | 第九系列橙     |

### 2.6 语义色卡背景（用于数据卡片/标签色块的淡色底）

| 用途      | HEX       | RGB                  |
| --------- | --------- | -------------------- |
| 信息/蓝色 | `#EBF9FF` | `rgb(235, 249, 255)` |
| 成功/绿色 | `#E8F5E9` | `rgb(232, 245, 233)` |
| 紫色      | `#F7F1FF` | `rgb(247, 241, 255)` |
| 黄绿      | `#F9FBE7` | `rgb(249, 251, 231)` |
| 暖黄      | `#FFFDE7` | `rgb(255, 253, 231)` |
| 蓝灰      | `#ECEFF1` | `rgb(236, 239, 241)` |
| 蓝紫      | `#EAF0FF` | `rgb(234, 240, 255)` |
| 粉红      | `#FFEBE9` | `rgb(255, 235, 233)` |
| 橙暖      | `#FDEDDA` | `rgb(253, 237, 218)` |

---

## 3. TYPOGRAPHY（完整版）

### 3.1 字体族

| 字体                       | 类型                 | 用途                                                     |
| -------------------------- | -------------------- | -------------------------------------------------------- |
| `"Aeonik Mono"`            | 等宽字体（主字体）   | H1、H2-H6（大标题）、body 正文、按钮、nav、badge、input  |
| `"Aeonik Fono"`            | 衬线等宽（特殊标签） | Banner文字、导航注册链接                                 |
| `Aeonik`                   | 非等宽（紧凑标签）   | AI widget内部标题（H2 如 "MotherDuck AI"）、深色区域标签 |
| `Inter`                    | 无衬线（辅助）       | Footer链接/标题、图表辅助文字、eyebrow小标签             |
| `"Aeonik Mono", monospace` | 等宽变体             | 代码块、数据表格内文字                                   |

**加载方式（Agent 参考）：** Aeonik 系列为商业字体，需通过 `@font-face` 或字体托管服务加载。备用字体栈：`"Aeonik Mono", monospace, sans-serif`。

### 3.2 Type Scale（完整排版刻度）

| 级别             | 元素                  | 字号   | 字重      | 字体          | 行高           | 字间距   | TextTransform | 颜色               |
| ---------------- | --------------------- | ------ | --------- | ------------- | -------------- | -------- | ------------- | ------------------ |
| Display / H1     | 英雄区大标题          | `56px` | `400`     | `Aeonik Mono` | `67.2px` (1.2) | `1.12px` | `uppercase`   | `#383838`          |
| H2 Large         | 主要章节标题          | `48px` | `400`     | `Aeonik Mono` | `57.6px` (1.2) | `normal` | `uppercase`   | `#383838`          |
| H2 Medium        | 次要章节标题          | `32px` | `400`     | `Aeonik Mono` | `44.8px` (1.4) | `normal` | `uppercase`   | `#383838`          |
| H2 Small         | 模块标题              | `24px` | `500`     | `Aeonik Mono` | `28.8px` (1.2) | `normal` | `uppercase`   | `#383838`          |
| H2 Widget        | AI Widget 标题        | `24px` | `400`     | `Aeonik Mono` | `28.8px`       | `normal` | `uppercase`   | `#383838`          |
| H2 Label         | 深色区域小标题        | `16px` | `600`     | `Aeonik`      | `19.2px`       | `0.48px` | `none`        | `#FFFFFF`          |
| H3 Role Tag      | 角色标签标题          | `18px` | `400`     | `Aeonik Mono` | `25.2px`       | `normal` | `uppercase`   | `#383838`          |
| H3 Section Tag   | 区块分类标签          | `14px` | `400`     | `Aeonik Mono` | `16.8px`       | `normal` | `uppercase`   | `#383838`          |
| H3 Eyebrow       | 上眉标（如 FEATURES） | `10px` | `600`     | `Inter`       | `14px`         | `normal` | `uppercase`   | `rgb(129,129,129)` |
| Body Large       | 大段正文/副标题       | `16px` | `400`     | `Aeonik Mono` | `normal`       | `normal` | `none`        | `#383838`          |
| Body Default     | 正文                  | `14px` | `400`     | `Aeonik Mono` | `16.8px`       | `normal` | `none`        | `#383838`          |
| Body Small       | 小字/注释             | `13px` | `400–500` | `Aeonik Mono` | —              | `normal` | `none`        | `#383838`          |
| Body XSmall      | 极小文字              | `12px` | `400–500` | `Aeonik Mono` | —              | `normal` | `none`        | `#383838`          |
| Caption          | 图表注释/辅助         | `11px` | `400–600` | `Aeonik Mono` | —              | `normal` | `none`        | `#383838`          |
| Badge Counter    | Badge数字             | `12px` | `500`     | `Aeonik Mono` | —              | `normal` | `lowercase`   | `#383838`          |
| AI Tag           | "AI" 标签             | `13px` | `600`     | `Aeonik`      | —              | `normal` | `none`        | `#FFFFFF`          |
| Stop/Mini Button | 极小操作按钮          | `10px` | `500`     | `Aeonik Mono` | —              | `normal` | `uppercase`   | `#FFFFFF`          |
| Footer Brand     | Footer 品牌名         | `14px` | `700`     | `Inter`       | —              | `normal` | `none`        | `#FFFFFF`          |
| Footer Links     | Footer 链接           | `14px` | `300`     | `Inter`       | —              | `normal` | `none`        | `#FFFFFF`          |
| Banner Text      | 顶部 Banner           | `14px` | `400`     | `Aeonik Fono` | `16.8px`       | `normal` | `uppercase`   | `#000000`          |
| Nav Item         | 导航文字              | `16px` | `400`     | `Aeonik Mono` | —              | `0.32px` | `none`        | `#383838`          |

### 3.3 字重刻度

| Weight | 用途                                            |
| ------ | ----------------------------------------------- |
| `300`  | Footer 链接文字（轻量辅助文字）                 |
| `400`  | 全局默认（标题、正文、按钮、nav）               |
| `500`  | 数据标签、小型按钮、medium强调                  |
| `600`  | 卡片 header、badge 数字、AI widget标题、eyebrow |
| `700`  | Footer 品牌标题                                 |

---

## 4. SPACING SYSTEM（完整版）

### 4.1 基础间距单位

采用 **4px 基础单位**，实际使用以下常见值：

`2, 4, 6, 8, 10, 12, 16, 20, 22, 24, 28, 32, 40, 48, 64, 72, 80, 90, 92, 140, 160, 180`

### 4.2 响应式断点

| 断点名           | 最小宽度 | max-width容器 | 水平Padding |
| ---------------- | -------- | ------------- | ----------- |
| Mobile (default) | `0px`    | 全宽          | `24px`      |
| Tablet           | `728px`  | `728px`       | `20px`      |
| Desktop          | `960px`  | `960px`       | `20px`      |
| Large Desktop    | `1302px` | `1302px`      | `30px`      |

### 4.3 Section 上下内边距

| 断点                    | padding-top | padding-bottom |
| ----------------------- | ----------- | -------------- |
| Mobile                  | `24px`      | `82px`         |
| Tablet (≥728px)         | `64px`      | `140px`        |
| Desktop (≥960px)        | `80px`      | `180px`        |
| Large Desktop (≥1302px) | `92px`      | `160px`        |

### 4.4 常见组件内边距

| 组件                        | Padding                          |
| --------------------------- | -------------------------------- |
| 主CTA按钮（大）             | `16.5px 22px`                    |
| 主CTA按钮（小/nav）         | `11.5px 18px`                    |
| 次要按钮                    | `16.5px 22px`                    |
| Nav 菜单项                  | `12px`（四边）                   |
| 卡片                        | `20px`                           |
| 卡片 header                 | `12px 16px`                      |
| Input 输入框                | `0px 16px`（高度由 height 决定） |
| Input 大型                  | `16px 40px 16px 24px`            |
| Badge / Counter             | `6.6px 10px`                     |
| AI Tag ("AI")               | `2px 4px`                        |
| Stop 按钮                   | `6px 12px`                       |
| Banner                      | `16px 24px`                      |
| Footer                      | `90px 0px 72px`                  |
| Widget Toolbar              | `16px`（四边）                   |
| Tooltip                     | `8px 12px`                       |
| Section Tag (H3 with color) | `4px 16px`                       |

### 4.5 Flex / Grid 间距

| 用途                    | Gap         |
| ----------------------- | ----------- |
| 按钮内部（图标+文字）   | `8px`       |
| Widget toolbar 内部元素 | `12px`      |
| 按钮组（CTA并排）       | `16px` 推算 |

---

## 5. COMPONENT STYLES（扩充版）

### 5.1 按钮系统

#### Primary Button（主CTA）
```css
.btn-primary {
  background-color: rgb(111, 194, 255);  /* #6FC2FF */
  color: rgb(56, 56, 56);               /* #383838 — 注意：非纯黑 */
  border: 2px solid rgb(56, 56, 56);
  border-radius: 2px;                   /* 轻微圆角 */
  padding: 16.5px 22px;                 /* 大尺寸 */
  font-family: "Aeonik Mono", sans-serif;
  font-size: 16px;
  font-weight: 400;
  text-transform: uppercase;
  letter-spacing: normal;
  line-height: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: transform 0.12s ease-in-out;
  text-decoration: none;
}
.btn-primary:hover {
  transform: scale(1.02);              /* 使用 transform 而非 opacity */
}
```
**注：** 按钮的实际容器外层有 `background: rgb(56,56,56); border-radius: 2px` 的深色包裹容器，形成"双层框"视觉效果（外深色框 + 内蓝色按钮）。

#### Secondary Button（次级）
```css
.btn-secondary {
  background-color: rgb(244, 239, 234);  /* 与页面背景同色 */
  color: rgb(56, 56, 56);
  border: 2px solid rgb(56, 56, 56);
  border-radius: 2px;
  padding: 16.5px 22px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 16px;
  font-weight: 400;
  text-transform: uppercase;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: transform 0.12s ease-in-out;
}
.btn-secondary:hover {
  transform: scale(1.02);
}
```

#### Nav Menu Item（导航按钮）
```css
.btn-nav {
  background-color: transparent;
  color: rgb(56, 56, 56);
  border: 1px solid transparent;    /* 有边框但透明，hover时可变色 */
  border-radius: 2px;
  padding: 0px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 16px;
  font-weight: 400;
  text-transform: none;             /* nav 不强制大写 */
  letter-spacing: 0.32px;
  cursor: pointer;
}
```

#### Mini / Stop Button（极小操作按钮，用于 Widget toolbar）
```css
.btn-mini {
  background-color: transparent;
  color: rgb(255, 255, 255);
  border: 1px solid rgb(255, 255, 255);
  border-radius: 0px;
  padding: 6px 12px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 10px;
  font-weight: 500;
  text-transform: uppercase;
  cursor: pointer;
}
```

### 5.2 Badge / Tag 系统

#### Counter Badge（如 "3 QUESTIONS LEFT"）
```css
.badge-counter {
  background-color: rgb(255, 222, 0);
  color: rgb(56, 56, 56);
  padding: 6.6px 10px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 12px;
  font-weight: 500;
  border-radius: 0px;
  display: inline-block;
  text-transform: none;             /* lowercase: "3 questions left" */
}
```

#### AI 系统标签（"AI" 嵌入在深色toolbar）
```css
.badge-ai {
  background-color: rgb(90, 90, 90);  /* #5A5A5A */
  color: rgb(255, 255, 255);
  padding: 2px 4px;
  font-family: Aeonik, sans-serif;
  font-size: 13px;
  font-weight: 600;
  border-radius: 0px;
  display: inline-block;
}
```

#### 角色分类标签（Role Tag，用于 persona 分类）
```css
/* 每种角色用不同背景色 */
.role-tag {
  padding: 4px 16px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 18px;
  font-weight: 400;
  text-transform: uppercase;
  border: 2px solid rgb(56, 56, 56);
  border-radius: 0px;
  color: rgb(56, 56, 56);
}
/* 色值映射 */
.role-tag--software-engineers { background-color: rgb(255, 222, 0); }   /* Yellow */
.role-tag--data-scientists    { background-color: rgb(83, 219, 201); }  /* Teal */
.role-tag--data-engineers     { background-color: rgb(111, 194, 255); } /* Blue */
.role-tag--active             { background-color: rgb(56, 56, 56); color: rgb(255, 255, 255); }
```

### 5.3 Card 系统

#### Report Card（主卡片）
```css
.card {
  background-color: rgb(255, 255, 255);
  border: 3px solid rgb(56, 56, 56);     /* 注意：3px，比标准 2px 更粗 */
  border-radius: 0px;
  padding: 0px;
  box-shadow: none;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 16px;
  overflow: hidden;                       /* header 负 margin 替换为 overflow hidden */
}
.card-header {
  background-color: rgb(111, 194, 255);  /* #6FC2FF */
  color: rgb(0, 0, 0);
  padding: 12px 16px;
  font-size: 16px;
  font-weight: 400;
}
.card-body {
  padding: 16px 20px;
  background-color: rgb(255, 255, 255);
}
```

#### Content Card / Feature Card
```css
.card-content {
  background-color: rgb(255, 255, 255);
  border: 2px solid rgb(56, 56, 56);
  border-radius: 0px;
  padding: 20px;
  box-shadow: none;
}
```

### 5.4 Banner Component

```css
.banner {
  background-color: rgb(255, 222, 0);
  color: rgb(0, 0, 0);
  padding: 0px;
  font-family: "Aeonik Fono", "Aeonik Mono";
  font-size: 14px;
  font-weight: 400;
  text-transform: uppercase;
  text-align: center;
  border-radius: 0px;
}
.banner-inner {
  padding: 16px 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}
.banner-link {
  text-decoration: underline;
  color: rgb(0, 0, 0);
  font-size: 14px;
  font-weight: 400;
  text-transform: uppercase;
}
```

### 5.5 Navigation / Header

```css
/* Fixed 顶部导航容器 */
.nav-wrapper {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 99;
  background-color: rgb(244, 239, 234);  /* 与页面背景同色 */
  border-bottom: 2px solid rgb(56, 56, 56);
  height: 127px;                         /* 含 banner 时约 127px */
}

/* Logo */
.nav-logo {
  /* SVG duck logo，橙色 #F59E3F 系颜色 */
  height: 32px;
}

/* 汉堡菜单（移动端） */
.nav-hamburger {
  width: 24px;
  height: 24px;
  color: rgb(56, 56, 56);
}

/* Dropdown 菜单链接项 */
.nav-link {
  display: flex;
  align-items: center;
  padding: 12px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 16px;
  color: rgb(56, 56, 56);
  text-decoration: none;
  border-radius: 0px;
}
.nav-link:hover {
  background-color: rgb(244, 239, 234);
}
```

### 5.6 Form Input 系统

#### 标准文本输入框
```css
.input-text {
  background-color: rgb(255, 255, 255);
  color: rgb(56, 56, 56);
  border: 2px solid rgb(56, 56, 56);
  border-radius: 0px;
  padding: 0px 16px;
  height: 44px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 14px;
  font-weight: 400;
  outline: none;
  width: 100%;
  box-sizing: border-box;
}
.input-text::placeholder {
  color: rgb(161, 161, 161);
}
.input-text:focus {
  outline: none;
  border-color: rgb(56, 56, 56);         /* 保持同色，无高亮变化 */
}
```

#### 大型搜索输入框
```css
.input-search-large {
  background-color: rgba(248, 248, 247, 0.7);
  color: rgb(0, 0, 0);
  border: 2px solid rgb(56, 56, 56);
  border-radius: 2px;
  padding: 16px 40px 16px 24px;
  height: 58px;
  font-family: Inter, sans-serif;
  font-size: 16px;
  outline: none;
}
```

#### 提交按钮（禁用态示例，附在输入框旁）
```css
.input-submit-disabled {
  background-color: rgb(215, 215, 215);
  color: rgb(192, 192, 192);
  border: 2px solid rgb(56, 56, 56);
  border-radius: 0px;
  cursor: not-allowed;
}
```

### 5.7 AI Widget Panel（完整结构）

```css
/* 外层容器 */
.ai-widget {
  background-color: rgb(248, 248, 247);  /* #F8F8F7 */
  border: 2px solid rgb(56, 56, 56);
  border-radius: 0px;
  overflow: hidden;
}

/* Toolbar（顶部深色条） */
.ai-widget-toolbar {
  background-color: rgb(56, 56, 56);
  color: rgb(255, 255, 255);
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  height: 57px;
  z-index: 20;
  position: relative;
}

/* "MotherDuck" 品牌文字 + AI badge 组合 */
.ai-widget-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 16px;
  font-weight: 400;
  color: rgb(255, 255, 255);
}

/* "AI" 标签 */
.ai-badge {
  background-color: rgb(90, 90, 90);
  color: rgb(255, 255, 255);
  padding: 2px 4px;
  font-size: 13px;
  font-weight: 600;
  font-family: Aeonik, sans-serif;
  border-radius: 0px;
}

/* "Powered by MotherDuck MCP" 副标题 */
.ai-widget-subtitle {
  font-family: "Aeonik Mono", monospace;
  font-size: 12px;
  font-weight: 500;
  color: rgba(255,255,255,0.7);
}

/* 右侧状态 Badge（"3 questions left"） */
.ai-widget-counter {
  background-color: rgb(255, 222, 0);
  color: rgb(56, 56, 56);
  padding: 6.6px 10px;
  font-size: 12px;
  font-weight: 500;
  font-family: "Aeonik Mono", sans-serif;
  border-radius: 0px;
  text-transform: none;
}

/* 内容区 */
.ai-widget-body {
  padding: 20px;
  background-color: rgb(248, 248, 247);
}

/* 输入区域 */
.ai-widget-input {
  background-color: rgb(255, 255, 255);
  color: rgb(56, 56, 56);
  border: 2px solid rgb(56, 56, 56);
  border-radius: 0px;
  padding: 0px 16px;
  height: 44px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 14px;
}
```

### 5.8 Tooltip

```css
.tooltip {
  background-color: rgb(56, 56, 56);
  color: rgb(255, 255, 255);
  padding: 8px 12px;
  border-radius: 0px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 14px;
  box-shadow: none;
  position: absolute;
  z-index: 20;
}
```

### 5.9 Status Indicator（处理步骤 Checkmark）

```css
.status-check {
  width: 18px;
  height: 18px;
  background-color: rgb(34, 197, 94);   /* 绿色 */
  color: rgb(56, 56, 56);
  border-radius: 3px;                   /* 轻微圆角，与全局 0px 不同 */
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
}
```

### 5.10 Footer

```css
footer {
  background-color: rgb(56, 56, 56);
  color: rgb(255, 255, 255);
  padding: 90px 0px 72px;
  font-family: "Aeonik Mono", sans-serif;
  font-size: 16px;
}
.footer-brand-name {
  font-family: Inter, sans-serif;
  font-size: 14px;
  font-weight: 700;
  color: rgb(255, 255, 255);
}
.footer-link {
  font-family: Inter, sans-serif;
  font-size: 14px;
  font-weight: 300;
  color: rgb(255, 255, 255);
  text-decoration: none;
}
.footer-link:hover {
  text-decoration: underline;
}
.footer-section-title {
  font-family: Inter, sans-serif;
  font-size: 14px;
  font-weight: 700;
  color: rgb(255, 255, 255);
  margin-bottom: 12px;
}
/* Footer 内的白色装饰线分隔 */
.footer-divider {
  background-color: rgb(255, 255, 255);
  height: 1px;
  opacity: 0.2;
}
```

---

## 6. BORDER SYSTEM（完整版）

### 6.1 边框规格

| 用途                                  | 规格                           | 颜色                  |
| ------------------------------------- | ------------------------------ | --------------------- |
| 主组件默认边框（card, input, widget） | `2px solid`                    | `rgb(56, 56, 56)`     |
| 强调边框（report card, nav wrapper）  | `3px solid`                    | `rgb(56, 56, 56)`     |
| Nav 菜单项焦点框                      | `1px solid transparent`        | 透明（hover时可显现） |
| 黄色强调边框                          | `3px solid rgb(255, 222, 0)`   | —                     |
| 蓝色强调边框                          | `2px solid rgb(111, 194, 255)` | —                     |
| Mini button 边框（白色轮廓）          | `1px solid rgb(255, 255, 255)` | —                     |
| 图表数据点边框（圆角版）              | `2px solid [series-color]`     | r=3px                 |

### 6.2 Border Radius 规格

| 值    | 用途                                                         |
| ----- | ------------------------------------------------------------ |
| `0px` | 所有主要组件（按钮、卡片、badge、input、widget、banner、tooltip） |
| `2px` | Nav 菜单按钮/链接、CTA容器包裹层、大型搜索框                 |
| `3px` | 状态指示器（checkmark）、数据图表圆角元素                    |

---

## 7. SHADOW & ELEVATION

全站采用 **Flat Design**，所有 `box-shadow: none`。

层次关系通过以下方式建立：
1. **粗边框**（2–3px solid dark）
2. **背景色差异**（深色 vs 浅色 vs 白色）
3. **z-index 分层**：
   - `z-index: 99`：Fixed 导航栏
   - `z-index: 20`：AI Widget toolbar、覆盖元素
   - `z-index: 3`：浮动卡片
   - `z-index: 1`：层叠输入元素

---

## 8. ANIMATION & TRANSITION

### 8.1 标准过渡

| 用途                | Transition                          |
| ------------------- | ----------------------------------- |
| 主/次 CTA 按钮      | `transform 0.12s ease-in-out`       |
| 背景色变化          | `background-color 0.2s`             |
| 通用快速            | `0.15s`                             |
| 通用中速            | `0.4s ease-out`                     |
| 旋转/位移           | `transform 0.3s`                    |
| Material-style 平滑 | `0.3s cubic-bezier(0.4, 0, 0.2, 1)` |

### 8.2 Hover 效果规范

| 组件                     | Hover 效果                             |
| ------------------------ | -------------------------------------- |
| Primary/Secondary Button | `transform: scale(1.02)`               |
| Nav Link                 | `background-color: rgb(244, 239, 234)` |
| Text Link（正文内）      |                                        |