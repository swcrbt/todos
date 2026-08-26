# Design: tray-todo

## Design Inputs

- Proposal Revision / Digest: `proposal.md` Revision 1（2026-08-26，含 Design Handoff）
- Spec Revisions / Digests:
  - `specs/tray-card/spec.md`（CAP-001 托盘常驻与待办卡片，ADDED REQ-TRAY-001~003）
  - `specs/todo-management/spec.md`（CAP-002 待办列表管理，ADDED REQ-TODO-001~004）
- Covered CAP / REQ / SCN IDs:
  - CAP-001, CAP-002
  - REQ-TRAY-001, REQ-TRAY-002, REQ-TRAY-003
  - REQ-TODO-001, REQ-TODO-002, REQ-TODO-003, REQ-TODO-004
  - SCN-TRAY-001-A, SCN-TRAY-002-A, SCN-TRAY-002-B, SCN-TRAY-003-A
  - SCN-TODO-001-A, SCN-TODO-001-B, SCN-TODO-002-A, SCN-TODO-002-B, SCN-TODO-003-A, SCN-TODO-003-B, SCN-TODO-004-A
- Context Files Read:
  - `openspec/changes/tray-todo/proposal.md`
  - `openspec/changes/tray-todo/specs/tray-card/spec.md`
  - `openspec/changes/tray-todo/specs/todo-management/spec.md`
  - `openspec/changes/tray-todo/.openspec.yaml`
  - `openspec/schemas/engineering-spec-driven/schema.yaml`（链接目标，模板来源）
- Additional Sources Read:
  - `openspec status` / `openspec instructions design` 输出（change 状态、模板与指令）
  - 工作目录结构探查：`openspec/` 与 `.git/` 之外无任何业务代码（确认全新空项目）
  - 环境探查：node v25.9.0、cargo 1.95.0、python3 3.9.6、Chrome for Testing 缓存可用（用于验证命令设计）
- Design Revision: 1

## Context

### Confirmed Facts and Sources

| Fact ID | Fact | Source | Design 落地 |
| --- | --- | --- | --- |
| FACT-001 | 一期平台 macOS + Windows，架构预留 Linux | 需求结晶确认 | 全部选型采用 tauri 跨平台 API；不引入单平台锁定 API 形态 |
| FACT-002 | 技术栈 Tauri v2（Rust 后端 + Web 前端） | 需求结晶确认 | 本设计全部基于 tauri 2.x API（`tauri::tray`、`WindowEvent::Focused` 等） |
| FACT-003 | 点击托盘图标切换卡片显示/隐藏 | 需求结晶确认 | INT-002、DEC-002、DEC-007 |
| FACT-004 | 点击卡片外部区域（失焦）自动关闭 | 需求结晶确认 | INT-003、DEC-006、DEC-007 |
| FACT-005 | 输入框回车添加、空输入不添加 | 需求结晶确认 | REQ-TODO-001 语义；前后端双重校验（INV-003） |
| FACT-006 | 勾选完成划线置灰保留、可取消 | 需求结晶确认 | done 状态建模 + CSS 样式 + toggle 语义（INV-006） |
| FACT-007 | 悬停显示删除按钮、直接删除无确认无撤回 | 需求结晶确认 | hover 显示 + 点击即删（INV-005） |
| FACT-008 | 本地持久化，重启恢复 | 需求结晶确认 | MOD-001 原子持久化 + 启动恢复（INV-002、SCN-TODO-004-A） |
| FACT-009 | 中文界面文案，一期不做多语言 | 需求结晶确认 | UI/UX 部分全部中文文案 |
| HYP-001 | 单实例运行（已确认） | 用户 2026-08-26 确认 | DEC-004：采用 `tauri-plugin-single-instance` |
| HYP-002 | Rust 数据层可作为独立单元测试接缝 | open，待本设计验证 | DEC-001/MOD-001：todo-store 独立 crate、零 tauri 依赖，`cargo test -p todo-store` 可在 Tauri 运行时之外运行 |

### Constraints

- 技术栈固定为 Tauri v2，不得推翻。
- 一期平台范围固定为 macOS + Windows；任何模块接口不得锁定单平台（Linux 预留）。
- FACT-003~FACT-009 的用户可见交互为需求约束，不得静默改变；样式、动效、布局等交互细节由本设计决定。
- 不引入任何需要账号、联网或付费的外部服务；零网络权限。
- 全新空项目：无既有代码、无既有数据、无兼容承诺；本地数据格式由本设计决定（一期不承诺外部可读）。
- 不引入 Tauri 官方插件之外的第三方插件（单实例插件为官方插件）。

### Existing Architecture and Affected Areas

- 当前状态：仓库仅包含 `openspec/`（OpenSpec 元数据与 schema 链接）与 `.git/`；无 `src-tauri/`、无前端目录、无 `package.json`。本 change 是第一期，全部代码从零新建。
- 受影响路径（全部新建）：
  - `src-tauri/`：Rust 后端（托盘、卡片窗口、commands、数据层），`tauri.conf.json`、`Cargo.toml`、图标资源。
  - `src/`：Web 前端（零构建原生 ES Modules）。
  - 用户数据目录：`app_data_dir/todos.json`（首次运行创建）。
- 现状行为：无（本项目无既有行为可观察）。

## Goals / Non-Goals

**Goals:**

- 确定可实施、可验证的完整技术方案：托盘常驻、卡片窗口显隐/失焦关闭、增删勾选、原子持久化、单实例、跨平台打包。
- 数据层与 Tauri 运行时解耦（HYP-002 落地），使核心业务逻辑可无缝单元测试。
- 定义明确的前后端接口（INT-*）、不变量（INV-*）、测试接缝（SEAM-*）与决策记录（DEC-*）。
- 通过静态交互 prototype 与真实 UI 验证，在设计阶段确认全部用户可见交互与状态表达（UI Evidence）。

**Non-Goals:**

- 不做数据同步、云服务、账号与多设备。
- 不做提醒、通知、日程、到期日。
- 不做多列表、分类、标签、搜索、排序或"清除已完成"。
- 不做待办文本编辑；删除后只能重新输入。
- 不做 Linux 一期交付（仅确保架构不阻塞）。
- 不做多语言切换、设置界面、主题/图标定制。
- 不做窗口置顶、多窗口、常驻贴边卡片。
- 不引入 Web 前端构建链（无 Vite/React/Vue/package.json）、不引入数据库（rusqlite 等）、不引入网络/日志第三方插件。
- 卡片内不提供键盘快捷键（如 Esc 关闭）——非需求，显式排除。

## Architecture and Data Flow

```
┌────────────────────────── Tauri v2 应用（单实例进程）──────────────────────────┐
│  main.rs / lib.rs (MOD-002 app bootstrap)                                       │
│   ├─ 初始化：single-instance 插件 → 托盘（MOD-003）→ 卡片窗口（MOD-004）          │
│   ├─ 注册 commands（MOD-005）→ 注入 Manager<TodoStore>（MOD-001）                │
│   └─ macOS ActivationPolicy::Accessory（无 Dock 图标，仅菜单栏常驻）               │
│                                                                                  │
│  托盘图标(TrayIcon) ──Left Click──▶ MOD-003: toggle 指令                          │
│        │                    ┌────▶ MOD-004: show/focus 或 hide（含定位与竞态守卫）  │
│        └──Right Click──▶ 原生菜单【退出】                                         │
│                                                                                  │
│  卡片窗口(WkWebView/WebView2) <──(J)──┬── blur(Focused(false)) ──▶ 延时检查后 hide │
│      │                                │    （DEC-007 竞态守卫）                     │
│      └── invoke ──▶ MOD-005 commands ──▶ MOD-001 TodoStore（原子落盘）            │
│                                                        │                         │
│  系统路径: app_data_dir/todos.json（FACT-008 持久化） ◀── tmp+rename 原子写 ───────┘
└───────────────────────────────────────────────────────────────────────────────┘

前端（src/，零构建 ES Modules，MOD-006）
  index.html ← style.css ← logic.mjs(纯函数) ← app.mjs（invoke 调用、DOM 渲染）
  logic.mjs 供 Node 侧 node --test 直接 import（SEAM-002），浏览器内复用同一文件。
```

数据流（用户视角）：
1. 启动 → 单实例插件确认唯一进程 → 创建托盘图标 → 创建卡片窗口（初始 hidden）→ 从 `todos.json` 加载列表到内存。
2. 点击托盘图标 → 左键事件 → 若 hidden：定位并 show + focus（前端自动聚焦输入框）；若 visible：hide。
3. 卡片显示中点击外部任意区域 → 窗口 Focused(false) → 防抖（约 120ms）后复核焦点与托盘点击守卫 → hide。未提交的输入内容随窗口隐藏丢弃（不产生条目、不持久化）。
4. 输入框输入 + 回车 → 前端验证 trim 非空 → `add_todo` → TodoStore 追加到队尾 → 原子写盘 → 返回条目 → 前端清空输入框并追加渲染。
5. 勾选/取消 → `toggle_todo(id)` → 内存置反 done → 原子写盘 → 前端更新样式（划线置灰 / 恢复）。
6. 悬停条目显示删除按钮 → 点击 → `remove_todo(id)` → 内存移除 → 原子写盘 → 前端删除该条目。
7. 任何 command 失败（读写异常）→ 前端顶部错误横幅（中文文案，约 3 秒自动消失）→ 界面保持原状，不产生半更新状态。
8. 托盘右键菜单"退出" → 正常退出进程，最后一次成功写盘的数据即为重启后恢复的数据。

## Module Boundaries

| Module ID | Owner / Module | Responsibility | Must Not Own | Dependency Direction |
| --- | --- | --- | --- | --- |
| MOD-001 | `src-tauri/crates/todo-store`（独立 crate） | 待办数据模型（`TodoItem`、`TodoStore`）、list/add/toggle/remove、原子持久化、损坏文件恢复；零 tauri 依赖 | 任何 Tauri API、托盘、窗口、前端渲染 | 仅依赖 std/serde/serde_json/uuid/thiserror（可选） |
| MOD-002 | `src-tauri/src/lib.rs` + `main.rs` | 应用生命周期、单实例插件初始化、`AppState<TodoStore>` 管理、托盘与窗口装配、command 注册、macOS ActivationPolicy 设置 | 业务规则、持久化细节 | 依赖 MOD-001/003/004/005 |
| MOD-003 | `src-tauri/src/tray.rs` | 托盘图标创建（常驻）、左键点击 → toggle 指令、右键原生菜单（含"退出"） | 窗口定位、焦点逻辑 | 只发指令给 MOD-004；不直接操作数据 |
| MOD-004 | `src-tauri/src/card.rs` | 卡片窗口创建（无边框、固定尺寸、初始 hidden）、显示前定位（靠近托盘/光标所在屏幕边缘）、show/hide/toggle、失焦自动隐藏（含竞态守卫）、二次启动时唤起 | 数据读写、渲染内容 | 只做窗口生命周期；不依赖 MOD-001 |
| MOD-005 | `src-tauri/src/commands.rs` | 薄 command 层：`list_todos` / `add_todo` / `toggle_todo` / `remove_todo`，参数校验（trim 非空）、错误到中文可读字符串映射 | 托盘/窗口逻辑、持久化实现 | 只依赖 MOD-001 |
| MOD-006 | `src/`（Web 前端） | 卡片 UI 渲染与交互：输入提交、勾选切换、悬停删除按钮、空状态/错误横幅、调用 invoke、窗口 show 后聚焦输入框 | 数据持久化、跨进程任何状态 | 仅经 MOD-005 的 command API 访问数据 |

依赖方向：MOD-006 → MOD-005 → MOD-001；MOD-002 聚合 MOD-003/004/005；MOD-003 → MOD-004 单向指令；MOD-001 与 MOD-003/004/006 互不依赖。

## Interfaces

### INT-001 — Tauri 前端调用命令（Tauri command API）

- Callers: MOD-006 前端（`window.__TAURI__.core.invoke`，开启 `app.withGlobalTauri: true`）
- Inputs:
  - `list_todos() -> Vec<TodoItemDto>`
  - `add_todo(text: String) -> TodoItemDto`（text 经 trim；空文本返回错误）
  - `toggle_todo(id: String) -> TodoItemDto`（返回切换后的条目，供前端幂等渲染）
  - `remove_todo(id: String) -> ()`
  - `TodoItemDto { id: String(uuid), text: String, done: bool, created_at: i64 }`
- Outputs: serde JSON 结构；失败返回 `Err(String)`（中文可读，如"保存失败：写入磁盘出错"）
- Ordering: 相同 id 的 toggle/remove 由前端串行调用即可（一期单用户、单击互斥，无需队列）
- Error Semantics: 读取失败（启动期）→ 损坏恢复流程（INV-002）；写入失败 → command 返回 Err → 前端错误横幅；业务错误（空文本、id 不存在）→ `NotFound`/`InvalidInput` 映射为中文 Err
- Compatibility: 一期仅本前端调用；字段即数据文件字段子集
- Covers: REQ-TODO-001, REQ-TODO-002, REQ-TODO-003, REQ-TODO-004；SCN-TODO-001-A/B, SCN-TODO-002-A/B, SCN-TODO-003-A/B, SCN-TODO-004-A

### INT-002 — 托盘左键点击 → 卡片显隐切换

- Callers: MOD-003 托盘事件回调
- Inputs: `TrayIconEvent::Click { button: Left, .. }`
- Outputs: 向 MOD-004 发出 `ToggleCard` 指令（直接调用 `card.toggle()`）
- Ordering: 与 INT-003（blur）存在并发时序，按 DEC-007 守卫处理
- Error Semantics: 窗口不存在时重建（不常见路径）；事件吞掉不 panic
- Compatibility: macOS 菜单栏左键、Windows 托盘左键均触发；右键走原生菜单（DEC-011）
- Covers: REQ-TRAY-001, REQ-TRAY-002；SCN-TRAY-001-A, SCN-TRAY-002-A/B

### INT-003 — 窗口失焦 → 自动隐藏

- Callers: MOD-004 窗口事件回调（`WindowEvent::Focused(false)`）
- Inputs: 窗口焦点丢失事件
- Outputs: 防抖延时后复核 `window.is_focused()` 与"最近托盘点击时间戳"，仍失焦且非托盘点击窗口期 → `window.hide()`
- Ordering: 显隐状态为单写者（MOD-004 内部串行）
- Error Semantics: 复核/隐藏失败仅记录 stderr，不影响进程
- Compatibility: macOS/Windows 均由 WebView 窗口焦点事件触发；系统"点击外部"语义由 OS 保证
- Covers: REQ-TRAY-003；SCN-TRAY-003-A

### INT-004 — TodoStore（Rust 内部 API，MOD-001 暴露）

- Callers: MOD-005 commands；MOD-002 启动加载
- Inputs:
  - `TodoStore::load(path) -> Result<TodoStore>`（文件不存在 → 空库；损坏 → 备份并空库）
  - `list() -> Vec<TodoItem>`
  - `add(text: String) -> Result<TodoItem>`（trim 后为空 → `InvalidInput`）
  - `toggle(id) -> Result<TodoItem>`（不存在 → `NotFound`）
  - `remove(id) -> Result<()>`（不存在 → `NotFound`）
  - 内部：每次变更成功后同步 `persist()`：`todos.json.tmp` 写入 → `fs::rename` 原子替换
- Outputs: Result 类型 + 变更后的完整列表可经 `list()` 再次读取或直接返回被变更条目
- Ordering: 单实例进程内单写者；add 追加队尾（顺序 = 添加顺序，INV-004）
- Error Semantics: 磁盘错误 → `IoError`；JSON 结构错误 → `CorruptData`（触发备份恢复）
- Compatibility: 一期数据格式 `{ "version": 1, "items": [...] }`，version 字段预留给未来迁移
- Covers: REQ-TODO-001, REQ-TODO-002, REQ-TODO-003, REQ-TODO-004；SCN-TODO-001-A/B, SCN-TODO-002-A/B, SCN-TODO-003-B, SCN-TODO-004-A

### INT-005 — 前端纯逻辑导出（MOD-006 内）

- Callers: `app.mjs`（浏览器）+ Node 测试运行器（SEAM-002）
- Inputs: `isValidInput(text: string) -> boolean`（`text.trim().length > 0`）
- Outputs: boolean；`formatCount(n)` 供计数徽标文案（如"共 3 项"）
- Ordering: 无状态、纯函数
- Error Semantics: 不入参为 null/undefined 时返回 false（防御）
- Compatibility: ES Module（`.mjs`），浏览器 `<script type="module">` 与 `node --test` 双端可加载
- Covers: REQ-TODO-001；SCN-TODO-001-A, SCN-TODO-001-B

## State Model

| State | Event | Guard | Next State | Observable Result |
| --- | --- | --- | --- | --- |
| 窗口 hidden | 托盘左键点击 | 无 | visible | 卡片出现，位于托盘/鼠标所在屏幕边缘，输入框聚焦 |
| 窗口 visible | 托盘左键点击 | 无 | hidden | 卡片不可见，进程常驻托盘 |
| 窗口 visible | 外部点击（失焦） | 防抖复核仍失焦且非托盘点击窗口期（DEC-007） | hidden | 卡片自动隐藏；未提交输入丢弃 |
| 条目 pending | `toggle_todo(id)` | 条目存在 | completed | 划线置灰，保留在列表 |
| 条目 completed | `toggle_todo(id)` | 条目存在 | pending | 恢复正常样式 |
| 条目 任意 | `remove_todo(id)` | 条目存在 | 不存在 | 从列表消失，不可撤回 |
| 条目 任意 | 变更成功 | persist 成功 | 已持久化 | 重启后该变更仍在 |
| 变更后 | persist 失败 | 写盘异常 | 内存状态不变/回滚 | 错误横幅提示，界面不显示半更新 |
| 启动 | 加载 `todos.json` | 文件缺失 | 空库 | 空状态文案 |
| 启动 | 加载 `todos.json` | 文件损坏/版本不符 | 备份损坏文件 + 空库 | 以空列表启动，stderr 记录 |

## Invariants

| Invariant ID | Invariant | Enforcement Boundary | Failure Behavior | Covers |
| --- | --- | --- | --- | --- |
| INV-001 | 任意时刻最多一个应用进程、一个托盘图标、一份数据文件写入者 | MOD-002 单实例插件（DEC-004） | 插件初始化失败 → 拒绝启动并报错退出 | REQ-TRAY-001；SCN-TRAY-001-A |
| INV-002 | 用户可见的每次增/勾/删变更，在 command 返回成功前必须已原子落盘（tmp+rename） | MOD-001 persist | 写盘失败 → command 返回 Err，前端错误横幅，内存状态不承诺持久 | REQ-TODO-003, REQ-TODO-004；SCN-TODO-003-B, SCN-TODO-004-A |
| INV-003 | 空/纯空白文本不产生条目（前端与 store 双重校验，行为一致） | MOD-006 + MOD-001 | 后端校验兜底返回 `InvalidInput`；前端不发起调用 | REQ-TODO-001；SCN-TODO-001-A/B |
| INV-004 | 列表顺序 = 添加顺序；新条目追加队尾；无排序功能 | MOD-001 add 实现 | 不可违反（无排序入口） | REQ-TODO-001；SCN-TODO-001-A |
| INV-005 | 删除立即生效、不可撤回、无确认弹窗；完成与未完成条目均可删除 | MOD-006 交互 + MOD-001 remove | 无撤销路径；一次误删需重新输入 | REQ-TODO-003；SCN-TODO-003-A/B |
| INV-006 | completed 条目保留在列表中（不消失、不被清除），可取消勾选、可删除 | MOD-006 渲染 + MOD-001 状态 | 渲染层必须为 done 单独样式分支 | REQ-TODO-002；SCN-TODO-002-A/B |
| INV-007 | 失焦隐藏不产生条目（未提交输入丢弃），不丢失已持久化列表 | MOD-004 blur 流程 + 前端不自动提交 | 输入未回车即隐藏 = 该输入不触发任何 command | REQ-TRAY-003；SCN-TRAY-003-A |
| INV-008 | 卡片窗口全局唯一（同一个 WebView 实例 show/hide，不重建） | MOD-004 单例持有 | 事件回调在窗口未创建时先创建再处理 | REQ-TRAY-002；SCN-TRAY-002-A/B |
| INV-009 | 中文文本在持久化/重启往返后无乱码（UTF-8） | MOD-001 JSON 编解码 | JSON 序列化默认 UTF-8；测试覆盖中文往返 | REQ-TODO-004；SCN-TODO-004-A |

## External Dependencies and Adapters

| External ID | Service / Data | Adapter | Auth / ENV | Limits | Failure Behavior |
| --- | --- | --- | --- | --- | --- |
| EXT-001 | `tauri-plugin-single-instance`（官方插件） | 插件初始化回调（二次启动 → 唤起/显示窗口） | 无 | Windows/Linux 原生机制；macOS 依赖其自带机制，需真实环境验证 | 初始化失败 → 启动报错退出（不产生双实例即可接受） |
| EXT-002 | 系统 WebView：macOS WKWebView / Windows WebView2 | Tauri WebviewWindow | 无 | Windows 需 WebView2 运行时（打包时启用 bootstrapper） | 缺失 → 启动失败；tauri 错误提示 |
| EXT-003 | 平台应用数据目录 `app_data_dir` | `app.path().app_data_dir()` | 用户数据目录权限 | 单机单用户 | 目录创建失败 → command 层错误映射为中文提示 |
| EXT-004 | `serde` / `serde_json` / `uuid`（Rust crate，非 tauri 依赖） | todo-store 直接使用 | 无 | serde_json 无外部限制 | 反序列化错误 → `CorruptData` 恢复流程 |
| EXT-005 | 系统托盘/菜单栏能力 | `tauri::tray::TrayIconBuilder` | macOS 需 Accessory 激活策略 | 左键事件 macOS/Windows 均支持；右键差异见 DEC-011 | 图标创建失败 → 启动失败（报错退出） |
| EXT-006 | 系统打包工具（bundle）：dmg / nsis | `tauri build` bundler | 无 | macOS 需 codesign 之选配（未配置开发签名时可跳过） | 打包失败见 tauri 日志，不影响 `tauri dev` |

## Test Seams and Verification Strategy

| Seam ID | Public Seam | Scenarios | Test Level | Command | Expected Evidence |
| --- | --- | --- | --- | --- | --- |
| SEAM-001 | `todo-store` crate 公开 API（MOD-001，零 tauri 依赖，HYP-002 落地） | SCN-TODO-001-A/B（add/空文本拒绝/追加顺序）、SCN-TODO-002-A/B（toggle 往返）、SCN-TODO-003-B（remove 后持久化且不可撤销）、SCN-TODO-004-A（load/persist 往返、中文 UTF-8、损坏文件恢复、原子写后文件完整） | unit | `cargo test -p todo-store`（从 `src-tauri/` 目录运行；无需 Tauri 运行时/系统 WebView） | 全部 pass；输出显示 add/toggle/remove/persist 往返、中文文本、损坏恢复用例通过 |
| SEAM-002 | 前端纯逻辑 `logic.mjs`（isValidInput / formatCount） | SCN-TODO-001-B（空/纯空白）；SCN-TODO-001-A（非空 trim 后） | unit | `node --test src/logic.test.mjs`（Node 25 内置 test runner，零依赖） | 全部 pass；空字符串/空格/Tab 返回 false，非空返回 true |
| SEAM-003 | macOS 本机手动 UI 验收（托盘显隐、失焦、增删勾选、重启恢复） | 全部 SCN | manual/UI | `cargo tauri dev` + 验收清单（Apply 阶段执行） | 每项验收结果截图/记录，覆盖 SUCCESS-001~006 |
| SEAM-004 | Windows 真实验证（DEF-005 落地方案：优先 GitHub Actions `windows-latest` runner；候选远程 Windows 环境） | 全部 SCN（Windows 平台行为） | manual/CI | `tauri build` on windows-latest + 运行构建产物执行验收清单 | Windows 上托盘/失焦/勾选/删除/重启恢复通过 |
| SEAM-005 | 静态 prototype（本 change 已完成；Design 阶段交互验证） | SCN-TRAY-002-A/B、SCN-TRAY-003-A、SCN-TODO-001-A/B、002-A/B、003-A/B、004-A | UI（mock 数据） | `python3 -m http.server <port>` + Chrome CDP 驱动（见 Design Validation Evidence） | UI Evidence 全文：交互前后状态、DOM 证据、截图、console 无错误 |

## Decisions

### DEC-001 — 数据层：JSON 文件 + serde + 原子写（独立 crate）

- Status: accepted
- Drivers and Constraints: FACT-008 本地持久化；HYP-002 要求数据层可脱离 Tauri 运行环境测试；拒绝数据库类重型依赖（Non-Goals）；一期无复杂查询需求
- Covers: REQ-TODO-001~004；SCN-TODO-001-A/B、002-A/B、003-B、004-A
- Option A: `todo-store` 独立 crate：serde 结构体 + `todos.json`（version 字段）+ 临时文件写入后 `fs::rename` 原子替换；依赖仅 std/serde/serde_json/uuid
- Option B: `rusqlite`（SQLite）：查询能力强但引入原生依赖、初始化与 schema 管理，远超一期需求
- Option C: `tauri-plugin-store`：官方键值包装，但把持久化耦合进 Tauri 运行环境，无法满足 HYP-002 的独立单测接缝
- Decision: Option A
- Rationale: 条目量小（一期数百条），JSON 读写微秒级；独立 crate 无 tauri 依赖，`cargo test -p todo-store` 直接运行，HYP-002 从"待验证"变为"结构已保证"。
- Consequences / Trade-offs: 无数据库的复杂查询/索引能力；全量重写文件（数据量小时无感）；需要自己保证原子写与损坏恢复（已由 INV-002 与启动恢复流程覆盖）
- Reversal Trigger: 出现跨列表/搜索/大规模数据需求，或持久化性能成为瓶颈时，迁移到 SQLite 并保留 TodoStore 接口（MOD-001 作为唯一适配面）

### DEC-002 — 弹出卡片：独立无边框小窗口（非原生菜单）

- Status: accepted
- Drivers and Constraints: FACT-003/004 要求"卡片"式弹出与失焦关闭；需求要求自定义中文布局与悬停交互；SCN-TRAY-002-A 要求"位置靠近托盘图标"
- Covers: REQ-TRAY-002, REQ-TRAY-003；SCN-TRAY-002-A/B、SCN-TRAY-003-A
- Option A: 独立无边框窗口（`decorations: false`，固定尺寸 320×440，初始 hidden）+ show/hide 切换 + Focused(false) 自动隐藏
- Option B: TrayIcon 原生 menu（系统原生菜单呈现待办列表）——样式不可控、"卡片"体验差、动态增删与勾选交互受限
- Option C: menu + 窗口组合——只负责"入口菜单"，卡片主体仍需窗口，组合复杂度高于收益
- Decision: Option A；定位策略：显示前取鼠标所在显示器（`available_monitors` + `Monitor` 几何），macOS 置于菜单栏下方（屏幕工作区顶部 + 8px），Windows 置于任务栏上方（工作区底部 - 高度 - 8px），水平方向对齐鼠标 x 并夹紧于工作区内（TrayIcon 位置 API 在 macOS 不可用，故用鼠标坐标近似"靠近托盘图标"）
- Rationale: 原生菜单无法表达划线置灰、悬停删除按钮等需求指定交互；窗口方案完全可控且是 tauri 最常规做法
- Consequences / Trade-offs: 需处理窗口生命周期与失焦时序（DEC-006/007）；固定尺寸卡片无响应式需求
- Reversal Trigger: 需求出现"贴边常驻""多窗口"等新交互时重新设计窗口层，而非更换本方案

### DEC-003 — 前端：原生 HTML/CSS/JS ES Modules（零构建）

- Status: accepted
- Drivers and Constraints: 一期界面为单卡片单视图，交互五类（输入/勾选/删除/显隐/错误提示）；Non-Goals 排除构建链；依赖最少化
- Covers: 全部 REQ/SCN（UI 相关）
- Option A: 原生 HTML/CSS/JS，`<script type="module">`，`frontendDist: "../src"`，无 package.json、无 dev server
- Option B: Vite + React/Vue：组件化与 HMR，但引入 node 构建链与依赖树，与一期规模不匹配
- Decision: Option A
- Rationale: 功能极简、团队为个人开发者；零构建使 `cargo tauri dev` 直接加载静态目录，无 node 依赖；纯逻辑抽到 `logic.mjs` 仍可 Node 单测（SEAM-002）
- Consequences / Trade-offs: UI 复杂度上升后需要重构成 B 方案（界面模块可平移：渲染层换框架，logic.mjs 与 command 契约不变）
- Reversal Trigger: 界面增长到多视图/复杂组件树时切换 B 方案

### DEC-004 — 单实例：`tauri-plugin-single-instance`（官方插件）

- Status: accepted
- Drivers and Constraints: HYP-001 已确认（重复启动只保留一个托盘图标与一份数据）；INV-001
- Covers: REQ-TRAY-001；SCN-TRAY-001-A
- Option A: `tauri-plugin-single-instance`：官方、跨平台（Windows/Linux 原生互斥机制，macOS 自带机制），二次启动回调可"唤起"既有进程（显示卡片）
- Option B: 手写锁：锁文件 + PID 校验（平台行为差异大、易留下僵尸锁），或平台互斥量（单平台 API 与 FACT-001 Linux 预留冲突）
- Decision: Option A
- Rationale: 官方维护、跨平台、行为一致；二次启动回调中唤起窗口提升了单实例 UX（用户再点应用看到卡片）
- Consequences / Trade-offs: 依赖官方插件行为（macOS 细节需 Apply 阶段真机验证，RISK-006）
- Reversal Trigger: 插件在目标平台出现缺陷或行为不符时，降级为平台锁方案（保持 MOD-002 单点封装）

### DEC-005 — 数据路径：`app.path().app_data_dir()`

- Status: accepted
- Drivers and Constraints: FACT-008；proposal"路径由 Design 决定（候选：平台标准应用数据目录）"
- Covers: REQ-TODO-004
- Option A: tauri path API `app_data_dir`：macOS `~/Library/Application Support/<identifier>`、Windows `%APPDATA%\<identifier>`，随 `identifier` 配置自动正确
- Option B: 手写跨平台目录推导（如 dirs crate 或按 OS 分支）——重复实现、易错、且绕开 tauri 配置
- Decision: Option A
- Rationale: 零成本获得平台标准位置；identifier 在 `tauri.conf.json` 统一定义
- Consequences / Trade-offs: 与 tauri 配置绑定（identifier 变更会改变数据目录，需在 identifier 定型后不再变更）
- Reversal Trigger: 无（该 API 是 tauri 官方推荐）

### DEC-006 — 隐藏语义：close 请求 → hide；macOS Accessory；退出走托盘菜单

- Status: accepted
- Drivers and Constraints: FACT-003/004 要求"隐藏而非退出"；窗口无边框无系统关闭按钮；需保留唯一 WebView 实例（INV-008）
- Covers: REQ-TRAY-001, REQ-TRAY-002, REQ-TRAY-003
- Option A: 拦截 `CloseRequested` → `prevent_close + hide`；窗口创建一次（hidden 启动），show/hide 复用；macOS `ActivationPolicy::Accessory`（无 Dock 图标）
- Option B: 每次关闭销毁窗口、每次显示重建——重建导致状态与焦点受损、闪烁，丢 WebView 缓存
- Decision: Option A
- Rationale: show/hide 切换最接近"卡片弹层"语义；Accessory 与"托盘常驻、无主窗口"体验一致（菜单栏图标是唯一入口）
- Consequences / Trade-offs: 退出路径必须显式提供：托盘右键原生菜单【退出】（本决策涉及内容，见 DEC-010 交互细节中"退出路径"）——Windows 托盘区域右键菜单为平台惯例、macOS 菜单栏图标右键触发；进程正常退出由该菜单项触发
- Reversal Trigger: 出现"主窗口"类需求时调整

### DEC-007 — 失焦自动隐藏：防抖复核 + 托盘点击竞争守卫

- Status: accepted
- Drivers and Constraints: SCN-TRAY-003-A（点外部即隐藏）；FACT-003 点击托盘切换；两个事件的真实时序存在竞争：卡片可见时点击托盘图标，macOS/Windows 上可能先触发窗口失焦（blur）再触发托盘 Click，若 blur 直接 hide 会把"点击托盘想关闭"变成"关闭后又打开"或"想打开却隐藏"
- Covers: REQ-TRAY-002, REQ-TRAY-003；SCN-TRAY-002-A/B、SCN-TRAY-003-A
- Option A: 竞态守卫 + 防抖：托盘左键事件处理器在 `/` 前记录 `last_tray_click = now`；blur 处理器等待约 120ms 后复核 `window.is_focused()` 且距 `last_tray_click` 超过窗口期，才执行 hide；toggle 内部以窗口当前可见性为准（show 或 hide），任何时刻只有一个动作生效
- Option B: 无守卫，blur 直接 hide——点击托盘关卡片时先 blur 再 click，出现"关了又开"或"还开着"，直接违反 SCN-TRAY-002-B
- Decision: Option A
- Rationale: 以"最近一次用户意图"为准的窗口期抑制，覆盖两个方向（卡片可见/隐藏时点击托盘）与真实点击外部两种情形，实现为少量状态与时间戳，无需全局锁
- Consequences / Trade-offs: 隐藏有约 120ms 主观延迟（用户无感）；极端时序（一帧内"外部点击 + 托盘点击"同时发生）仍以托盘意图为准
- Reversal Trigger: 实际平台时序与假设不符时，调整守卫阈值或在托盘 Click 中直接显式 `show + focus`（当前设计已是此方向）

### DEC-008 — 打包产物：macOS `.app` + `.dmg`；Windows NSIS 安装器（含 WebView2 引导）

- Status: accepted
- Drivers and Constraints: 一期交付 macOS + Windows；FACT-001 预留 Linux；无既有发布管道
- Covers: 无直接 REQ（打包均为交付机制）；支持 SUCCESS-001~006 在真实环境验收
- Option A: `tauri build` 默认 bundle：macOS `app`+`dmg`；Windows `nsis`（含 WebView2 bootstrapper 选项）；图标由 `tauri icon` 从 1024×1024 源 PNG 生成各平台格式（.icns/.ico）
- Option B: 仅打 `app`/可执行目录，不做安装包——分发体验差，Windows 缺 WebView2 兜底
- Decision: Option A
- Rationale: tauri bundler 标准产出；NSIS 引导解决 WebView2 缺失（RISK-007）；签名/公证在一期不做硬性要求（个人本地使用），列为 DEF-DESIGN-006
- Consequences / Trade-offs: 未签名 dmg/app 在 macOS 首次打开有 Gatekeeper 提示（本地使用可接受）；Windows 安装包产出在 macOS 上不可构建（须在 Windows 环境/CI 上构建，与 SEAM-004 合并执行）
- Reversal Trigger: 需要公开分发时补签名与公证流程

### DEC-009 — 条目标识：UUID v4

- Status: accepted
- Drivers and Constraints: 删除不可撤回（INV-005）→ id 必须永不复用；自增 id 在删除后复用会产生陈旧引用歧义；一期无需要人读的顺序号
- Covers: REQ-TODO-002, REQ-TODO-003
- Option A: `uuid::Uuid::new_v4()` 字符串 id
- Option B: 自增计数 id（max+1）——删除后复用 id，若未来引入日志/引用则语义损坏
- Decision: Option A
- Rationale: 全局唯一、零状态、无需持久化计数器；todo-store crate 级别依赖，不引入 tauri 生态负担
- Consequences / Trade-offs: id 不可读（对用户无影响，UI 不展示 id）
- Reversal Trigger: 无

### DEC-010 — 前端交互细节：完成态样式、悬停删除、错误横幅、中文文案

- Status: accepted
- Drivers and Constraints: FACT-006/007/009；SCN-TODO-002-A/003-A 的可观察样式要求
- Covers: REQ-TODO-002, REQ-TODO-003；REQ-TRAY-003；SCN-TODO-002-A/B、SCN-TODO-003-A/B、SCN-TRAY-003-A
- Option A（选定组合）: 完成态 `text-decoration: line-through` + 置灰（降低对比度的中性色）；勾选框为原生 `<input type="checkbox">` 以保留键盘可达性；悬停条目（`:hover`）时显示该条目的删除按钮（其余条目不显示）；点击删除立即发起 remove；任何 command 失败时顶部显示中文错误横幅（3 秒自动消失）；空列表显示中文空状态文案；输入框 placeholder 中文；过渡动效仅元素级淡入（约 150ms），不引入动画库
- Option B: 完成态仅变色不划线 / 删除按钮常驻 / 错误仅 console —— 分别违反 SCN-TODO-002-A 的"划线置灰"、SCN-TODO-003-A 的"仅悬停条目出现"、以及用户可见的错误反馈
- Decision: Option A
- Rationale: 直接映射 SCN 可观察期望；原生 checkbox 保证键盘与辅助技术可达（Accessibility 节）；动效最小化避免范围蔓延
- Consequences / Trade-offs: 无图标库，图标用内联 SVG/Unicode；错误横幅为原型与实现共用的状态组件
- Reversal Trigger: UI 规范出现（无设计系统约束）时按规范微调样式，行为不变

## Non-Functional Design

### Security

- 零网络权限：应用不发任何网络请求（无命令、无插件、无外部 CDN；前端全部资源随包加载）。
- 数据仅写用户数据目录（`app_data_dir`），文件权限随默认用户目录（本机单用户）。
- WebView 默认安全策略（tauri v2 默认无 `dangerousRemoteDomainIpcAccess`、不启用远程 URL 加载）；`withGlobalTauri` 仅暴露 `core.invoke` 所需 API 面。
- 无账号、无密钥、无环境变量。

### Performance

- 数据规模（数百条）下：内存操作即时；JSON 序列化/写盘为微秒~毫秒级，同步写不产生可感阻塞。
- 卡片显示/隐藏为 `show/hide`（WebView 常驻），避免重建开销（INV-008）。
- 前端仅全量重渲染列表（单列表、条目级最小操作：仅更新受影响 DOM 节点）。

### Reliability

- 原子写（tmp + rename）保证任何时刻磁盘上要么旧文件、要么完整新文件（INV-002）。
- 启动加载损坏/版本不符文件 → 改名备份（`todos.json.corrupt-<timestamp>`）并以空列表继续，应用不崩溃（stderr 记录）。
- 单实例插件在重复启动时唤起既有进程，避免双写竞争（INV-001）。

### Observability

- 一期不引入日志插件：服务质量事件走 stderr（Rust `eprintln!`）与前端 `console.error`；`tauri dev` 下可直接观察。
- 结构化日志框架（`tauri-plugin-log`）列为 DEF-DESIGN-003，需要时再引入。

### Resource Cleanup

- 退出流程：托盘右键菜单"退出" → `app.exit(0)`，进程生命周期由 OS 回收；WebView 随进程退出销毁。
- 写中间文件 `todos.json.tmp` 在成功 rename 后自然消失；异常中断留下的 tmp 文件在下次 persist 时被覆盖。

## UI/UX Design

UI change，单视图卡片窗口。所有文案为中文（FACT-009）。

### Views and Pages

- 单视图：待办卡片（无边框小窗口 320×440）。结构自上而下：
  1. 卡片头部：标题"待办" + 右侧计数徽标（`formatCount`，如"共 3 项"）。
  2. 输入区：文本输入框，placeholder"添加待办，回车确认"，聚焦时边框高亮。
  3. 列表区：条目垂直排列，区域可滚动（`max-height` + `overflow-y: auto`）。
  4. 空状态/错误横幅：空列表时列表区显示"暂无待办，先添加一条吧"；错误时顶部滑入横幅。
- 无独立页面/路由；无加载页（本地即时数据）；无设置页。

### Main Flow

1. 用户点击托盘图标 → 卡片在鼠标所在屏幕边缘出现并聚焦输入框（SCN-TRAY-002-A）。
2. 输入"买牛奶"回车 → 列表末尾新增未完成条目，输入框清空（SCN-TODO-001-A）。
3. 悬停条目 → 右侧出现删除按钮（×图标，`aria-label="删除"`）（SCN-TODO-003-A）。
4. 点击勾选框 → 条目划线置灰保留（done 态）；再点恢复（SCN-TODO-002-A/B）。
5. 点击删除按钮 → 条目立即消失（无确认、无撤回）（SCN-TODO-003-B）。
6. 点击卡片外任意区域 → 卡片自动隐藏，未提交输入丢弃（SCN-TRAY-003-A）；再点托盘图标可重新打开。
7. 重启应用 → 列表与完成状态与退出前一致（SCN-TODO-004-A）。

### Empty / Loading / Error / Restricted States

- Empty：空列表 → "暂无待办，先添加一条吧"（列表区居中置灰文案）。
- Loading：本地直读 + 同步 command，无加载状态（不引入骨架屏）。
- Error：command 失败（read/write）→ 顶部中文错误横幅"保存失败，请重试"（3 秒自动消失）；界面保持变更前状态（无半更新）。
- Restricted（原型专用演示状态）：原型页脚"原型工具"区提供"模拟保存失败"按钮，仅用于演示错误横幅；真实运行路径无此入口（原型 README 说明）。

### Information Hierarchy

- 主层级：输入框（高频操作，置于最上）> 标题/计数 > 待办列表（勾选完成为置灰降级层级，突出可继续处理的未完成项）。

### Responsive Behavior

- 卡片为固定尺寸窗口（320×440），无窗口缩放/换行布局需求；列表区独立滚动保证小屏幕（如 1366×768）下不溢出。
- 不做移动端适配（桌面托盘应用）。

### Accessibility

- 勾选框使用原生 `<input type="checkbox">`（tab 可达 + 屏幕阅读器朗读 + 空格切换）。
- 删除按钮为 `<button>`，带 `aria-label="删除"`，可 tab 聚焦；悬停仅改变可见性，键盘聚焦时同样显示（`:focus-visible` 显示）。
- 输入框带 `aria-label="添加待办"`；完成态文本使用 `aria-pressed`/文本语义（划线为主要视觉提示，文本不变）。
- 对比度：置灰完成态仍保持 WCAG AA 下限（不做超低对比度）；错误横幅文本对比清晰。
- 焦点可见：输入框与删除按钮均有 `:focus-visible` 高亮。

### Prototype and Screenshot References

- 静态 prototype：`prototype/index.html`（详见 Prototype Path 与 README）。
- 设计阶段真实验证截图：`prototype/screenshots/*.png`（UI Evidence 清单见 Design Validation Evidence）。

## Risks

| Risk ID | Trigger | Impact | Mitigation | Residual Risk | Owner |
| --- | --- | --- | --- | --- | --- |
| RISK-001 | 失焦（blur）与托盘点击事件竞争时序导致误隐藏 | 违反 SCN-TRAY-002-B/003-A（点了托盘关不掉或点外部后又被打开） | DEC-007 防抖复核 + 托盘点击守卫；prototype 已覆盖显隐主路径；Apply 阶段 macOS/Windows 真实环境重点验证 | 极端并发时序仍以托盘意图为准，可能有极小窗口期次优 | 后端实现者 |
| RISK-002 | 同步写盘导致 UI 卡顿（极端大列表） | 数据量大时 command 延迟 | 一期规模（数百条）写盘毫秒级，无感知；monitor 阈值（约 >5000 条或 >10ms）触发 REVIEW 迁移异步写 | 未做压力测试 | 后端实现者 |
| RISK-003 | Windows 托盘点击事件行为差异（button/双击/右键） | 左键切换失效 | INT-002 只能以 Left button 为 toggle 键；SEAM-004 Windows 验证覆盖 | 无 Windows 环境前无法实机确认 | 后端实现者 + 用户（DEF-005） |
| RISK-004 | macOS 菜单栏点击时窗口焦点行为与预期不同（Accessory 下 status item 点击不一定先 blur 后 click） | 守卫方向偏差 | DEC-007 的守卫同时兼容先 blur 后 click 与仅 click 两种时序；macOS 真机验收覆盖两种打开/关闭路径 | 行为差异待 Apply 阶段实测校准阈值 | 后端实现者 |
| RISK-005 | 数据文件损坏 / 写入中断 | 启动空列表或数据丢失（违反 SCN-TODO-004-A 精神） | 原子写（INV-002）+ 损坏文件备份恢复 + version 字段；SEAM-001 覆盖损坏恢复用例 | 极端断电窗口（tmp 写入瞬间）最多丢最后一次变更 | 后端实现者 |
| RISK-006 | 单实例插件在 macOS 行为不完整（重复启动唤起） | 二次启动出现双托盘图标或数据竞争（违反 INV-001） | 插件为标准官方插件；macOS 真机验收（SEAM-003）验证二次启动；失效则回退 MOD-002 平台锁 | macOS 插件机制未在 macOS 上实测前存在不确定性 | 后端实现者 |
| RISK-007 | Windows 缺 WebView2 运行时 | 应用无法启动/卡片空白 | NSIS 打包启用 WebView2 bootstrapper（DEC-008） | 陈旧系统安装策略差异 | 后端实现者 |
| RISK-008 | 原型 localStorage mock 与实际 Rust 存储行为偏差（如写入失败模拟） | 误以为原型=实现 | SEAM-005 仅用于设计交互验证；Apply 阶段 SEAM-003/004 以真实运行时为准，mock 数据集中于 `assets/mock-data.js` | 原型不做为业务验收依据（README 明示） | Build/Apply |

## Migration / Rollout / Rollback

- Applicability: 全新项目首版，无既有数据/部署迁移；本地数据为易失重建代价极低（用户可重新输入）。
- Preconditions: 无。
- Steps: （一期交付即首秀）`cargo tauri dev` 调试 → `tauri build` 产物分发（macOS dmg / Windows nsis）；数据文件随首启自动创建。
- Compatibility Window: 数据文件含 `version: 1` 字段，未来版本兼容检查点在此；一期不承诺跨版本格式兼容（proposal 已声明）。
- Rollback Trigger: 明显回归（托盘失效/数据丢失/频繁崩溃）。
- Rollback Procedure: 重新安装上一构建产物；数据文件为单文件，可手动备份 `todos.json` 恢复。
- Post-Rollback Verification: 重新执行 SEAM-003 验收清单中受影响项（托盘显隐、列表恢复、增删勾选）。

## Deferred Design Items

| ID | Reason | Build Blocking | Owner | Resume Condition |
| --- | --- | --- | --- | --- |
| DEF-DESIGN-001 | Linux 平台（DEF-001 落地方案）：选型均为跨平台 API，无阻碍项 | no | 用户 | 出现 Linux 用户需求或打包通道 |
| DEF-DESIGN-002 | Windows 真实验证环境（DEF-005 落地方案：GitHub Actions `windows-latest` 为最低成本路径；远程 Windows 机为备选） | no（须在 Apply 验证前确定） | 用户 | Apply 阶段开始前必须选定并可用 |
| DEF-DESIGN-003 | 结构化日志框架（`tauri-plugin-log`），一期用 stderr/console | no | 后端实现者 | 出现远程排障或崩溃归因需求 |
| DEF-DESIGN-004 | 托盘右键菜单扩展（macOS 二级菜单、Windows 常用项） | no | 后端实现者 | 用户提出托盘菜单功能需求 |
| DEF-DESIGN-005 | 数据格式跨版本迁移（version 字段已预留） | no | 后端实现者 | 数据格式变更时 |
| DEF-DESIGN-006 | 代码签名/公证（macOS 未签名提示、Windows SmartScreen） | no | 用户 | 公开分发需求 |

## Requirement Change Requests

| Change Request ID | Affected IDs | Reason | Proposed Change | Status |
| --- | --- | --- | --- | --- |
| （无） | – | 设计未改变任何已批准的用户可观察行为、capability、spec requirement 或非目标；交互细节（尺寸、文案、动效、定位、退出路径）属于 Design 授权范围 | – | – |

说明（非变更，仅记录）：本设计新增"托盘右键菜单【退出】"作为应用正常退出路径（决策归属 DEC-006 与 DEC-010）。该元素不改变任何已确认的可观察行为（REQ-TRAY-001~003 均只约束图标常驻、左键切换、失焦关闭），且是托盘常驻应用的必需退出通道，故按设计细节处理而非需求变更。

## Design Validation Evidence

- Validation Commands:
  - `openspec validate "tray-todo" --type change --json` → `{"items":[{"id":"tray-todo","type":"change","valid":true,"issues":[]}],"summary":{"totals":{"items":1,"passed":1,"failed":0}}}`
  - `node openspec/changes/tray-todo/prototype/tools/run-logic-check.mjs` → `LOGIC_CONTRACT_OK`（isValidInput 空/纯空白/非空/trim/防御分支 + formatCount 中文文案全部断言通过）
  - `python3 -m http.server 8701 --directory openspec/changes/tray-todo/prototype`（UI 预览入口，HTTP 200）
  - Chrome headless（`--remote-debugging-port=9333`，Chrome 151）+ `node openspec/changes/tray-todo/prototype/tools/validate-ui.mjs`（CDP 真实交互）
- Validation Results: `openspec validate` 1/1 change valid、0 issues；UI 驱动 20/20 断言 PASS、0 运行时错误（consoleErrors/exceptions/logErrors 均为空）；logic 契约检查通过。详见下方 UI Evidence。
- Prototype Path: `openspec/changes/tray-todo/prototype/`（入口 `index.html`；`README.md` 含覆盖范围、mock 映射与 Build 提示；`tools/validate-ui.mjs` 为可复跑的 CDP 验证驱动）
- UI Evidence:
  - 预览入口：`http://127.0.0.1:8701/index.html`（python3 http.server，真实浏览器 Chrome headless 加载）
  - 实际执行并通过（真实鼠标/键盘事件注入，非静态检查）：
    1. 初始：卡片隐藏、空列表（托盘常驻态）
    2. 点击模拟托盘图标 → 卡片显示 + 空状态可见 + 输入框可聚焦（SCN-TRAY-002-A）
    3. 空输入回车 → 列表仍 0 条（SCN-TODO-001-B）
    4. 依次输入"买牛奶""开会""写周报"回车 → 3 条、顺序一致、输入框清空（SCN-TODO-001-A）
    5. 悬停第 2 条 → 删除按钮 opacity 0,1,0（仅被悬停条目显示）（SCN-TODO-003-A）
    6. 勾选第 1 条 → done + `line-through` + color `rgb(154,160,166)`（置灰）、保留在列表（SCN-TODO-002-A）
    7. 再点勾选 → 恢复 `none` 样式（SCN-TODO-002-B）
    8. 悬停并点击"开会"删除按钮 → 条目立即消失、无确认弹窗、列表 2 条（SCN-TODO-003-B）
    9. 点击卡片外部区域 → 卡片自动隐藏（SCN-TRAY-003-A）
    10. 再次点击模拟托盘 → 重新显示（SCN-TRAY-002-A 往返）
    11. reload（模拟重启）→ 2 条恢复且完成状态一致（`买牛奶:done|写周报:pending`）（SCN-TODO-004-A）
    12. 开启"模拟保存失败"后添加 → 列表不变（不提交变更）+ 中文错误横幅
      "保存失败：模拟保存失败：写入磁盘出错"（错误/受限状态）；恢复后添加成功
  - DOM/样式断言基于真实渲染 computed style（删除按钮 opacity、`textDecorationLine`、颜色、横幅文本、localStorage 内容）
  - consoleErrors: none；exceptions: none；logErrors: none（Chrome `Runtime.consoleAPICalled error` / `exceptionThrown` / `Log.entryAdded error` 三类全部为空）
  - 视口：桌面 1280×800（deviceScaleFactor 1；卡片 320×440 居中于菜单栏下方，模拟 DEC-002 定位形态）
  - 说明：本模型不支持图片内视，截图路径与前端断言均为自动记录值；截图已落盘供 Review 人工目检。示例断言原文：
    `勾选完成-划线置灰 | decoration=line-through, color=rgb(154, 160, 166)`；
    `悬停仅显示被悬停条目的删除按钮 | opacities=0,1,0`；
    `reload后列表与完成状态一致 | items=买牛奶:done|写周报:pending`
- Screenshots: `prototype/screenshots/`，11 组步骤截图（页面级 1280×800 + 卡片级特写）：`01-empty`、`02-added`、`03-hover-delete`、`04-checked`、`05-unchecked`、`06-after-delete`、`07-hidden`（隐藏态仅页面级）、`08-reopened`、`09-reload-restored`、`10-error-banner`、`11-recovered`
- Remaining Blockers: 无。DEF-005（Windows 验证环境）不阻塞设计，须在 Apply 验证前确定。

## Recovery Checkpoint

- Design Status: completed（design.md v1 已写入；`openspec validate "tray-todo" --type change --json` 于 2026-08-26 结果为 `valid: true`、1/1 passed、0 issues；prototype 与 UI Evidence 已完成）
- Last Completed Section: 全部章节完成，含 Design Validation Evidence 实况固化；prototype 生成、CDP 真实 UI 验证（20/20 PASS、零运行时错误、11 组截图）与进程清理均已完成
- Unresolved Decisions: 无。DEC-001~010 全部 accepted；唯一有待真实验证的平台行为（RISK-003/004/006，macOS/Windows 托盘与焦点时序）全部有明确缓解与 Apply 阶段验证项，不阻塞
- Required Revalidation: 若 proposal/任一 spec 制品摘要变化 → 重读 Design Handoff 与对应 REQ/SCN 后重校本设计；若进入下一步（tasks）前开放任何 CR → 先由 Plan 决策
- Next Permitted Action: 交回 `openspec-plan`——进入 Apply 前运行 `openspec-create-tasks` 生成 `tasks.md`（design 为唯一任务来源）；Apply 阶段按 SEAM-001/002（`cargo test -p todo-store`、`node --test src/logic.test.mjs`）与 SEAM-003/004（平台验收）执行验证