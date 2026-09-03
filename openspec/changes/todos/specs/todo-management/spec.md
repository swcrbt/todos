# Capability: CAP-002 待办列表管理

## ADDED Requirements

### Requirement: REQ-TODO-001 — 回车添加非空待办

The system MUST add a new pending todo item with the entered text to the list when the user presses Enter in the input field with non-whitespace-only content, and MUST NOT create any item when the input is empty or whitespace-only.

- Source Capability: CAP-002
- Success Criteria: SUCCESS-003

#### Scenario: SCN-TODO-001-A — 输入内容回车后添加待办

- **GIVEN** 卡片显示中，待办列表当前为空
- **WHEN** 用户在输入框中输入“买牛奶”并按下回车
- **THEN** 列表新增一条内容为“买牛奶”的未完成待办，且输入框清空
- **Expected Values:** 列表长度为 1；条目文本精确等于“买牛奶”；条目状态为 pending；输入框内容为空字符串
- **Boundary / Failure Semantics:** 连续回车 N 次非空内容，列表依次新增 N 条；添加后新条目立即可见

#### Scenario: SCN-TODO-001-B — 空输入回车不添加

- **GIVEN** 卡片显示中，输入框为空或仅包含空白字符
- **WHEN** 用户按下回车
- **THEN** 列表不增加任何条目，且无报错或异常提示
- **Expected Values:** 列表长度保持不变；无空文本条目产生
- **Boundary / Failure Semantics:** 仅包含空格、Tab 等空白字符视为空输入

### Requirement: REQ-TODO-002 — 勾选完成且可取消

The system MUST mark a todo item as completed when the user clicks its checkbox, MUST display completed items with strikethrough and grayed-out styling while keeping them in the list, and MUST restore a completed item to pending state when the user clicks its checkbox again.

- Source Capability: CAP-002
- Success Criteria: SUCCESS-004

#### Scenario: SCN-TODO-002-A — 勾选后条目完成并保留在列表

- **GIVEN** 列表中恰有 1 条未完成待办“开会”
- **WHEN** 用户点击该条目的勾选框
- **THEN** 该条目状态变为 completed，呈现划线置灰样式，并继续保留在列表中
- **Expected Values:** 条目状态为 completed；样式为划线加置灰；列表长度仍为 1
- **Boundary / Failure Semantics:** 完成条目不得自动消失、不得被清除，仍可被勾选取消、被删除

#### Scenario: SCN-TODO-002-B — 取消勾选恢复未完成

- **GIVEN** 列表中恰有 1 条已完成待办“开会”
- **WHEN** 用户再次点击该条目的勾选框
- **THEN** 该条目恢复为 pending 状态，样式恢复正常（无划线、正常颜色）
- **Expected Values:** 条目状态为 pending；列表长度仍为 1
- **Boundary / Failure Semantics:** 连续切换 N 次后最终状态与初始状态一致（N 为偶数）或相反（N 为奇数）

### Requirement: REQ-TODO-003 — 悬停显示删除按钮并直接删除

The system MUST reveal a delete button on an individual todo item when the user hovers over that item, and MUST remove the item from the list immediately when the user clicks the delete button, without showing any confirmation dialog.

- Source Capability: CAP-002
- Success Criteria: SUCCESS-005

#### Scenario: SCN-TODO-003-A — 悬停条目出现删除按钮

- **GIVEN** 列表中至少有 2 条待办
- **WHEN** 用户鼠标悬停在其中一条上
- **THEN** 被悬停的条目显示删除按钮，其他条目不显示
- **Expected Values:** 被悬停条目的删除按钮可见；未被悬停条目不出现删除按钮
- **Boundary / Failure Semantics:** 鼠标移出该条目后删除按钮隐藏

#### Scenario: SCN-TODO-003-B — 点击删除按钮条目消失并持久化

- **GIVEN** 列表中恰有 1 条待办“买牛奶”
- **WHEN** 用户点击该条目的删除按钮
- **THEN** 该条目立即从列表消失，删除结果被持久化，且全程无确认弹窗
- **Expected Values:** 列表长度变为 0；无确认对话框；应用重启后该条目不再出现
- **Boundary / Failure Semantics:** 删除不可撤回；已完成与未完成条目均可删除；删除不连带影响其他条目

### Requirement: REQ-TODO-004 — 本地持久化重启恢复

The system MUST persist todo items, including their text and completion state, to local storage whenever they change, and MUST restore the same list with identical text and completion state after the application restarts.

- Source Capability: CAP-002
- Success Criteria: SUCCESS-006

#### Scenario: SCN-TODO-004-A — 重启后列表与退出前一致

- **GIVEN** 用户添加了 3 条待办且第 2 条已勾选完成，随后退出应用
- **WHEN** 用户重新启动应用并打开待办卡片
- **THEN** 卡片显示与退出前完全相同的 3 条待办，且第 2 条仍为完成状态
- **Expected Values:** 列表长度为 3；每条文本与退出前一致；第 2 条状态为 completed，其余为 pending
- **Boundary / Failure Semantics:** 应用被强制结束（进程被杀）前已提交的变更也必须恢复；带中文文本的条目在重启后无乱码