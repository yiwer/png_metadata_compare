以下是从 DOM、CSS 样式表、动画帧中直接提取并完整整理的 **MotherDuck 交互设计逻辑指南**，作为前一份设计指南的独立补充模块。

---

# MOTHERDUCK INTERACTION DESIGN GUIDE

## 1. 交互设计哲学总纲

MotherDuck 的交互设计遵循三条核心原则，Agent 在设计任何交互行为时均应以此为基准判断：

**物理感（Physicality）：** 所有按下/抬起动作均有轻微的位移反馈（`translate(7px, -7px)` 或 `translateY(-4px)`），配合 `box-shadow` 模拟立体偏移，营造"印章按压"的物理质感。

**克制的速度感（Restrained Speed）：** 几乎所有动效时长控制在 `0.1s–0.4s` 之间，无过度缓动。导航、表单反馈用 `0.15–0.2s`，内容进场用 `0.3–0.4s`，仅图片滑入等重场景用 `1s`。

**功能驱动（Function-Driven）：** 无装饰性动画。每个动效都有明确语义：进入 = `translateY` 向上 + `opacity 0→1`，退出 = 反向；内容切换 = `scale(0.8) + opacity` 交叉淡化；卡片状态切换 = `background-color`过渡。

---

## 2. 全量 Transition 规格表

以下所有 `transition` 值均从实际 DOM 计算样式中直接提取，按组件类型分类：

### 2.1 导航系统

| 组件                          | 触发条件       | Transition                                                   | 效果描述                                                     |
| ----------------------------- | -------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| 顶部导航栏                    | 页面滚动 > 0px | `border-bottom 0.2s ease-in-out, background-color 0.2s ease-in-out` | 滚动前 `border-bottom: transparent`，滚动后变为 `2px solid #383838` |
| 顶部 Banner 区域              | 滚动折叠       | `height 0.2s ease-in-out, margin-top 0.2s ease-in-out`       | 高度从 `55px` 收缩至 `0` 以折叠横幅                          |
| 导航下拉菜单箭头              | hover 展开     | `transform 0.2s ease-in-out`                                 | 箭头旋转（`rotate(180deg)`表示展开）                         |
| 导航子菜单链接 hover          | hover          | `background-color 0.2s`                                      | 背景变为 `rgb(241, 241, 241)`                                |
| 导航菜单栏展开/收起           | click          | `height 0.15s ease-in-out`                                   | 子菜单容器高度动态展开/关闭                                  |
| 导航子菜单面板滑出            | click          | `right 0.2s ease-in-out, opacity 0.5s ease-in-out`           | 从右侧滑入，同时淡入                                         |
| 导航内跨区块链接 hover 下划线 | hover          | `border-bottom` 即时                                         | `calc(0.09em + 1px) solid #6FC2FF` 蓝色下划线                |
| 导航内页脚区链接 hover        | hover          | 即时                                                         | `border-color: rgba(255,255,255,0.93)` 白色透明下划线        |

### 2.2 按钮系统

| 组件                                  | Transition                    | Hover 效果                                                  | Active/Press 效果                                            | Disabled 效果                                                |
| ------------------------------------- | ----------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| **Primary CTA（START FREE）**         | `transform 0.12s ease-in-out` | `transform: scale(1.02)`                                    | `transition: transform 30ms; transform: none`（快速回弹归位） | `color: #A1A1A1; background: #F8F8F7; border-color: #A1A1A1` |
| **Primary CTA（按钮外壳容器）**       | `all`                         | 触发内层按钮 `translate(7px, -7px)` 位移                    | `#styled-button-component:active { transform: none }`        | —                                                            |
| **Secondary CTA（BOOK A DEMO）**      | `transform 0.12s ease-in-out` | `transform: scale(1.02)`                                    | `transition: transform 30ms; transform: none`                | 同上                                                         |
| **Secondary Button（米白色变体）**    | `transform 0.12s ease-in-out` | `background-color: #F8F8F7; border-width: 2px`              | `background-color: rgb(225, 214, 203)`（暖灰按压色）         | 同上                                                         |
| **导航文字按钮（PRODUCT/COMMUNITY）** | 无过渡                        | span 子元素出现 `border-bottom: 0.09em + 1px solid #6FC2FF` | `color: #6FC2FF; border: 1px solid transparent`              | `border: 1px solid transparent; background: transparent`     |
| **Active 蓝色文字按钮**               | 无                            | hover = 蓝色下划线                                          | `color: #6FC2FF + 下划线 + path fill 变蓝`                   | —                                                            |
| **暗色区域链接（Footer等）**          | 无                            | `border-color: rgba(255,255,255,0.93)`                      | `color/fill: rgba(255,255,255,0.93)`                         | —                                                            |
| **Suggestion Chip 按钮**              | `0.15s`                       | `background-color: #FFDE00`（黄色填充）                     | `box-shadow: #383838 -1px 1px 0px 0px; transform: translate(-1px, 1px)` | —                                                            |
| **Submit 按钮（激活态）**             | 含 `transform 0.12s`          | hover = `#FFDE00` 填充                                      | `active: box-shadow: #383838 -2px 2px 0px; transform: translate(-2px, 2px)` | `background: #D7D7D7; color: #C0C0C0; cursor: not-allowed`   |

**CTA 双层容器交互模型（重要）：**
```
外层容器（dark bg #383838）
  └── #styled-button-component（内层蓝色按钮）
       hover on 外层 → 内层 translate(7px, -7px)  ← 视觉偏移制造"立体感"
       active on 外层 → 内层 transform: none       ← 按下归位
```

### 2.3 卡片系统

| 组件                                | Transition                          | Hover 效果                                                   | Active 效果                                                  |
| ----------------------------------- | ----------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| **Report Card（报告选择卡）**       | `0.4s ease-out`                     | `transform: translateY(-4px) scale(1.02); box-shadow: #383838 -4px 4px 0px 0px; border-color: #6FC2FF` | `transform: translateY(-2px) scale(1.01); box-shadow: #383838 -2px 2px 0px 0px` |
| **Duckling 定价卡**                 | `opacity 0.2s ease-in-out`          | 未选中卡的 `opacity: 0.5` → `1.0`（焦点高亮）                | —                                                            |
| **角色卡（Software Engineers 等）** | `background-color 0.3s, color 0.3s` | bg 变为对应颜色（Yellow/Teal/Blue）+ 黑色文字；`transform: scale(1.1) !important` | —                                                            |
| **Feature Card（导航下拉内容项）**  | `background-color 0.2s`             | `background-color: #F1F1F1`                                  | —                                                            |

**Report Card Hover 完整视觉逻辑：**
```
default: border 3px solid #383838 (dark), no shadow, no transform
hover:   border 3px solid #6FC2FF (蓝色!), translateY(-4px) scale(1.02),
         box-shadow: #383838 -4px 4px 0px 0px (左下偏移阴影，模拟立体浮起)
active:  translateY(-2px) scale(1.01), box-shadow缩小为 -2px 2px
```

### 2.4 表单系统

| 组件                     | Transition                  | 焦点状态                                                     | 禁用状态                                                   | Placeholder                        |
| ------------------------ | --------------------------- | ------------------------------------------------------------ | ---------------------------------------------------------- | ---------------------------------- |
| **AI Widget 文字输入框** | `box-shadow 0.15s`          | `box-shadow: #6FC2FF -3px 3px 0px 0px`（蓝色立体投影）       | `background: #F4EFEA; color: #C0C0C0; cursor: not-allowed` | `color: #C0C0C0`                   |
| **大型搜索框（Hero区）** | `0.2s ease-in-out, padding` | `outline: none; border: 2px solid #2BA5FF; padding-right: 24px` | —                                                          | `color: #A1A1A1; font-weight: 300` |
| **浮动标签输入框**       | Label: `0.15s ease-out`     | focus-within → label `font-size: 12px; top: -5px; transform: translateY(-100%); left: 24px; color: #383838; font-weight: 400` | —                                                          | —                                  |
| **Checkbox**             | 无                          | checked → 容器 `background-color: #383838`，内部 SVG path `fill: white` | —                                                          | —                                  |

**浮动标签（Floating Label）完整行为：**
```
未聚焦: label position:absolute, top:30px, left:24px, font-size:16px, color:#A1A1A1, pointerEvents:none
聚焦时: font-size:12px, top:-5px, transform:translateY(-100%), color:#383838 (through :focus-within on parent)
过渡:   0.15s ease-out (label), 0.2s ease-in-out (input padding)
```

### 2.5 图表/数据可视化

| 组件                   | Transition                                     | 效果                                             |
| ---------------------- | ---------------------------------------------- | ------------------------------------------------ |
| SVG 数据线/图形路径    | `0.1s ease-in-out`                             | 图表交互状态下 opacity/填充色过渡                |
| 图表幻灯片 enter       | `animation: idkxa 0.25s ease-in-out forwards`  | `opacity 0→1 + scale(0.8)→scale(1)` 渐显放大进入 |
| 图表幻灯片 exit        | `animation: jLKMiS 0.25s ease-in-out forwards` | `opacity 1→0 + scale(1)→scale(0.8)` 渐隐缩小退出 |
| 内容区 enter           | `animation: dRLQAb 0.4s ease-out`              | `opacity 0→1 + translateY(8px)→0`                |
| Suggestion Chips enter | `animation: dRLQAb 0.3s ease-out forwards`     | 同上，逐项依次进场                               |

### 2.6 Swiper 轮播（证言区）

| 属性             | 值                                           |
| ---------------- | -------------------------------------------- |
| 轮播切换         | `transition: transform`（由 Swiper.js 控制） |
| Slide 布局       | `display: flex`，`overflow: hidden`          |
| 活动幻灯片背景   | `rgb(255, 222, 0)`（黄色）                   |
| 非活动幻灯片背景 | `rgb(244, 239, 234)`（米色）                 |
| 切换方向         | 水平滑动 (`swiper-horizontal`)               |

---

## 3. 全量 Keyframes 动画库

以下是所有从 `@keyframes` 规则中提取的动画，并附带语义命名和使用场景：

### 3.1 内容进入动画

```css
/* [Enter From Bottom] 内容从下方淡入上浮 */
@keyframes contentEnter {   /* 原名: hCmOrq */
  0%   { transform: translateY(1rem); opacity: 0; }
  100% { transform: translateY(0rem); opacity: 1; }
}
/* 使用场景: 卡片、段落、模块内容进入页面 */
/* 使用规格: animation-duration: 0.4s, ease-out, fill: none */

/* [Enter Fade Up] 轻量上浮淡入 */
@keyframes contentSlideUp {  /* 原名: dRLQAb */
  0%   { opacity: 0; transform: translateY(8px); }
  100% { opacity: 1; transform: translateY(0px); }
}
/* 使用场景: AI Widget 内容区、Suggestion Chip 按钮逐项出现 */
/* 使用规格: duration 0.3–0.4s, ease-out, fill: forwards/none */

/* [Fade In Only] 纯淡入 */
@keyframes fadeIn {          /* 原名: jBcSpD */
  0%   { opacity: 0; }
  100% { opacity: 1; }
}
/* 使用场景: 浮层、overlay、弹出元素 */
/* 使用规格: 0.4s, ease-out, fill: none */
```

### 3.2 内容退出动画

```css
/* [Exit To Bottom] 内容向下淡出 */
@keyframes contentExit {     /* 原名: iBNjHa */
  0%   { transform: translateY(0rem); opacity: 1; }
  100% { transform: translateY(1rem); opacity: 0; }
}
/* 与 contentEnter 配对使用（如 tab 切换旧内容退出） */
```

### 3.3 图表 Slide 切换动画（进/出对）

```css
/* [Chart Slide Enter] 图表面板进入（scale+opacity） */
@keyframes chartSlideIn {    /* 原名: idkxa */
  0%    { opacity: 0; transform: translateY(calc(-50% + 1px)) scale(0.8); }
  49.9% { opacity: 0; }
  50%   { opacity: 1; }
  100%  { opacity: 1; transform: translateY(calc(-50% + 1px)) scale(1); }
}
/* 使用规格: 0.25s, ease-in-out, fill: forwards, position: absolute */

/* [Chart Slide Exit] 图表面板退出 */
@keyframes chartSlideOut {   /* 原名: jLKMiS */
  0%    { opacity: 1; transform: translateY(calc(-50% + 1px)) scale(1); }
  49.9% { opacity: 1; }
  50%   { opacity: 0; }
  100%  { opacity: 0; transform: translateY(calc(-50% + 1px)) scale(0.8); }
}
/* 注意: 进出动画在 50% 处做了硬切换（opacity 0↔1），避免中间帧闪烁 */
```

### 3.4 下拉菜单动画

```css
/* [Dropdown Reveal] 导航下拉菜单从上方滑出 */
@keyframes dropdownReveal {  /* 原名: bJnLpX */
  0%   { top: 100%; opacity: 0; }
  100% { top: calc(100% + 16px); opacity: 1; }
}
/* 使用规格: 0.2s, ease-out, fill: forwards, position: absolute, z-index: 3 */
/* 搭配: box-shadow: rgba(0,0,0,0.1) 0px 4px 8px（唯一使用阴影的组件！） */
```

### 3.5 Loading / 等待状态动画

```css
/* [Spin] 加载旋转 */
@keyframes spin {            /* 原名: eoUyJr */
  0%   { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
/* 使用场景: 加载指示器 spinner */

/* [Typing Dots] 打字省略号 */
@keyframes typingDots {      /* 原名: ddTMKC */
  0%, 20%   { content: ""; }
  40%        { content: "."; }
  60%        { content: ".."; }
  80%, 100% { content: "..."; }
}
/* 使用场景: AI 处理中等待状态文字提示（"processing..."） */
/* 通过 ::after 伪元素实现 */

/* [Opacity Flicker] 缓慢呼吸闪烁 */
@keyframes opacityFlicker {  /* 原名: jRLVqH */
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.6; }
}
/* 使用场景: 加载中的占位元素 */

/* [Scale Pulse] 尺度脉冲 */
@keyframes scalePulse {      /* 原名: jbaLYm */
  0%, 100% { opacity: 0.4; transform: scale(0.8); }
  50%       { opacity: 1; transform: scale(1); }
}
/* 使用场景: 活跃状态指示点/pulse 圆点 */

/* [Glow Pulse - Yellow] 黄色光晕脉冲 */
@keyframes yellowGlowPulse { /* 原名: hhVIz */
  0%, 100% { box-shadow: rgba(255, 213, 79, 0.7) 0px 0px 0px 0px; }
  50%       { box-shadow: rgba(255, 213, 79, 0) 0px 0px 8px 8px; }
}
/* 使用场景: 需要黄色高亮提示的交互元素（注意按钮/重点功能）*/

/* [Claude Pulse - Orange] AI 模式橙色内发光脉冲 */
@keyframes claudePulse {
  0%   { box-shadow: rgba(217,119,87,0.5) 0 0 10px inset, rgba(217,119,87,0.3) 0 0 20px inset, rgba(217,119,87,0.1) 0 0 30px inset; }
  50%  { box-shadow: rgba(217,119,87,0.7) 0 0 15px inset, rgba(217,119,87,0.4) 0 0 25px inset, rgba(217,119,87,0.2) 0 0 35px inset; }
  100% { box-shadow: rgba(217,119,87,0.5) 0 0 10px inset, ...; }
}
/* 使用场景: Claude AI 模式专属呼吸灯效果，infinite循环 */
/* 使用规格: 2s, ease-in-out, infinite */
```

---

## 4. 完整 Hover 状态规格

### 4.1 按钮 Hover 汇总（语义化描述）

```css
/* ① 主CTA 容器（深色外壳） → 内层按钮偏移 */
.cta-container:hover #styled-button-component {
  transform: translate(7px, -7px);  /* 右上角偏移，制造"弹出"感 */
}

/* ② 导航文字按钮 → 蓝色下划线 */
.nav-text-btn:hover span {
  border-bottom: calc(0.09em + 1px) solid rgb(111, 194, 255);
  margin-bottom: 0px;
}

/* ③ 导航文字按钮（暗色区域）→ 深色下划线 */
.nav-text-btn-dark:hover span {
  border-bottom: 0.09em solid rgb(56, 56, 56);
  margin-bottom: 1px;
}

/* ④ Report Card → 蓝边 + 上浮 + 左下阴影 */
.report-card:hover {
  transform: translateY(-4px) scale(1.02);
  box-shadow: rgb(56, 56, 56) -4px 4px 0px 0px;
  border-color: rgb(111, 194, 255);
}

/* ⑤ Suggestion Chip → 黄色填充 */
.suggestion-chip:hover {
  background-color: rgb(255, 222, 0);
}

/* ⑥ 角色卡 h3 标签 → 对应颜色复原 + scale */
.persona-card:hover {
  transform: scale(1.1) !important;
  opacity: 1 !important;
}
.persona-card.software:hover h3 { background-color: rgb(255, 222, 0); color: rgb(56, 56, 56); }
.persona-card.scientist:hover h3 { background-color: rgb(83, 219, 201); color: rgb(56, 56, 56); }
.persona-card.engineer:hover h3 { background-color: rgb(111, 194, 255); color: rgb(56, 56, 56); }

/* ⑦ 下拉菜单内容链接 → 灰色底色 */
.dropdown-link:hover {
  background-color: rgb(241, 241, 241);
}

/* ⑧ Footer 链接 → 深色图标填充 */
.footer-icon:hover path {
  fill: rgb(111, 194, 255);
}

/* ⑨ 元素隐藏提示文字 hover 显示 */
.has-tooltip:hover .tooltip-label {
  display: initial;  /* hover时才显示隐藏的标签文字 */
}

/* ⑩ 通用可交互元素减淡 */
.interactive-dim:hover {
  opacity: 0.6;
}

/* ⑪ 文字链接悬停下划线 */
.text-link:hover {
  text-decoration: underline;
}

/* ⑫ 加粗 hover（导航某些链接） */
.bold-on-hover:hover {
  font-weight: 600;
  letter-spacing: 0.01em;
}

/* ⑬ Input 聚焦边框变蓝 */
.input-hover:hover + div {
  border: 2px solid rgb(43, 165, 255);  /* 蓝色焦点框 */
}
```

### 4.2 Focus / Active / Disabled 状态规格

```css
/* === FOCUS 状态 === */

/* 标准输入框 focus */
input:focus {
  outline: none;
  border: 2px solid rgb(43, 165, 255);   /* #2BA5FF 亮蓝 */
  padding-right: 24px;                   /* 图标空间收缩 */
}

/* AI Widget 小输入框 focus（立体阴影风格） */
.ai-input:focus {
  box-shadow: rgb(111, 194, 255) -3px 3px 0px 0px;  /* 左下蓝色立体框 */
}

/* 浮动标签容器 focus-within（父级状态驱动子label动画） */
.floating-label-wrapper:focus-within .label {
  color: rgb(56, 56, 56);
  font-size: 12px;
  line-height: 140%;
  font-weight: 400;
  top: -5px;
  transform: translateY(-100%);
  /* 由 0.15s ease-out 过渡驱动 */
}

/* === ACTIVE / PRESS 状态 === */

/* CTA 主按钮 press（快速归位） */
.btn-primary:active {
  transition: transform 30ms ease-in-out;  /* 极短回弹 */
  transform: none;
}

/* CTA 容器内部按钮 press 归位 */
.cta-container #styled-button-component:active {
  transform: none;
}

/* Suggestion Chip press（左下微位移） */
.suggestion-chip:active {
  box-shadow: rgb(56, 56, 56) -1px 1px 0px 0px;
  transform: translate(-1px, 1px);
}

/* Submit 按钮 press（中等位移） */
.submit-btn:active:not(:disabled) {
  box-shadow: rgb(56, 56, 56) -2px 2px 0px 0px;
  transform: translate(-2px, 2px);
}

/* Report Card press（更小浮起） */
.report-card:active {
  transform: translateY(-2px) scale(1.01);
  box-shadow: rgb(56, 56, 56) -2px 2px 0px 0px;
}

/* 导航文字按钮 active */
.nav-text-btn:active {
  color: rgb(111, 194, 255);
  border: 1px solid transparent;
}
.nav-text-btn:active span {
  border-bottom: calc(0.09em + 1px) solid rgb(111, 194, 255);
}
.nav-text-btn:active path {
  fill: rgb(111, 194, 255);  /* SVG图标同步变蓝 */
}

/* Primary 按钮 active（变为亮蓝） */
.btn-primary:active {
  background-color: rgb(43, 165, 255);  /* #2BA5FF 更亮蓝 */
}

/* Secondary（米白）按钮 active */
.btn-secondary:active {
  background-color: rgb(225, 214, 203);  /* 暖灰按压色 */
}

/* === DISABLED 状态 === */

/* 通用禁用样式 */
[disabled] {
  color: rgb(161, 161, 161);           /* #A1A1A1 */
  background-color: rgb(248, 248, 247); /* #F8F8F7 */
  border-color: rgb(161, 161, 161);
  cursor: pointer;                      /* 注意：仍保留 pointer（非 not-allowed）*/
}

/* 图标按钮禁用 */
[disabled] path {
  fill: rgb(161, 161, 161);
}

/* 发送/提交按钮禁用（更强） */
.send-btn:disabled {
  background-color: rgb(215, 215, 215);  /* #D7D7D7 更深灰 */
  color: rgb(192, 192, 192);
  cursor: not-allowed;
}

/* 输入框禁用 */
input:disabled {
  background-color: rgb(244, 239, 234);  /* 米色背景 */
  color: rgb(192, 192, 192);
  cursor: not-allowed;
}

/* Checkbox checked 状态 */
input[type="checkbox"]:checked + div {
  background-color: rgb(56, 56, 56);     /* 深色填充 */
}
input[type="checkbox"]:checked + div svg path {
  fill: rgb(255, 255, 255);              /* 白色 ✓ 图标 */
}
```

---

## 5. 滚动驱动交互（Scroll Behaviors）

### 5.1 Navigation Scroll States

```
┌────────────────────────────────────────────────────┐
│ scrollY = 0 (顶部)                                   │
│  ├── 导航栏: border-bottom: 2px solid transparent   │
│  ├── Banner: height: 55px, marginTop: 0px           │
│  └── 总高度约 127px                                  │
├────────────────────────────────────────────────────┤
│ scrollY > 0 (滚动后)                                 │
│  ├── 导航栏: border-bottom: 2px solid #383838       │  ← 0.2s ease-in-out
│  ├── Banner: 保持显示 (height 不变)                  │
│  └── position: fixed, z-index: 99                   │
└────────────────────────────────────────────────────┘
```

### 5.2 图片视差进场（Scroll-Reveal Images）

```css
/* 图片在进入视口时触发 */
.scroll-reveal-img {
  transition: transform 1s, opacity 1s;
  /* 初始: opacity: 0, transform: translateY(20px) 或 scale(0.95) */
  /* 进入后: opacity: 1, transform: none */
}
```

### 5.3 Sticky 区块内容切换

角色卡（Persona Cards）使用 Intersection Observer 驱动，当卡片进入视口时 `opacity: 0.5 → 1.0`（`0.2s ease-in-out`），当前激活卡片保持 `opacity: 1`，其余降至 `0.5`。

---

## 6. Scrollbar 样式规格

### 6.1 全局 Scrollbar

```css
::-webkit-scrollbar { width: 5px; height: 5px; }
::-webkit-scrollbar-track { background: rgb(241, 241, 241); }  /* #F1F1F1 */
::-webkit-scrollbar-thumb { background: rgb(136, 136, 136); }  /* #888888 */
::-webkit-scrollbar-thumb:hover { background: rgb(85, 85, 85); } /* #555555 */
```

### 6.2 AI Widget 区域 Scrollbar（更窄，更浅）

```css
.ai-widget-scroll::-webkit-scrollbar { width: 6px; }
.ai-widget-scroll::-webkit-scrollbar-track { background: rgb(248, 248, 247); }  /* #F8F8F7 */
.ai-widget-scroll::-webkit-scrollbar-thumb {
  background: rgb(192, 192, 192);   /* #C0C0C0 */
  border-radius: 0px;               /* 保持锐角 */
}
.ai-widget-scroll::-webkit-scrollbar-thumb:hover {
  background: rgb(129, 129, 129);   /* #818181 */
}
```

---

## 7. 交互组件状态机总览

以下是各核心组件的完整状态机，Agent 在实现时可直接参考：

### 7.1 CTA 按钮状态机

```
default    → bg:#6FC2FF, border: 2px solid #383838, transform: none
hover      → transform: scale(1.02) [0.12s ease-in-out]
             (容器) inner-btn: translate(7px, -7px)
active     → transition: transform 30ms; transform: none (快速归位)
             bg: #2BA5FF (更亮蓝按压感)
             (容器) inner-btn: transform: none
disabled   → bg:#F8F8F7, color:#A1A1A1, border-color:#A1A1A1
focus      → 无特殊样式 (outline: none)
```

### 7.2 Report Card 状态机

```
default    → border: 3px solid #383838, transform:none, shadow:none
hover      → border-color→#6FC2FF, translateY(-4px) scale(1.02),
             box-shadow: #383838 -4px 4px 0 0 [0.4s ease-out]
active     → translateY(-2px) scale(1.01), box-shadow: -2px 2px
selected   → border: 3px solid #FFDE00 (当前选中卡片用黄色边框)
```

### 7.3 Input 状态机

```
default    → bg: white, border: 2px solid #383838, no shadow
hover      → border: 2px solid #2BA5FF (兄弟元素触发)
focus      → border: 2px solid #2BA5FF, box-shadow: #6FC2FF -3px 3px 0 0
             (大型) border: 2px solid #2BA5FF, outline: none
disabled   → bg: #F4EFEA, color: #C0C0C0, cursor: not-allowed
```

### 7.4 Suggestion Chip 状态机

```
enter      → animation: dRLQAb 0.3s ease-out forwards (translateY(8px)→0 + opacity 0→1)
default    → bg: white, border: 1px solid #383838
hover      → bg: #FFDE00 [0.15s]
active     → transform: translate(-1px, 1px), box-shadow: #383838 -1px 1px 0 0
```

### 7.5 Nav Dropdown 状态机

```
closed     → display: none / height: 0
open       → animation: bJnLpX 0.2s ease-out forwards
              top: 100%→calc(100%+16px), opacity: 0→1
              box-shadow: rgba(0,0,0,0.1) 0 4px 8px (唯一用阴影的场景!)
link-hover → background-color: #F1F1F1 [0.2s]
arrow-icon → transform: rotate(180deg) [0.2s ease-in-out]
```

---

## 8. 交互时序常量（Timing Constants）

Agent 在设计新交互时应优先从以下时序档中选取，保持全站一致：

| 时序常量              | 值         | 典型用途                  |
| --------------------- | ---------- | ------------------------- |
| `--duration-instant`  | `30ms`     | 按钮 press 归位（最短）   |
| `--duration-fast`     | `0.1s`     | SVG path 状态切换         |
| `--duration-swift`    | `0.12s`    | 主按钮 hover（scale）     |
| `--duration-quick`    | `0.15s`    | Chip hover/交互反馈       |
| `--duration-standard` | `0.2s`     | Nav、下拉、背景色         |
| `--duration-medium`   | `0.25s`    | 图表幻灯片切换            |
| `--duration-content`  | `0.3–0.4s` | 内容进场、Report Card     |
| `--duration-reveal`   | `0.5s`     | 菜单 opacity 淡出         |
| `--duration-slow`     | `1s`       | 图片视差/大型进场         |
| `--duration-ambient`  | `2s`       | AI 脉冲呼吸灯（infinite） |

| Easing 常量       | 值                                       | 用途                                |
| ----------------- | ---------------------------------------- | ----------------------------------- |
| `--ease-standard` | `ease-in-out`                            | 双向动作（展开/收起）               |
| `--ease-enter`    | `ease-out`                               | 进场（从慢到停）                    |
| `--ease-exit`     | `ease-in`                                | 退场（从停到快）                    |
| `--ease-snap`     | `ease-in-out`                            | 快速按钮反馈                        |
| `--ease-spring`   | `cubic-bezier(0.4, 0, 0.2, 1)`           | Material-style 弹性                 |
| `--ease-bounce`   | `linear(0 0%, 0.006 1.15%, ..., 1 100%)` | Spring 弹跳效果（HubSpot CTA 控件） |

---

## 9. 响应式媒体查询断点

从 CSS 中提取的完整断点列表：

```css
@media (max-width: 480px)  { /* 超小屏 */ }
@media (min-width: 728px)  { /* Tablet */ }
@media (min-width: 960px)  { /* Desktop */ }
@media (min-width: 1302px) { /* Large Desktop */ }
/* 注：还存在 max-width: 1301px 的反向断点用于限定某些区块 */
```

---

## 10. 完整交互场景案例代码

### 10.1 Report Card（完整交互状态）

```css
.report-card {
  background: #FFFFFF;
  border: 3px solid #383838;
  border-radius: 0px;
  transition: 0.4s ease-out;
  cursor: pointer;
  position: relative;