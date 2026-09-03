# Tasks: todos

## Plan Metadata

- Proposal Revision / Digest: proposal.md Revision 1（2026-08-26 用户确认需求结晶；HYP-001 confirmed 单实例；HYP-002 open，由 DEC-001 结构保证落实）
- Spec Revisions / Digests: specs/tray-card/spec.md v1（CAP-001，REQ-TRAY-001~003）；specs/todo-management/spec.md v1（CAP-002，REQ-TODO-001~004）
- Design Revision / Digest: design.md v1（DEC-001~010 accepted；INT-001~005；INV-001~009；SEAM-001~005；RISK-001~008；DEF-DESIGN-001~006）
- Plan Digest: 从零新建 Tauri v2 项目，按垂直切片实施：1) todo-store 独立 crate（数据模型/load/add/toggle/remove/原子持久化/损坏恢复，SEAM-001）；2) 前端纯逻辑 logic.mjs（SEAM-002）；3) 前端 UI（列表渲染/添加/勾选/悬停删除/错误横幅/中文文案）；4) command 层（list/add/toggle/remove + 中文错误映射）；5) 托盘图标与左键切换/右键退出；6) 无边框卡片窗口（show/hide/定位/失焦守卫）；7) 应用装配（单实例/accessory/数据路径）。验证：SEAM-001/002 单测全绿 → SEAM-003 macOS 手动验收 → SEAM-004 Windows 验收（环境待用户确认）→ 打包验收。清理调试残留。完整验证须覆盖全部 7 REQ / 13 SCN / SUCCESS-001~006。
- Build-Ready Status: pending（待 Apply 前复验）
- Apply Approval Status: pending（待用户批准）

## 编码规范（对 Build 的强制纪律）

业务代码注释用中文清晰描述代码本身的意图、输入、输出与边界；**严禁**在 `src-tauri/` 与 `src/` 的任何源码、配置、测试或注释中出现 OpenSpec 计划追踪字眼：`DEC-*`、`INT-*`、`INV-*`、`SEAM-*`、`TASK-*`、`REQ-*`、`SCN-*`、`SUCCESS-*`、`MOD-*`、`CAP-*`、`HYP-*`、`FACT-*`、`RISK-*`、`DEF-*`、`EVD-*`。这些字眼仅允许出现在 `tasks.md`、`design.md`、`proposal.md`、`specs/` 与 `prototype/` 等 OpenSpec 制品中。若 Apply 从旧代码中带入此类注释，必须改写为中文描述后再提交。

## Traceability Matrix

| REQ / SCN | DEC / INT / INV | SEAM | TASK | Evidence |
| --- | --- | --- | --- | --- |
| REQ-TRAY-001, SCN-TRAY-001-A | DEC-004, INT-002, INV-001, EXT-001/005 | SEAM-003 | TASK-1.7, TASK-1.9 | EVD-TASK-1.7, EVD-TASK-1.9 |
| REQ-TRAY-002, SCN-TRAY-002-A | DEC-002/006/007, INT-002, INV-008, EXT-005 | SEAM-003 | TASK-1.7, TASK-1.8 | EVD-TASK-1.7, EVD-TASK-1.8 |
| REQ-TRAY-002, SCN-TRAY-002-B | DEC-002/006/007, INT-002, INV-008 | SEAM-003 | TASK-1.7, TASK-1.8 | EVD-TASK-1.7, EVD-TASK-1.8 |
| REQ-TRAY-003, SCN-TRAY-003-A | DEC-006/007, INT-003, INV-007 | SEAM-003 | TASK-1.8 | EVD-TASK-1.8 |
| REQ-TODO-001, SCN-TODO-001-A | DEC-001/009, INT-001/004/005, INV-002/003/004 | SEAM-001, SEAM-002 | TASK-1.1, TASK-1.2, TASK-1.4, TASK-1.5, TASK-1.6 | EVD-TASK-1.1, EVD-TASK-1.2, EVD-TASK-1.4, EVD-TASK-1.5, EVD-TASK-1.6 |
| REQ-TODO-001, SCN-TODO-001-B | DEC-001, INT-001/004/005, INV-003 | SEAM-001, SEAM-002 | TASK-1.2, TASK-1.4, TASK-1.5, TASK-1.6 | EVD-TASK-1.2, EVD-TASK-1.4, EVD-TASK-1.5, EVD-TASK-1.6 |
| REQ-TODO-002, SCN-TODO-002-A | DEC-001/009/010, INT-001/004, INV-006 | SEAM-001, SEAM-002 | TASK-1.3, TASK-1.5, TASK-1.6 | EVD-TASK-1.3, EVD-TASK-1.5, EVD-TASK-1.6 |
| REQ-TODO-002, SCN-TODO-002-B | DEC-001/010, INT-001/004, INV-006 | SEAM-001, SEAM-002 | TASK-1.3, TASK-1.5, TASK-1.6 | EVD-TASK-1.3, EVD-TASK-1.5, EVD-TASK-1.6 |
| REQ-TODO-003, SCN-TODO-003-A | DEC-010, INT-001, INV-005 | SEAM-002, SEAM-003 | TASK-1.5, TASK-1.6 | EVD-TASK-1.5, EVD-TASK-1.6 |
| REQ-TODO-003, SCN-TODO-003-B | DEC-001/010, INT-001/004, INV-002/005 | SEAM-001, SEAM-002 | TASK-1.3, TASK-1.5, TASK-1.6 | EVD-TASK-1.3, EVD-TASK-1.5, EVD-TASK-1.6 |
| REQ-TODO-004, SCN-TODO-004-A | DEC-001/005, INT-004, INV-002/009, EXT-003 | SEAM-001 | TASK-1.1, TASK-1.2, TASK-1.3, TASK-1.9 | EVD-TASK-1.1, EVD-TASK-1.2, EVD-TASK-1.3, EVD-TASK-1.9 |
| DEC-003 | — | SEAM-002 | TASK-1.4, TASK-1.5 | EVD-TASK-1.4, EVD-TASK-1.5 |
| DEC-008 | — | SEAM-003, SEAM-004 | TASK-3.3, TASK-3.4, TASK-3.5 | EVD-TASK-3.3, EVD-TASK-3.4, EVD-TASK-3.5 |

## 1. Behavior

- [ ] 1.1 [behavior] TASK-1.1 实现 todo-store 数据模型与加载（SEAM-001 建立）
  - Covers: REQ-TODO-004, SCN-TODO-004-A（load/persist 往返基础）
  - Implements: DEC-001, DEC-005（路径解耦接口）, INT-004（load/list 部分）, INV-002（原子写基础）, INV-009
  - Depends On: none
  - Targets: `src-tauri/crates/todo-store`（独立 crate，零 tauri 依赖：Cargo.toml + src/lib.rs + src/todos.rs）
  - Test Seam: SEAM-001（`cargo test -p todo-store`）
  - Red Verification: `cargo test -p todo-store`
  - Expected Red Result: `error[E0432]: unresolved import` 或测试失败（crate/类型/测试不存在）
  - Implementation Direction: 新建独立 crate；定义 `TodoItem { id: String(uuid), text: String, done: bool, created_at: i64 }`、`TodoFile { version: u32 }`、`TodoStore`；`load(path)`：文件缺失→空库、JSON 损坏/version!=1→改名备份（`todos.json.corrupt-<timestamp>`）+ 空库、成功→内存列表；`list()` 返回追加顺序列表；依赖仅 std/serde/serde_json/uuid/thiserror
  - Done Criteria: `cargo test -p todo-store` 全绿；测试覆盖：空路径→空库、损坏 JSON→备份文件生成且空库、正常 roundtrip 中文 UTF-8 无乱码
  - Pass Verification: `cargo test -p todo-store` 退出码 0 且输出显示 4 个以上用例通过
  - Evidence: pending

- [ ] 1.2 [behavior] TASK-1.2 实现 add 与原子持久化（append + 空文本拒绝）
  - Covers: REQ-TODO-001, SCN-TODO-001-A/B；REQ-TODO-004 持久化写入路径
  - Implements: DEC-001（原子写）、DEC-009（uuid）、INT-004（add/persist）、INV-002/003/004
  - Depends On: TASK-1.1
  - Targets: `src-tauri/crates/todo-store/src/lib.rs`（add + persist）
  - Test Seam: SEAM-001
  - Red Verification: `cargo test -p todo-store`
  - Expected Red Result: 新增用例失败：add 未实现/空文本未拒绝/追加顺序不符（如期望 items.len()==1 实际 0）
  - Implementation Direction: `add(text)`：trim 后为空→`Err(InvalidInput)`；非空→`Uuid::new_v4()` + `created_at`（unix 秒）+ done=false 追加队尾；随后 `persist()`：序列化 `TodoFile{version:1, items}` 写入 `<path>.tmp` 后 `fs::rename` 原子替换；落在测试可写临时目录（std::env::temp_dir + 唯一子目录）
  - Done Criteria: 测试覆盖：非空添加返回条目且列表顺序=添加顺序；空/纯空白→InvalidInput 且列表不变；persist 后磁盘文件可被重新 load 且内容一致（中文文本无乱码）；tmp 文件在 persist 成功后不存在
  - Pass Verification: `cargo test -p todo-store` 退出码 0 且新增用例（空文本拒绝、追加顺序、原子写后重载一致）全部 pass
  - Evidence: pending

- [ ] 1.3 [behavior] TASK-1.3 实现 toggle/remove 与变更即落盘
  - Covers: REQ-TODO-002, SCN-TODO-002-A/B；REQ-TODO-003, SCN-TODO-003-B（删除后持久化语义）
  - Implements: DEC-001, INT-004（toggle/remove）, INV-002/005/006
  - Depends On: TASK-1.2
  - Targets: `src-tauri/crates/todo-store/src/lib.rs`（toggle/remove）
  - Test Seam: SEAM-001
  - Red Verification: `cargo test -p todo-store`
  - Expected Red Result: 新增用例失败：toggle 未翻转 done/NotFound 未返回/remove 未移除或未持久化
  - Implementation Direction: `toggle(id)`：找不到→`Err(NotFound)`；找到→置反 done 并立即 persist；`remove(id)`：找不到→`Err(NotFound)`；找到→移除并立即 persist；两个方法均返回 `Result`，磁盘错误→`IoError` 且内存状态可回滚（本任务先以"persist 失败时返回 Err、内存不回滚可接受"为最小实现，INV-002 的强一致在 UI 层配合：command 失败界面不更新）
  - Done Criteria: 测试覆盖：toggle 往返（pending→completed→pending）、toggle 后持久化（重载一致）、remove 后列表减少且持久化（重载不存在）、id 不存在→NotFound；completed 条目可删除
  - Pass Verification: `cargo test -p todo-store` 退出码 0 且 toggle/remove 相关用例全部 pass
  - Evidence: pending

- [ ] 1.4 [behavior] TASK-1.4 实现前端纯逻辑 logic.mjs 与 Node 单测（SEAM-002）
  - Covers: REQ-TODO-001, SCN-TODO-001-A/B（前端校验部分）
  - Implements: DEC-003, INT-005, INV-003（前端侧）
  - Depends On: none（可与 TASK-1.1 并行）
  - Targets: `src/logic.mjs` + `src/logic.test.mjs`
  - Test Seam: SEAM-002（`node --test src/logic.test.mjs`）
  - Red Verification: `node --test src/logic.test.mjs`
  - Expected Red Result: `# Subtest ... not ok`（断言失败：空字符串/空格/Tab 未返回 false）
  - Implementation Direction: 按 prototype `assets/logic.mjs` 契约实现：`isValidInput(text)`（`typeof text !== "string"` 或 trim 后长度为 0 → false；否则 true）；`formatCount(n)`（"共 N 项"中文文案，非数及负数防御返回"共 0 项"）；`.mjs` ES Module 双端可加载
  - Done Criteria: 测试覆盖：空字符串/空格/Tab/非字符串→false；中文与首尾空格→true；formatCount 0/1/3 中文输出
  - Pass Verification: `node --test src/logic.test.mjs` 退出码 0，全部断言 pass（输出含 `# pass` 计数）
  - Evidence: pending

- [ ] 1.5 [behavior] TASK-1.5 实现前端 UI 主逻辑（渲染、添加、勾选、悬停删除、错误横幅、空状态、中文文案）
  - Covers: REQ-TODO-001/002/003 全部 UI 场景（SCN-TODO-001-A/B, 002-A/B, 003-A/B）；REQ-TRAY-003 的"未提交输入丢弃"配合
  - Implements: DEC-003, DEC-010（全部交互细节与中文文案），INV-003（前端先行校验）, INV-005/006/007（渲染与交互侧）
  - Depends On: TASK-1.4
  - Targets: `src/index.html` + `src/styles.css` + `src/app.mjs`
  - Test Seam: SEAM-002（逻辑复用 + 结构保证）；交互验收挂 SEAM-003（手动）
  - Red Verification: `node --test src/logic.test.mjs` 持续绿约束；UI 验证为 SEAM-003 手动清单：在 `cargo tauri dev`（依赖 TASK-1.6 起）加载前先以静态方式检查语法：`node --check src/app.mjs`、`node --check src/logic.mjs`
  - Expected Red Result: `node --check` 报语法错误，或手动验收清单任一项失败（勾选无划线与置灰、删除按钮悬停不出现、错误横幅不出现）
  - Implementation Direction: 按 prototype 交互契约（非拷贝实现）：320×440 卡片布局（头部"待办"+计数徽标、placeholder"添加待办，回车确认"输入框、可滚动列表区、空状态"暂无待办，先添加一条吧"、顶部错误横幅 3 秒自动消失）；`add_todo`/`toggle_todo`/`remove_todo`/`list_todos` 经 `window.__TAURI__.core.invoke` 调用（withGlobalTauri）；前端先调 `isValidInput` 校验再提交；done 条目 `line-through`+置灰保留；删除按钮仅 `:hover`/`:focus-visible` 显示且 `aria-label="删除"`；checkbox 用原生控件；任一 invoke 失败→中文错误横幅且界面保持变更前状态；输入框聚焦
  - Done Criteria: 手动验收（SEAM-003）：空状态可见；回车添加非空/拒绝空输入；勾选划线置灰保留、再点恢复；悬停仅被悬停条目显示删除按钮；删除即消失无确认；错误横幅出现与自动消失；全部中文文案
  - Pass Verification: `node --check src/app.mjs src/logic.mjs` 退出码 0；SEAM-003 清单对应项通过并记录
  - Evidence: pending

- [ ] 1.6 [behavior] TASK-1.6 实现 command 层（list/add/toggle/remove + 中文错误映射 + 注册）
  - Covers: REQ-TODO-001/002/003/004 后端入口（SCN-TODO-001-A/B, 002-A/B, 003-B, 004-A）
  - Implements: DEC-001（TodoStore 注入）, INT-001（DTO 与错误语义）
  - Depends On: TASK-1.1, TASK-1.3（store API 就绪；可与 TASK-1.5 并行）
  - Targets: `src-tauri/src/commands.rs` + `lib.rs`（invoke_handler 注册）
  - Test Seam: SEAM-001（底层语义已测）+ SEAM-003（经 UI 端到端验证）
  - Red Verification: `cargo check`（在 `src-tauri/` 下）
  - Expected Red Result: 编译失败（命令函数/注册缺失）或在 `cargo tauri dev` 下 invoke 返回 reject
  - Implementation Direction: `list_todos`/`add_todo(text)`/`toggle_todo(id)`/`remove_todo(id)` 薄包装 `Mutex<TodoStore>`（`tauri::State<'_, Mutex<TodoStore>>`）；返回 `TodoItemDto { id, text, done, created_at }`；错误映射：`InvalidInput`→"内容为空，无法添加"、`NotFound`→"待办不存在"、`IoError`→"保存失败：写入磁盘出错"；全部注册进 `invoke_handler`
  - Done Criteria: `cargo check` 通过；`cargo tauri dev` 下经 SEAM-003 验证添加/勾选/删除均成功落盘；写入失败路径（对只读目录等）返回中文 Err（SEAM-003 清单记录）
  - Pass Verification: `cargo check` 退出码 0；SEAM-003 清单对应项通过
  - Evidence: pending

- [ ] 1.7 [behavior] TASK-1.7 实现托盘图标（常驻、左键切换指令、右键退出菜单）
  - Covers: REQ-TRAY-001, SCN-TRAY-001-A；REQ-TRAY-002 事件入口（SCN-TRAY-002-A/B）
  - Implements: DEC-004（单实例前提）, DEC-006（退出路径）, DEC-007（托盘点击时间戳守卫）, INT-002, INV-001（配合单实例）
  - Depends On: TASK-1.8（toggle 目标窗口接口；可先定义 trait/占位接口再并行）
  - Targets: `src-tauri/src/tray.rs`（TrayIconBuilder）
  - Test Seam: SEAM-003（macOS 手动）
  - Red Verification: `cargo check` 后 `cargo tauri dev`
  - Expected Red Result: 托盘图标不出现，或左键无反应/右键菜单缺【退出】
  - Implementation Direction: `TrayIconBuilder::with_id("main")` + 图标资源 + `Tooltip("待办")`；左键 `TrayIconEvent::Click { button: Left, .. }` → 记录 `last_tray_click` 时间戳（供 TASK-1.8 守卫）并调用卡片 `toggle()`；`TrayIconBuilder::menu` 原生菜单含【退出】→ `app.exit(0)`；图标资源由 `tauri icon` 从源 1024×1024 PNG 生成（.png/.icns/.ico 按平台）
  - Done Criteria: `cargo tauri dev` 下 macOS 菜单栏出现图标；左键触发卡片显隐切换（与 TASK-1.8 联调）；右键菜单【退出】正常退出进程
  - Pass Verification: SEAM-003 清单"托盘常驻/左键切换/右键退出"三项通过并记录截图
  - Evidence: pending

- [ ] 1.8 [behavior] TASK-1.8 实现卡片窗口（无边框、定位、show/hide、失焦守卫、CloseRequested→hide）
  - Covers: REQ-TRAY-002, SCN-TRAY-002-A/B（窗口打开/关闭路径）；REQ-TRAY-003, SCN-TRAY-003-A（失焦隐藏与未提交输入丢弃）
  - Implements: DEC-002（无边框 320×440 + 定位策略）, DEC-006（CloseRequested→hide、窗口单例）, DEC-007（防抖复核 + 托盘点击守卫）, INT-003, INV-007/008
  - Depends On: TASK-1.7（托盘事件）；可先实现窗口单例与 toggle API
  - Targets: `src-tauri/src/card.rs` + `lib.rs`（窗口创建装配）
  - Test Seam: SEAM-003（macOS 手动）
  - Red Verification: `cargo check` 后 `cargo tauri dev`
  - Expected Red Result: 窗口尺寸/边框不符（出现标题栏或尺寸错误）、位置离谱、失焦不隐藏、点托盘"关了又开"
  - Implementation Direction: `WebviewWindowBuilder`：`decorations(false)`、`inner_size(320, 440)`、启动即 `visible(false)`、`title("待办")`；窗口单例持有（INV-008）；`toggle()`：visible→hide；hidden→定位后 `show + set_focus`；定位：`available_monitors` 取鼠标所在显示器（`CursorPosition`），macOS 工作区顶部 +8px、Windows 工作区底部 - 高度 -8px，水平对齐鼠标 x 并夹紧工作区；`CloseRequested` → `prevent_close` + hide；`Focused(false)` → 120ms 防抖后复核 `is_focused()` 且距 `last_tray_click` 超过约 120ms 才 hide；hide 时清空输入框与错误横幅（未提交输入丢弃，INV-007）
  - Done Criteria: macOS 手动：打开位置贴近菜单栏下方；点击外部自动隐藏且无残留；显隐往返无重复窗口；关闭请求（如 Cmd+W 类路径）不退出进程仅隐藏；托盘点击不触发"关了又开或开了又关"
  - Pass Verification: `cargo check` 退出码 0；SEAM-003 清单"显隐切换/失焦关闭/定位"通过并记录截图（含关闭前后）
  - Evidence: pending

- [ ] 1.9 [behavior] TASK-1.9 实现应用装配（单实例、macOS Accessory、数据路径、启动加载、二次启动唤起）
  - Covers: REQ-TRAY-001, SCN-TRAY-001-A（单实例/常驻前提）；REQ-TODO-004, SCN-TODO-004-A（启动恢复完整闭环）
  - Implements: DEC-004（single-instance 插件）, DEC-005（app_data_dir）, DEC-006（Accessory）, EXT-001/003, INV-001
  - Depends On: TASK-1.3, TASK-1.6, TASK-1.7, TASK-1.8
  - Targets: `src-tauri/src/lib.rs` + `main.rs` + `tauri.conf.json`（identifier/打包基础）
  - Test Seam: SEAM-001（load 已有测试复跑）+ SEAM-003（macOS 手动）
  - Red Verification: `cargo tauri dev`
  - Expected Red Result: 二次启动出现第二个图标/进程，或启动时数据不恢复，或 macOS Dock 出现图标
  - Implementation Direction: `tauri.conf.json` 固定 `identifier`（如 `com.swcrbt.todos`）与 `app_data_dir` 后端取路径；`tauri-plugin-single-instance` 初始化（`tauri_plugin_single_instance::init`），二次启动回调 → 唤起并显示卡片窗口；`setup`：`app.path().app_data_dir()` 下 `todos.json` → `TodoStore::load` 注入 `Mutex<TodoStore>`；macOS `ActivationPolicy::Accessory`（无 Dock 图标）；commands/tray/card 完成装配；`main.rs` 调 `run()`
  - Done Criteria: macOS 手动：重复启动不产生双进程/双图标且二次启动后卡片出现；重启后列表与完成状态恢复；Dock 无图标、仅菜单栏常驻；退出菜单后进程完全结束
  - Pass Verification: `cargo tauri dev` 无报错；SEAM-003 清单"单实例/重启恢复/无 Dock 图标"通过并记录；`cargo test -p todo-store` 复跑仍全绿
  - Evidence: pending

## 2. Bugs

本 change 为全新项目，无既有代码/已知缺陷可复现，本节省略独立任务；Apply 期间若 TASK-1.1~1.9 暴露缺陷，在对应 EVD 中记录复现命令、根因与回归修复（不新增改动范围）。

## 3. Verification

- [ ] 3.1 [verify] TASK-3.1 全量运行 SEAM-001（todo-store 单元测试）
  - Scope: TASK-1.1, TASK-1.2, TASK-1.3（数据模型/加载/加添/勾删/持久化/损坏恢复全部用例）；覆盖 SCN-TODO-001-A/B, 002-A/B, 003-B, 004-A
  - Commands: `cd src-tauri && cargo test -p todo-store`
  - Required Result: 退出码 0；输出显示全部测试 pass（含中文 UTF-8 往返、原子写、损坏恢复、NotFound/InvalidInput 用例）
  - Evidence: pending

- [ ] 3.2 [verify] TASK-3.2 全量运行 SEAM-002（前端纯逻辑单测）
  - Scope: TASK-1.4（isValidInput/formatCount 契约）
  - Commands: `node --test src/logic.test.mjs`
  - Required Result: 退出码 0；全部断言 pass（空/纯空白/非字符串拒绝、中文合法、formatCount 中文输出）
  - Evidence: pending

- [ ] 3.3 [verify] TASK-3.3 macOS 手动 UI 验收（SEAM-003）
  - Scope: TASK-1.5~1.9 全部可观察行为；覆盖全部 SCN 与 SUCCESS-001~006
  - Commands: `cargo tauri dev` + 验收清单（托盘出现/左键切换/失焦关闭/添加拒绝空输入/勾选划线置灰可取消/悬停删除/重启恢复/单实例二次启动/右键退出/错误横幅触发路径）
  - Required Result: 全部清单项通过；每项记录截图或文字证据；console 无错误；`cargo tauri dev` 进程退出干净
  - Evidence: pending

- [ ] 3.4 [verify] TASK-3.4 Windows 真实验证（SEAM-004，依赖 DEF-DESIGN-002 环境选定）
  - Scope: 全部 SCN 的 Windows 平台行为（托盘左键/失焦/添加/勾选/删除/重启恢复/单实例/右键退出/打包启动）
  - Depends On: 用户确认验证环境（DEF-DESIGN-002：GitHub Actions `windows-latest` 优先，远程 Windows 机备选）
  - Commands: 选定环境中 `tauri build`（NSIS，含 WebView2 bootstrapper）+ 运行构建产物执行验收清单
  - Required Result: Windows 上清单全部通过；`tauri build` 产出 NSIS 安装包；记录证据（日志/截图/CI 输出）
  - Evidence: pending

- [ ] 3.5 [verify] TASK-3.5 macOS 打包验收（DEC-008）
  - Scope: 打包产物 `.app` + `.dmg`；产物启动后验收清单抽查（托盘常驻/列表恢复）
  - Commands: `cd src-tauri && cargo tauri build`（bundle dmg）+ 安装/挂载后启动产物执行抽查
  - Required Result: 构建退出码 0；`.app`/`.dmg` 产物存在；产物启动后托盘图标出现且列表数据恢复
  - Evidence: pending

## 4. Cleanup

- [ ] 4.1 [cleanup] TASK-4.1 清理业务代码调试残留
  - Depends On: TASK-1.9, TASK-3.3
  - Artifacts to Remove: 业务代码（src-tauri/ 与 src/）中的临时调试输出（eprintln!/console.log 调试行）、占位图标/占位文本、未使用资源与未引用模块；确认无调试用进程/临时文件残留（`todos.json.tmp` 等不应残留在数据目录）
  - Absence Verification: `git status --porcelain` 仅含预期文件；`rg "TODO|HACK|debug|console\.log|eprintln" src src-tauri/src` 仅命中有意保留的日志路径（stderr 服务事件记录）或零命中
  - Evidence: pending

## Apply Evidence

### EVD-TASK-1.1

- Task: TASK-1.1
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-1.2

- Task: TASK-1.2
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-1.3

- Task: TASK-1.3
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-1.4

- Task: TASK-1.4
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-1.5

- Task: TASK-1.5
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-1.6

- Task: TASK-1.6
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-1.7

- Task: TASK-1.7
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-1.8

- Task: TASK-1.8
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-1.9

- Task: TASK-1.9
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-3.1

- Task: TASK-3.1
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-3.2

- Task: TASK-3.2
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-3.3

- Task: TASK-3.3
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-3.4

- Task: TASK-3.4
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-3.5

- Task: TASK-3.5
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

### EVD-TASK-4.1

- Task: TASK-4.1
- Plan Digest:
- Changed Files:
- Red Command:
- Red Exit Code:
- Red Result:
- Pass Command:
- Pass Exit Code:
- Pass Result:
- Spec Compliance Review:
- Code Quality Review:
- Diff / Commit Reference:
- Evidence Digest / Log Reference:
- Remaining Risk:

## Full Verification

- Commands: `cargo test -p todo-store`；`node --test src/logic.test.mjs`；`cargo tauri dev`（SEAM-003 手动清单）；Windows 环境 `tauri build` + 验收清单（SEAM-004）；`cargo tauri build`（macOS 打包）；`openspec validate "todos" --type change --json`
- Results: pending（Apply 完成后填写：全部命令退出码与证据、跨完整性结果）
- Environment / Preconditions: macOS 开发机（Xcode 命令行工具 + Rust 工具链 + Node ≥ 20）；Windows 验证环境待用户确认（DEF-DESIGN-002）；无网络/账号要求
- UI Evidence: SEAM-003/004 验收清单的截图与记录；错误横幅触发路径记录；console 无错误声明；进程清理确认
- Screenshots: 验收截图落盘于 change 目录或工作记录（路径在 Apply 后填写）
- Verification Digest / Log Reference: pending
- Unresolved Failures: pending（已知非阻断：RISK-003/004/006 平台行为差异以 SEAM-003/004 实测结果为准）

## Recovery Checkpoint

- Last Completed Task: none（tasks.md 刚创建，Plan Digest 已固化）
- In-Flight Task: none
- Blocked Tasks: TASK-3.4 等待用户确认 Windows 验证环境（DEF-DESIGN-002）
- Confirmed Changed Files: `tasks.md`（本次）；既有制品 proposal.md/specs/design.md/prototype 未改动
- Required Revalidation: 若 proposal/specs/design 制品摘要变化（含 CR 开启），重读对应章节并重核 Traceability Matrix 后继续；若环境（Rust/Node 版本）与 design 记录不符，先重验 SEAM-001/002 可运行性
- Resume Condition: 用户批准 Apply 后，openspec-build 从 TASK-1.1 开始按依赖顺序实施，先写失败测试再最小实现
- Next Permitted Action: 交回 `openspec-plan`——展示本计划、design 与 prototype 后，请求用户批准 `todos` 进入 Apply（`openspec-build` 执行）