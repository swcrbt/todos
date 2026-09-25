# todos

系统托盘常驻的轻量待办卡片（Tauri v2）。

- 点击托盘/菜单栏图标：切换待办卡片显示/隐藏
- 输入框回车添加待办，空输入不添加
- 勾选完成：划线置灰保留，可取消
- 悬停条目显示删除按钮；双击待办文本可修改，点击 × 可直接删除
- 本地 JSON 持久化，重启后恢复
- MCP 服务器（stdio 传输，无端口）：AI 客户端可直接操作待办列表，与桌面应用共享同一份数据
- 中文界面

## 项目结构

```
todos/
├─ src/                       # 前端（零构建 ES Modules）
│  ├─ index.html
│  ├─ styles.css
│  ├─ app.mjs                 # 界面交互 + Tauri invoke
│  ├─ logic.mjs               # 纯逻辑（Node 可测）
│  └─ logic.test.mjs
├─ src-tauri/                 # Tauri v2 后端
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  ├─ capabilities/           # Tauri ACL 权限
│  ├─ crates/todo-store/      # 待办数据层（独立 crate，零 Tauri 依赖）
│  ├─ crates/todos-mcp/       # MCP 服务器（stdio 传输，零 Tauri 依赖）
│  └─ src/
│     ├─ main.rs / lib.rs     # 应用装配
│     ├─ commands.rs          # Tauri command 层
│     ├─ tray.rs              # 托盘图标
│     └─ card.rs              # 无边框卡片窗口
└─ openspec/changes/todos/  # OpenSpec 设计/原型制品
```

## 环境要求

- **Rust 1.89+**：数据层使用标准库跨进程文件锁（`File::lock`，1.89 稳定）；依赖链包含使用 `edition2024` 的 crate（如 getrandom/uuid），旧工具链（如 1.79）无法编译；推荐 1.92.0。
- **Node.js 20+**：仅用于前端纯逻辑单测，项目零构建、无 npm 依赖。
- **Tauri v2 桌面工具链**：
  - macOS：Xcode Command Line Tools
  - Windows：WebView2 Runtime
- rustup 用户可固定默认工具链：

```bash
rustup default 1.92.0
# 或单次使用：
cargo +1.92.0 check
```

## 开发命令

```bash
# 前端纯逻辑单测（Node 内置 test runner，零依赖）
node --test src/logic.test.mjs

# 数据层单测（无需 Tauri 运行时）
cd src-tauri && cargo test -p todo-store

# 构建 MCP 服务器（产物：target/debug/todos-mcp 或 target/release/todos-mcp）
cd src-tauri && cargo build -p todos-mcp --release

# 全量 Rust 检查（Tauri 侧编译验证）
cd src-tauri && cargo check

# 开发运行（需先安装 Tauri CLI，如 `cargo install tauri-cli --locked`）
cd src-tauri && cargo tauri dev

# 打包（macOS .app/.dmg；Windows 请在 Windows/CI 环境构建）
cd src-tauri && cargo tauri build
```

## 数据文件

待办数据保存在平台应用数据目录下（`app_data_dir/todos.json`，由 `identifier` 决定），使用 tmp + rename 原子写入；损坏或版本不符的文件会备份为 `todos.json.corrupt-<unix秒>` 后以空库启动。

桌面应用与 MCP 服务器通过 `<数据文件>.lock` 跨进程排他锁协调读写：任何一方变更前先持锁并从磁盘重载，避免并发互相覆盖。

## MCP 服务器（AI 操作待办）

`todos-mcp` 是一个标准 MCP（Model Context Protocol）服务器，使用 stdio 传输（换行分隔 JSON-RPC），**不监听任何端口**。支持 MCP 的 AI 客户端（Claude Desktop、Cursor 等）以子进程方式启动它，即可操作与桌面应用共享的同一份待办列表。

提供五个工具：`list_todos`（列出全部）、`add_todo`（新增）、`toggle_todo`（切换完成状态）、`update_todo`（修改文本）、`remove_todo`（删除）。

### 构建

```bash
cd src-tauri && cargo build -p todos-mcp --release
# 产物：src-tauri/target/release/todos-mcp
```

### 客户端配置示例（Claude Desktop / Cursor 等）

```json
{
  "mcpServers": {
    "todos": {
      "command": "/绝对路径/todos/src-tauri/target/release/todos-mcp"
    }
  }
}
```

默认读写平台应用数据目录下的 `todos.json`（与桌面应用一致）；设置环境变量 `TODOS_DATA_PATH` 可指定其他数据文件：

```json
{
  "mcpServers": {
    "todos": {
      "command": "/绝对路径/todos-mcp",
      "env": { "TODOS_DATA_PATH": "/自定义路径/todos.json" }
    }
  }
}
```

桌面应用运行期间 AI 的改动会实时写入同一数据文件；再次打开待办卡片时界面自动刷新为最新内容。
