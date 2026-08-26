# Capability: CAP-001 托盘常驻与待办卡片

## ADDED Requirements

### Requirement: REQ-TRAY-001 — 托盘图标常驻

The system MUST keep a tray icon visible in the system tray area (macOS menu bar / Windows notification area) for the entire time the application is running.

- Source Capability: CAP-001
- Success Criteria: SUCCESS-001

应用启动后，系统托盘/菜单栏区域持续显示应用图标，作为打开待办卡片的唯一入口。

#### Scenario: SCN-TRAY-001-A — 应用启动后出现托盘图标

- **GIVEN** 应用已启动且未退出
- **WHEN** 用户查看系统托盘/菜单栏区域
- **THEN** 托盘区域出现应用图标
- **Expected Values:** 图标可见且可点击交互
- **Boundary / Failure Semantics:** 应用退出后图标消失；应用运行期间图标不得消失或重复出现

### Requirement: REQ-TRAY-002 — 点击图标切换卡片显示/隐藏

The system MUST toggle the todo card window between shown and hidden each time the user clicks the tray icon.

- Source Capability: CAP-001
- Success Criteria: SUCCESS-001

#### Scenario: SCN-TRAY-002-A — 卡片隐藏时点击图标显示卡片

- **GIVEN** 应用运行中，卡片窗口处于隐藏状态
- **WHEN** 用户点击托盘图标
- **THEN** 卡片窗口显示，位置靠近托盘图标
- **Expected Values:** 卡片可见，包含输入框与待办列表（若已有待办则完整显示）
- **Boundary / Failure Semantics:** 卡片显示后获得输入焦点，用户可直接开始输入

#### Scenario: SCN-TRAY-002-B — 卡片显示时点击图标隐藏卡片

- **GIVEN** 应用运行中，卡片窗口处于显示状态
- **WHEN** 用户再次点击托盘图标
- **THEN** 卡片窗口隐藏，应用仍驻留托盘
- **Expected Values:** 卡片不可见且不残留窗口；应用进程不退出
- **Boundary / Failure Semantics:** 多次往返切换不产生重复窗口或状态错乱

### Requirement: REQ-TRAY-003 — 失焦自动关闭卡片

The system MUST hide the todo card window automatically when the card loses focus, such as when the user clicks on the desktop, another application window, or any area outside the card.

- Source Capability: CAP-001
- Success Criteria: SUCCESS-002

#### Scenario: SCN-TRAY-003-A — 点击卡片外部区域后卡片自动关闭

- **GIVEN** 卡片窗口显示中
- **WHEN** 用户点击卡片窗口之外的任意区域（桌面、其他应用、系统区域）
- **THEN** 卡片窗口自动隐藏，且不残留任何部分
- **Expected Values:** 卡片完全不可见；再次点击托盘图标可重新显示
- **Boundary / Failure Semantics:** 隐藏瞬间输入框中未通过回车提交的内容丢弃，不产生待办且不持久化；隐藏不丢失已存在的待办列表