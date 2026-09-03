# todos 待办卡片原型（Design 阶段 UI 验证原型）

本原型用于 Design 阶段真实 UI 验证：表达 `todos` change 的全部用户可见交互与状态。**它不是业务实现**，数据使用 localStorage mock，无任何真实后端、Tauri 运行时或业务源码。

## 入口与真实预览方式

- 入口：`index.html`（页面即"模拟 macOS 桌面 + 待办卡片"）
- 预览：`python3 -m http.server 8701 --directory openspec/changes/todos/prototype`，浏览器打开 `http://127.0.0.1:8701/index.html`
- UI Evidence（截图与断言）：`screenshots/` 目录

## 覆盖与未覆盖范围

覆盖（设计阶段交互体验与状态表达验证）：
- 托盘图标点击切换卡片显示/隐藏（SCN-TRAY-002-A/B，模拟托盘图标按钮）
- 点击卡片外部区域自动隐藏（SCN-TRAY-003-A，模拟失焦）
- 输入框回车添加非空待办；空/纯空白输入回车不添加（SCN-TODO-001-A/B）
- 勾选完成：划线置灰、保留在列表；再次点击取消恢复（SCN-TODO-002-A/B）
- 悬停条目显示删除按钮（仅被悬停条目显示）；点击直接删除、无确认（SCN-TODO-003-A/B）
- 本地持久化模拟：localStorage 落盘，reload 后列表与完成状态恢复（SCN-TODO-004-A）
- 受限状态：错误横幅（模拟保存失败）

未覆盖（原型不表达，交付给 Build 实现）：
- 真实托盘/菜单栏、真实窗口系统的焦点事件（原型以点击模拟）
- 窗口定位算法（DEC-002：真实实现按鼠标所在屏幕边缘定位；原型简化固定居中）
- 真实持久化（Rust todo-store crate + `todos.json`）
- 单实例、打包产物、右键退出菜单、真实按键焦点语义（键盘可达性以 CSS `:focus-visible`/`:focus-within` 表达）

## Context Files

- `openspec/changes/todos/proposal.md`（Design Handoff）
- `openspec/changes/todos/specs/tray-card/spec.md`（CAP-001，REQ-TRAY-001~003）
- `openspec/changes/todos/specs/todo-management/spec.md`（CAP-002，REQ-TODO-001~004）
- `openspec/changes/todos/design.md`（本原型的事实来源）

## 实现依据的设计决策

- DEC-002（独立无边框小窗口；原型 mock 其形态）
- DEC-003（原生 HTML/CSS/JS ES Modules，零构建）
- DEC-006（隐藏语义：点击托盘切换/失焦关闭；原型模拟）
- DEC-007（失焦与托盘点击时序；原型验证用户可见结果，真实竞态守卫由 Rust 实现）
- DEC-010（完成态划线置灰、悬停删除按钮、错误横幅、全中文文案）
- INV-002/003/004/005/006/007（失败不改内存、空输入拒绝、追加顺序、删除不可撤销、完成保留、隐藏丢弃未提交输入）

## Mock 数据与真实数据源映射

| Mock（原型） | 真实数据源 / API（Apply/Build 实现） |
| --- | --- |
| `assets/mock-data.mjs`（localStorage key `trayTodo.prototype.v1`） | Rust `todo-store` crate + `app_data_dir/todos.json`（INT-004） |
| `store.add/toggle/remove/list` | Tauri commands `add_todo/toggle_todo/remove_todo/list_todos`（INT-001） |
| 模拟保存失败开关 | command 层持久化错误 → `Err(String)` → 前端错误横幅 |
| 托盘图标按钮（页面模拟） | `TrayIcon` 左键事件（INT-002） |
| 点击外部隐藏 | `WindowEvent::Focused(false)` 防抖复核（INT-003） |

Mock 数据结构与真实文件格式一致：`{ "version": 1, "items": [{ id, text, done, created_at }] }`。

## 文件职责

- `index.html` — 页面骨架：模拟桌面/菜单栏/托盘图标、卡片、错误横幅、原型工具条
- `assets/styles.css` — 全部样式（卡片 320×440、完成态划线置灰、悬停删除按钮、横幅动效）
- `assets/logic.mjs` — 前端纯逻辑（`isValidInput`/`formatCount`），与真实 `src/logic.mjs` 同一契约（INT-005/SEAM-002）
- `assets/mock-data.mjs` — mock 数据层：localStorage 持久化、原子提交语义（先持久化成功再切换内存）、模拟失败开关、原型调试钩子
- `assets/app.mjs` — 交互主逻辑：渲染、输入、勾选、删除、显隐模拟、错误横幅、验证钩子 `window.__pt`
- `tools/run-logic-check.mjs` — 前端纯逻辑契约快速检查（`node tools/run-logic-check.mjs`，SEAM-002 契约预验证）
- `tools/validate-ui.mjs` — CDP 真实 UI 验证驱动（`node tools/validate-ui.mjs`；重置原型数据并从空态完整跑通 20 项断言 + 截图，输出 UI Evidence）
- `screenshots/` — Design 阶段 UI Evidence 截图（由 UI 验证流程生成）

## Required Real Data / ENV

- 无 ENV、无账号、无网络；mock 数据在浏览器 localStorage 内自动初始化。
- 触发持久化错误演示使用原型工具条按钮（真实运行路径不存在该按钮）。

## 给 Build 的实现提示

1. 按 design.md 各 DEC/INT/INV/SEAM 实现，本原型仅是交互契约的静态表达，不得拷贝为业务代码。
2. 真实前端前端 `src/` 中复制 `logic.mjs` 契约（`isValidInput`/`formatCount`），用 `node --test src/logic.test.mjs` 跑 SEAM-002。
3. 真实点击"失焦关闭"由 `WindowEvent::Focused(false)` + DEC-007 竞态守卫实现；关闭时未提交输入丢弃（隐藏时清空输入框与错误横幅是原型采用的口径）。
4. 错误横幅的触发方为 command 失败（`Err(String)`），文案由后端返回中文描述。
5. 卡片显示时聚焦输入框（原型 `inputEl.focus()` 对应窗口 `show` + 前端 focus）。
6. 键盘可达性：删除按钮 `:hover`/`:focus-within` 显示，checkbox 用原生控件，均保留在真实实现中。