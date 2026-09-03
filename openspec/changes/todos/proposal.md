# Change: todos

## Metadata

- Change ID: todos
- Schema / Version: spec-driven
- Proposal Revision: 1
- Current Stage: propose
- Requirements Confirmation: 2026-08-26 用户确认需求结晶摘要（平台 macOS+Windows：架构预留 Linux；技术栈 Tauri v2；点击托盘图标切换显隐、失焦关闭；输入框+回车添加、空输入不添加；勾选划线保留可取消；悬停删除按钮直删、无确认；本地持久化；中文界面），绑定 Proposal Revision 1

## Why

用户需要一个跨平台、常驻系统托盘（macOS 菜单栏 / Windows 托盘）的轻量待办工具：点击托盘图标即可弹出待办卡片，完成添加、勾选、删除等快速操作，无需打开完整主窗口。当前项目目录为空，没有任何既有工具或代码，需要从零构建第一期可用版本并选定可持续演进的技术路线。

## Problem

### Observed Current Behavior

全新空项目，无既有产品行为可观察：

- 工作目录（本次初始化时的仓库目录）为空，无任何业务代码、配置或制品。
- 无系统的、跨平台的托盘待办工具可用，用户需要在任意时刻快速记录和整理待办。

### Affected Users / Roles / Permission Boundaries

- 用户：开发者本人，本地单人使用；无多用户、账号或权限体系。
- 权限边界：纯本地桌面应用，仅读写本机用户数据目录；不需要网络权限、无外部服务调用。

### Evidence and Sources

- 用户直接陈述需求：跨平台 todo 工具，第一期仅实现点击菜单栏图标弹出待办卡片，可添加、删除、勾选完成。
- 环境事实：目录为空；OpenSpec 于 2026-08-26 初始化；git 仓库于同日初始化。
- 需求结晶确认记录：平台范围、技术栈、窗口行为、添加/勾选/删除交互、持久化、界面语言共 8 项决策由用户在 Explore 阶段逐项确认。

### Why Now

需求明确、范围收敛为可独立交付的第一期，且此刻项目无历史包袱，可以一次性确定跨平台技术路线（Tauri v2），避免后期从单平台方案重写。

## Confirmed Facts

| Fact ID | Fact | Source | Confidence |
| --- | --- | --- | --- |
| FACT-001 | 一期覆盖平台为 macOS + Windows，架构预留 Linux | 需求结晶确认 | confirmed |
| FACT-002 | 技术栈为 Tauri v2（Rust 后端 + Web 前端） | 需求结晶确认 | confirmed |
| FACT-003 | 点击托盘图标切换待办卡片显示/隐藏 | 需求结晶确认 | confirmed |
| FACT-004 | 点击卡片外部区域（失焦）自动关闭卡片 | 需求结晶确认 | confirmed |
| FACT-005 | 添加待办：卡片顶部输入框输入后回车提交，空输入不添加 | 需求结晶确认 | confirmed |
| FACT-006 | 勾选完成：条目划线置灰保留在列表中，可取消勾选恢复 | 需求结晶确认 | confirmed |
| FACT-007 | 删除待办：悬停条目显示删除按钮，点击直接删除；无确认弹窗、无撤回 | 需求结晶确认 | confirmed |
| FACT-008 | 待办数据本地持久化，应用或系统重启后列表保留 | 需求结晶确认 | confirmed |
| FACT-009 | 卡片界面文案使用中文，一期不做多语言切换 | 需求结晶确认 | confirmed |

## Hypotheses

| Hypothesis ID | Statement | Status | Evidence | Falsification Method |
| --- | --- | --- | --- | --- |
| HYP-001 | 用户期望应用单实例运行：重复启动只保留一个托盘图标和一份数据 | confirmed | 用户于 2026-08-26 在进入 Design 前显式确认单实例运行 | 已完成；结论已落入 Design Handoff Confirmed Technical Constraints |
| HYP-002 | Rust 数据层（增删改查）可作为独立单元测试接缝，覆盖 Tauri 运行环境 | open | Tauri 将核心逻辑与 command 层解耦是常见实践；未经本项目验证 | Design 阶段验证将数据层从 Tauri API 依赖中解耦后 cargo test 可直接运行；若无法解耦则降级为集成测试 |

## Goals

- 交付一个可运行、可打包的跨平台（macOS + Windows）托盘待办工具第一期。
- 应用启动后托盘/菜单栏出现图标，点击弹出待办卡片，再次点击隐藏，失焦自动关闭。
- 支持添加待办（输入框 + 回车）、勾选完成（划线保留、可取消）、悬停删除（直接删除）。
- 待办数据本地持久化，重启后恢复。
- 界面文案为中文。

## Success Criteria

| Success ID | Observable Criterion | Acceptance Source |
| --- | --- | --- |
| SUCCESS-001 | 应用启动后系统托盘/菜单栏出现应用图标，点击图标弹出待办卡片，再次点击图标卡片隐藏 | spec scenario / UI evidence |
| SUCCESS-002 | 卡片显示期间点击桌面或其他应用（失焦），卡片自动关闭 | spec scenario / UI evidence |
| SUCCESS-003 | 输入非空内容并回车后，新待办加入列表；空输入回车不产生任何条目 | spec scenario / UI evidence / unit test |
| SUCCESS-004 | 勾选某待办后该条目呈现完成状态（划线置灰），可再次点击取消勾选恢复未完成 | spec scenario / UI evidence / unit test |
| SUCCESS-005 | 悬停条目出现删除按钮，点击后该条目从列表消失，数据同时持久化 | spec scenario / UI evidence / unit test |
| SUCCESS-006 | 退出并重新启动应用后，待办列表与退出前一致（内容和完成状态） | spec scenario / UI evidence / unit test |

## Non-Goals

- 不做数据同步、云服务、账号与多设备。
- 不做提醒、通知、日程或到期日。
- 不做多列表、分类、标签、搜索、排序或“清除已完成”。
- 不做既有待办文本的编辑（删除后只能重新输入）。
- 不做 Linux 平台（一期只交付 macOS + Windows，但选型架构上不阻塞）。
- 不做多语言/i18n、设置界面、自定义图标/尺寸/主题。
- 不做窗口置顶、多窗口、常驻卡片或虚拟桌面管理。

## Deferred Items

| Deferred ID | Item | Reason | Revisit Trigger | Target Change |
| --- | --- | --- | --- | --- |
| DEF-001 | Linux 平台支持 | 一期确认只覆盖 macOS + Windows | 出现 Linux 用户需求或打包通道 | 后续 change |
| DEF-002 | 数据同步/云/多设备 | 超出第一期范围 | 用户需要跨设备时 | 后续 change |
| DEF-003 | 清除已完成、编辑、排序、多列表等交互扩展 | 超出第一期范围 | 用户提出整理类需求时 | 后续 change |
| DEF-004 | 待办提醒/通知 | 超出第一期范围 | 用户需要到期提醒时 | 后续 change |
| DEF-005 | Windows 真实验证环境 | 开发机为 macOS；Windows 需真实环境或 CI runner | Apply 阶段验证前必须确定 | 当前 change 的验证部分 |

## What Changes

### User-Visible Behavior

全新工具，无既有功能被改变；从零新增以下可观察行为：

- 系统托盘/菜单栏常驻图标；点击打开/关闭待办卡片窗口；卡片失焦自动隐藏。
- 卡片为无边框小窗口，含中文界面文案：标题栏区、输入框（占位中文提示）、待办条目列表。
- 待办条目支持：勾选完成（划线置灰、可取消）、悬停删除（直接删除）。
- 所有变更即时落盘，重启后列表与退出前一致。

### Compatibility Commitments

- 全新项目，无既有 API、数据格式或部署方式的兼容承诺。
- 本地数据格式由 Design 决定，一期不承诺外部可读、可迁移或跨版本兼容。
- 不引入 Tauri v2 及官方插件之外的网络或外部服务依赖。

### Data / API / Operational Impact

- 新增本地待办数据文件，路径由 Design 决定（候选：平台标准应用数据目录），仅本机可读写。
- 新增 Rust command API（候选命令语义：list / add / toggle / remove），供前端界面调用。
- 新增托盘图标与弹出窗口资源（图标文件、窗口配置）。
- 无网络请求、无账号、无环境变量需求。

## Capabilities

| Capability ID | Type | Description | Spec Path |
| --- | --- | --- | --- |
| CAP-001 | new | 托盘常驻与待办卡片弹出/隐藏/失焦关闭 | `specs/tray-card/spec.md` |
| CAP-002 | new | 待办列表管理：添加、勾选完成（可取消）、悬停删除、本地持久化 | `specs/todo-management/spec.md` |

## Constraints and Confirmed Commitments

- 技术栈固定为 Tauri v2，不得由 Design 推翻为其他框架。
- 一期平台范围固定为 macOS + Windows；架构不得阻塞未来 Linux 接入。
- 已确认的用户可见交互（FACT-003 至 FACT-007、FACT-009）为需求约束，不得由 Design 静默改变；交互细节（样式、动效、布局）由 Design 决定。
- 本地持久化（FACT-008）为硬性要求。
- 不引入任何需要账号、联网或付费的外部服务。

## Change Split and Dependencies

| Related Change | Relationship | Ordering | Shared Constraint |
| --- | --- | --- | --- |
| none | 单一 change 承载第一期全部范围，无拆分 | 不适用 | 不适用 |

## Impact Inventory

- Code areas: 新建 Tauri v2 项目（`src-tauri/` Rust 后端 + 前端目录）；托盘与弹出窗口；待办数据层
- APIs / events / commands: 新增 Tauri commands（候选：list/add/toggle/remove）；托盘点击、窗口失焦事件
- Dependencies: tauri v2（含 tray 能力）、Rust 工具链；系统 WebView（macOS WKWebView / Windows WebView2）
- Data / migrations: 全新本地待办数据文件，无既有数据、无迁移
- Systems / deployments: 桌面调试构建；打包产物（macOS app/.dmg、Windows .exe/nsis，具体由 Design 与后续阶段决定）
- Users / operators: 用户本人（本地单人桌面使用）

## Design Handoff

### Confirmed Technical Constraints

- 技术栈：Tauri v2（Rust 后端 + Web 前端）。
- 平台：macOS + Windows；预留 Linux 演进空间（不得采用锁定单平台的 API 形态）。
- 本地持久化必须；中文 UI 文案必须；交互行为按 FACT-003 ~ FACT-007、FACT-009。
- 单实例运行：用户于 2026-08-26 确认；重复启动只保留一个托盘图标与一份数据，Design 须落实对应方案。

### Candidate Technical Paths

- 托盘常驻：Tauri v2 `TrayIcon` API（官方跨平台支持 macOS 菜单栏 / Windows 托盘）。
- 弹出卡片：候选 A：独立无边框小窗口（`decorations: false`）+ `show`/`hide` 切换，配合失焦（blur）事件自动隐藏；候选 B：TrayIcon 原生 menu；候选 C：menu + 窗口组合。适用条件：候选 A 最接近“卡片”体验且可自定义样式，候选 B 适合纯菜单场景。
- 单实例：`tauri-plugin-single-instance`（候选；HYP-001 待确认）。
- 数据层：候选 A：Rust 侧 JSON 文件 + serde（含原子写入）；候选 B：rusqlite；候选 C：`tauri-plugin-store`（键值包装）。适用条件：第一期条目量小，A 最简单且可单元测试；B 适合未来复杂查询；C 耦合 Tauri 运行环境。
- 前端：候选 A：原生 HTML/CSS/JS（零构建）；候选 B：Vite + 轻量框架（React/Vue）。适用条件：A 最快、依赖最少；B 适合后续界面复杂度增长。
- 平台验证：macOS 本机手动验收；Windows 候选：GitHub Actions Windows runner 或远程 Windows 环境（DEF-005）。

### Candidate Test Seams

- SEAM（候选）：Rust 待办数据层单元测试 —— add/toggle/remove/持久化往返（`cargo test`），要求数据层不依赖 Tauri 运行时（对应 HYP-002）。
- SEAM（候选）：前端纯逻辑（如输入有效性判断）可抽取纯函数单测；一期以手动 UI 验收为主。
- 平台行为验证：macOS 本机手动验收（托盘、显隐、失焦、勾选、删除、重启恢复）；Windows 按 DEF-005 落实。

### Decisions Reserved for Design

- 数据文件格式、字段、存储路径与写入策略（原子性）。
- 弹出窗口尺寸、样式、布局与交互细节（悬停按钮呈现、滚动、动效）。
- 终端前端技术方案（原生 vs 框架）与项目结构。
- 单实例方案确认（HYP-001 结论落地）。
- 打包配置与产物类型（macOS app/.dmg、Windows 安装器）。
- 最终 SEAM-* 定义与验证命令。
- HYP-002 的数据层解耦方式验证。

### Open Questions

- 无阻断性问题。HYP-001（单实例）已于 2026-08-26 在 Design 前由用户确认（confirmed，见上）；DEF-005（Windows 验证环境）不阻断 Design/Tasks 的推进，但必须在 Apply 验证前确定方式。