# todos

系统托盘常驻的轻量待办卡片（Tauri v2）。

- 点击托盘/菜单栏图标：切换待办卡片显示/隐藏
- 输入框回车添加待办，空输入不添加
- 勾选完成：划线置灰保留，可取消
- 悬停条目显示删除按钮；双击待办文本可修改，点击 × 可直接删除
- 本地 JSON 持久化，重启后恢复
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
│  └─ src/
│     ├─ main.rs / lib.rs     # 应用装配
│     ├─ commands.rs          # Tauri command 层
│     ├─ tray.rs              # 托盘图标
│     └─ card.rs              # 无边框卡片窗口
└─ openspec/changes/todos/  # OpenSpec 设计/原型制品
```

## 环境要求

- **Rust 1.85+**：依赖链包含使用 `edition2024` 的 crate（如 getrandom/uuid），旧工具链（如 1.79）无法编译；推荐 1.92.0。
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

# 全量 Rust 检查（Tauri 侧编译验证）
cd src-tauri && cargo check

# 开发运行（需先安装 Tauri CLI，如 `cargo install tauri-cli --locked`）
cd src-tauri && cargo tauri dev

# 打包（macOS .app/.dmg；Windows 请在 Windows/CI 环境构建）
cd src-tauri && cargo tauri build
```

## 数据文件

待办数据保存在平台应用数据目录下（`app_data_dir/todos.json`，由 `identifier` 决定），使用 tmp + rename 原子写入；损坏或版本不符的文件会备份为 `todos.json.corrupt-<unix秒>` 后以空库启动。
