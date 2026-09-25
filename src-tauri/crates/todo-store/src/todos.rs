// todo-store：待办数据层。
// 零 Tauri 依赖的独立 crate，可直接以 `cargo test -p todo-store` 脱离 Tauri 运行环境运行。
// 本文件承载数据模型（TodoItem/TodoFile/TodoError）、加载（load/list）、
// 增删改（add/toggle/remove）与原子持久化（persist）的完整实现。

use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// 环境变量：直接指定数据文件完整路径（MCP 进程/测试覆盖默认解析用）。
pub const DATA_PATH_ENV: &str = "TODOS_DATA_PATH";

/// 解析默认数据文件路径：平台应用数据目录/<identifier>/todos.json。
/// 解析规则与 Tauri v2 `app_data_dir` 保持一致（macOS: ~/Library/Application Support，
/// Windows: %APPDATA%，Linux: $XDG_DATA_HOME 或 ~/.local/share），
/// 使独立进程（如 MCP 服务器）无需 Tauri 运行时即可定位同一份数据文件。
/// 设置环境变量 TODOS_DATA_PATH 时直接返回其值（非空），便于测试与自定义部署。
pub fn data_file_path(identifier: &str) -> Result<PathBuf, TodoError> {
    if let Ok(custom) = std::env::var(DATA_PATH_ENV) {
        let custom = custom.trim();
        if !custom.is_empty() {
            return Ok(PathBuf::from(custom));
        }
    }
    let base = platform_data_dir().ok_or_else(|| {
        TodoError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "无法定位平台应用数据目录（缺少 HOME/APPDATA 环境变量）",
        ))
    })?;
    Ok(base.join(identifier).join("todos.json"))
}

/// 平台应用数据目录（不含 identifier 后缀），规则对齐 Tauri/dirs crate。
#[cfg(target_os = "macos")]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
}

/// Windows：Roaming 应用数据目录（%APPDATA%）。
#[cfg(target_os = "windows")]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

/// Linux/其他 Unix：遵循 XDG 规范（$XDG_DATA_HOME 或 ~/.local/share）。
#[cfg(all(unix, not(target_os = "macos")))]
fn platform_data_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg));
        }
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
}

/// 数据文件跨进程排他锁守卫（advisory lock，进程崩溃由 OS 自动释放）。
/// 持有期间其他协作进程（App 与 MCP 服务器）的 lock_and_reload 会等待；
/// Drop 即释放。锁对象是数据文件旁的 `<name>.lock` 占位文件，
/// 不锁数据文件本身（persist 用 rename 原子替换，文件主体会变）。
#[derive(Debug)]
pub struct StoreLock {
    _file: File,
}

/// 单个待办条目（字段契约：id/text/done/created_at）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoItem {
    /// UUID v4 字符串标识（全局唯一、永不复用，支撑删除不可撤回语义）。
    pub id: String,
    /// 待办文本（trim 后保存，可为中文，UTF-8）
    pub text: String,
    /// 完成状态：false=未完成，true=已完成（划线置灰保留，可取消）
    pub done: bool,
    /// 创建时间戳（unix 秒），用于未来排序/展示，一期只做追加顺序
    pub created_at: i64,
}

/// 数据文件顶层结构：version 字段预留给未来迁移。
#[derive(Debug, Serialize, Deserialize)]
pub struct TodoFile {
    /// 数据格式版本：当前仅支持 1；不符视为损坏触发备份恢复
    pub version: u32,
    /// 条目列表，顺序 = 添加顺序
    pub items: Vec<TodoItem>,
}

/// 数据层错误（错误语义：业务错误与磁盘错误的显式区分）。
#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    /// 空/纯空白文本不允许产生条目
    #[error("内容为空，无法添加")]
    InvalidInput,
    /// 对不存在的 id 执行 toggle/remove
    #[error("待办不存在")]
    NotFound,
    /// 数据文件损坏/版本不符（load 内触发备份恢复流程，一般不外泄为启动错误）
    #[error("数据文件损坏")]
    CorruptData,
    /// 磁盘读写失败（读取权限、目录不可写等）
    #[error("磁盘读写失败：{0}")]
    Io(#[from] io::Error),
    /// JSON 序列化/反序列化失败（序列化几乎不发生，与腐坏数据同源）
    #[error("数据序列化失败：{0}")]
    Serde(#[from] serde_json::Error),
}

/// TodoStore：待办列表的内存态 + 数据文件路径。
/// 生命周期：单实例进程内单写者；所有变更方法成功后立即持久化。
#[derive(Debug, Clone)]
pub struct TodoStore {
    /// 数据文件完整路径（应用数据目录下的 todos.json）
    path: PathBuf,
    /// 追加顺序内存列表（顺序 = 添加顺序）
    items: Vec<TodoItem>,
}

impl TodoStore {
    /// 从数据文件加载。
    /// 语义：
    /// - 文件不存在 → 空库（首次启动）
    /// - JSON 损坏或 version != 1 → 原文件改名备份为 `<name>.corrupt-<unix秒>`，返回空库（应用不崩溃，stderr 记录）
    /// - 其他读取错误（权限等）→ Err(Io)，由启动层决定是否失败退出
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TodoError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self {
                path: path.to_path_buf(),
                items: Vec::new(),
            });
        }
        let raw = fs::read_to_string(path)?;
        match serde_json::from_str::<TodoFile>(&raw) {
            Ok(file) if file.version == 1 => Ok(Self {
                path: path.to_path_buf(),
                items: file.items,
            }),
            Ok(file) => {
                // 未来版本（version > 1）：备份后以空库启动（版本迁移预留）
                eprintln!(
                    "todo-store: 数据文件版本 {0} 不受支持，备份并重置",
                    file.version
                );
                Self::backup_corrupt(path);
                Ok(Self {
                    path: path.to_path_buf(),
                    items: Vec::new(),
                })
            }
            Err(e) => {
                // JSON 结构损坏：备份 + 空库，不使应用崩溃（Reliability 节）
                eprintln!("todo-store: 数据文件损坏，备份并重置（{e}）");
                Self::backup_corrupt(path);
                Ok(Self {
                    path: path.to_path_buf(),
                    items: Vec::new(),
                })
            }
        }
    }

    /// 返回追加顺序的完整列表副本。
    pub fn list(&self) -> Vec<TodoItem> {
        self.items.clone()
    }

    /// 从磁盘重新加载并覆盖内存态（损坏恢复语义与 load 一致）。
    /// 用于多进程（App + MCP 服务器）共享数据文件时感知对方写入。
    pub fn reload(&mut self) -> Result<(), TodoError> {
        let fresh = Self::load(&self.path)?;
        self.items = fresh.items;
        Ok(())
    }

    /// 获取数据文件的跨进程排他锁并 reload，返回锁守卫（Drop 释放）。
    /// 协作进程（App 命令层、MCP 服务器）的「读-改-写」应在持锁期间完成，
    /// 避免两个进程基于过期内存态互相覆盖写入（丢失更新）。
    pub fn lock_and_reload(&mut self) -> Result<StoreLock, TodoError> {
        let lock_path = self.path.with_file_name(format!(
            "{}.lock",
            self.path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "todos.json".to_string())
        ));
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(&lock_path)?;
        file.lock()?;
        self.reload()?;
        Ok(StoreLock { _file: file })
    }

    /// 添加待办。
    /// 语义：
    /// - 文本 trim 后为空 → Err(InvalidInput)（后端兜底）
    /// - 非空 → UUID v4 + unix 秒 created_at + done=false，追加队尾
    /// - 追加成功后立即原子持久化；persist 失败返回 Err（UI 层保证界面不更新）
    pub fn add(&mut self, text: String) -> Result<TodoItem, TodoError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(TodoError::InvalidInput);
        }
        let item = TodoItem {
            id: Uuid::new_v4().to_string(),
            text: trimmed.to_string(),
            done: false,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        };
        // 先持久化副本，成功后才提交内存（INV-002：失败不改内存）
        let mut next = self.items.clone();
        next.push(item.clone());
        self.persist(&next)?;
        self.items = next;
        Ok(item)
    }

    /// 切换完成状态。
    /// 语义：id 不存在 → Err(NotFound)；存在 → 置反 done 并立即持久化；
    /// persist 失败 → 返回 Err（UI 层保持界面不变）。返回切换后的条目供前端幂等渲染。
    pub fn toggle(&mut self, id: &str) -> Result<TodoItem, TodoError> {
        let mut next = self.items.clone();
        let item = next
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or(TodoError::NotFound)?;
        item.done = !item.done;
        let result = item.clone();
        self.persist(&next)?;
        self.items = next;
        Ok(result)
    }

    /// 修改待办文本。
    /// 语义：id 不存在 → Err(NotFound)；trim 后为空 → Err(InvalidInput)；
    /// 存在且非空 → 更新 text 并立即持久化；persist 失败 → 返回 Err（内存不变）。
    pub fn update(&mut self, id: &str, text: String) -> Result<TodoItem, TodoError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(TodoError::InvalidInput);
        }
        let mut next = self.items.clone();
        let item = next
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or(TodoError::NotFound)?;
        item.text = trimmed.to_string();
        let result = item.clone();
        self.persist(&next)?;
        self.items = next;
        Ok(result)
    }

    /// 删除待办。
    /// 语义：id 不存在 → Err(NotFound)；存在 → 移除并立即持久化；
    /// 删除不可撤回（UI 无确认/无撤销入口，id 永不复用）。完成与未完成条目均可删。
    pub fn remove(&mut self, id: &str) -> Result<(), TodoError> {
        let mut next = self.items.clone();
        let idx = next
            .iter()
            .position(|i| i.id == id)
            .ok_or(TodoError::NotFound)?;
        next.remove(idx);
        self.persist(&next)?;
        self.items = next;
        Ok(())
    }

    /// 原子持久化：序列化指定条目快照为完整 TodoFile 写入 `<path>.tmp`，
    /// 随后 `fs::rename` 原子替换正式文件；任何时刻磁盘上要么旧文件、要么完整新文件。
    /// 失败返回 Err（磁盘错误 → IoError / 序列化错误 → Serde），tmp 文件可能残留但下次 persist 覆盖。
    /// 调用方须在成功后才提交内存状态（INV-002：失败不改内存）。
    fn persist(&self, items: &[TodoItem]) -> Result<(), TodoError> {
        // 首次使用时数据目录可能不存在（应用数据目录），先确保父目录
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = TodoFile {
            version: 1,
            items: items.to_vec(),
        };
        let json = serde_json::to_string_pretty(&file)?;
        let tmp = PathBuf::from(format!("{}.tmp", self.path.display()));
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    /// 把损坏/版本不符的旧文件改名为 `<name>.corrupt-<unix秒>`（Reliability：备份恢复）。
    fn backup_corrupt(path: &Path) {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "todos.json".to_string());
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let backup = path.with_file_name(format!("{file_name}.corrupt-{ts}"));
        if let Err(e) = fs::rename(path, &backup) {
            eprintln!("todo-store: 备份损坏文件失败：{e}");
        }
    }
}

/// 测试模块：加载场景（空文件、损坏恢复、版本不符、UTF-8 中文往返）、
/// 添加与原子持久化、toggle/remove 与变更即落盘。
/// 场景说明：
/// 1. 数据文件不存在 → load 返回空库（首次启动）
/// 2. 数据文件 JSON 损坏 → 备份为 todos.json.corrupt-<timestamp>，返回空库，应用不崩溃
/// 3. 数据文件 version 不符 → 同上备份并空库
/// 4. 合法 JSON（含中文）→ load 解码后中文无乱码
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::ops::Deref;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 测试临时目录守卫：Drop 时自动清理，避免残留。
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            static SEQ: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "todo-store-test-{}-{}-{}",
                std::process::id(),
                name,
                SEQ.fetch_add(1, Ordering::SeqCst)
            ));
            fs::create_dir_all(&dir).expect("测试前置：创建临时目录");
            Self(dir)
        }
    }

    impl Deref for TempDir {
        type Target = Path;

        fn deref(&self) -> &Path {
            &self.0
        }
    }

    impl AsRef<Path> for TempDir {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// 场景 1：path 不存在（首次启动）→ load 成功且列表为空。
    #[test]
    fn load_missing_file_returns_empty_store() {
        let dir = TempDir::new("missing");
        let path = dir.join("todos.json");
        let store = TodoStore::load(&path).expect("文件缺失应返回空库而非错误");
        assert!(store.list().is_empty(), "文件缺失时列表应为空");
    }

    /// 场景 2：JSON 损坏 → 原文件被改名备份（.corrupt-<timestamp>），load 返回空库。
    #[test]
    fn load_corrupt_json_backs_up_and_returns_empty() {
        let dir = TempDir::new("corrupt");
        let path = dir.join("todos.json");
        fs::write(&path, "{ 这不是合法 JSON ").expect("测试前置：写入损坏文件");
        let store = TodoStore::load(&path).expect("损坏文件应备份后返回空库");
        assert!(store.list().is_empty(), "损坏恢复后列表应为空");
        // 备份文件应出现在同目录，且以 .corrupt- 为后缀
        let entries = fs::read_dir(&dir)
            .expect("目录可读")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(
            entries.iter().any(|f| f.starts_with("todos.json.corrupt-")),
            "应生成备份文件，实际目录内容：{entries:?}"
        );
    }

    /// 场景 3：version 不符（版本迁移预留）→ 备份并空库。
    #[test]
    fn load_version_mismatch_backs_up_and_returns_empty() {
        let dir = TempDir::new("version");
        let path = dir.join("todos.json");
        fs::write(&path, r#"{"version": 2, "items": []}"#).expect("测试前置：写入未来版本文件");
        let store = TodoStore::load(&path).expect("版本不符应备份后返回空库");
        assert!(store.list().is_empty(), "版本不符恢复后列表应为空");
    }

    /// 场景 4：合法数据文件（中文文本）→ load 后列表与完成状态一致、中文无乱码。
    #[test]
    fn load_valid_file_preserves_chinese_text() {
        let dir = TempDir::new("valid");
        let path = dir.join("todos.json");
        fs::write(
            &path,
            r#"{"version": 1, "items": [{"id": "a1", "text": "买牛奶", "done": true, "created_at": 100}, {"id": "a2", "text": "写周报", "done": false, "created_at": 200}]}"#,
        )
        .expect("测试前置：写入合法文件");
        let store = TodoStore::load(&path).expect("合法文件应加载成功");
        let items = store.list();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text, "买牛奶", "中文文本不得乱码");
        assert_eq!(items[0].done, true);
        assert_eq!(items[1].text, "写周报", "中文文本不得乱码");
        assert_eq!(items[1].done, false);
        assert_eq!(items[0].created_at, 100);
    }

    // ===== 添加与原子持久化场景 =====

    /// 场景 5：非空添加。
    /// 预期：add 返回条目（pending、非空 id、created_at > 0），连续添加顺序 = 添加顺序。
    #[test]
    fn add_appends_in_order_and_returns_item() {
        let dir = TempDir::new("add-order");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");

        let item1 = store.add("买牛奶".to_string()).expect("非空添加应成功");
        let item2 = store
            .add("  开会  ".to_string())
            .expect("带首尾空格添加应成功");

        assert!(!item1.id.is_empty(), "应生成 uuid id");
        assert_eq!(item1.text, "买牛奶");
        assert_eq!(item1.done, false);
        assert!(item1.created_at > 0, "created_at 应为 unix 秒");
        assert_eq!(item2.text, "开会", "文本应 trim 后保存");

        let items = store.list();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text, "买牛奶", "顺序 = 添加顺序");
        assert_eq!(items[1].text, "开会");
    }

    /// 场景 6：空/纯空白文本拒绝。
    /// 预期：空串、纯空格、纯 Tab 均返回 InvalidInput 且列表不变。
    #[test]
    fn add_rejects_empty_and_whitespace() {
        let dir = TempDir::new("add-blank");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");

        for blank in ["", "   ", "\t", "\n"] {
            let err = store.add(blank.to_string()).expect_err("空白文本应被拒绝");
            assert!(
                matches!(err, TodoError::InvalidInput),
                "空白输入应返回 InvalidInput，实际: {err:?}"
            );
        }
        assert!(store.list().is_empty(), "空白输入不得改变列表");
    }

    /// 场景 7：persist（原子写 tmp+rename）后重新 load 内容一致。
    /// 预期：中文文本无乱码、done 状态保留、追加顺序保留。
    #[test]
    fn persist_then_reload_roundtrip_chinese() {
        let dir = TempDir::new("roundtrip");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");
        store.add("买牛奶".to_string()).expect("添加成功");
        let done_item = store.add("写周报".to_string()).expect("添加成功");

        // 验证添加内容的磁盘往返一致性（完成态语义由 toggle 场景覆盖）
        let reloaded = TodoStore::load(&path).expect("persist 后文件可重新加载");
        let items = reloaded.list();
        assert_eq!(items.len(), 2, "重载后条目数一致");
        assert_eq!(items[0].text, "买牛奶", "中文文本重载后无乱码");
        assert_eq!(items[1].text, "写周报");
        assert_eq!(items[0].done, false);
        assert_eq!(items[1].id, done_item.id, "重载后 id 一致（条目完整往返）");
    }

    /// 场景 8：原子写中间文件清理。
    /// 预期：persist 成功后 `todos.json.tmp` 不存在；磁盘上只有最终文件。
    #[test]
    fn persist_success_leaves_no_tmp_file() {
        let dir = TempDir::new("no-tmp");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");
        store.add("第一条".to_string()).expect("添加并持久化成功");

        assert_eq!(path.exists(), true, "持久化后数据文件应存在");
        let tmp = PathBuf::from(format!("{}.tmp", path.display()));
        assert_eq!(tmp.exists(), false, "原子写成功后 tmp 文件不得残留");
    }

    // ===== 勾选切换 / 删除 + 变更即落盘场景 =====

    /// 场景 9：toggle 往返 + 持久化重载一致。
    /// 预期：第一次 toggle → done=true；第二次 toggle → done=false；重载后状态与内存一致。
    #[test]
    fn toggle_roundtrip_and_persists() {
        let dir = TempDir::new("toggle");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");
        let item = store.add("开会".to_string()).expect("添加成功");

        // pending → completed
        let toggled = store.toggle(&item.id).expect("toggle 应成功");
        assert_eq!(toggled.done, true, "toggle 后应为 completed");
        assert_eq!(toggled.id, item.id);

        // 完成态必须仍在列表中，且可取消勾选
        let items = store.list();
        assert_eq!(items.len(), 1, "完成条目保留在列表");
        assert_eq!(items[0].done, true);

        // completed → pending：再 toggle 一次
        let untoggled = store.toggle(&item.id).expect("第二次 toggle 应成功");
        assert_eq!(untoggled.done, false, "第二次 toggle 恢复 pending");
        assert_eq!(store.list()[0].done, false);

        // toggle 后已落盘：重载与内存一致
        let reloaded = TodoStore::load(&path).expect("重载成功");
        assert_eq!(reloaded.list()[0].done, false, "重载后完成状态一致");
    }

    /// 场景 10：remove 后列表减少且持久化；completed 条目可删除。
    /// 预期：删除后重载不存在；已勾选完成的条目同样可删；未连接删除不连带其他条目。
    #[test]
    fn remove_persists_and_deletes_completed() {
        let dir = TempDir::new("remove");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");
        let a = store.add("买牛奶".to_string()).expect("添加成功");
        let b = store.add("写周报".to_string()).expect("添加成功");

        // 先勾选完成再删除：completed 条目可删除
        store.toggle(&b.id).expect("toggle 成功");
        store.remove(&b.id).expect("删除完成条目应成功");

        let items = store.list();
        assert_eq!(items.len(), 1, "删除后列表减少");
        assert_eq!(items[0].id, a.id, "未删除条目不受影响");

        // 删除结果已持久化：重载后不存在（重启后该条目不再出现）
        let reloaded = TodoStore::load(&path).expect("重载成功");
        assert_eq!(reloaded.list().len(), 1, "重载后仍为 1 条");
        assert_eq!(reloaded.list()[0].id, a.id);
    }

    /// 场景 11：修改文本后内存与磁盘一致；中文往返、完成状态保持。
    #[test]
    fn update_modifies_text_and_persists() {
        let dir = TempDir::new("update");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");
        let item = store.add("买牛奶".to_string()).expect("添加成功");

        let updated = store
            .update(&item.id, "  买牛奶和面包  ".to_string())
            .expect("修改成功");
        assert_eq!(updated.id, item.id);
        assert_eq!(updated.text, "买牛奶和面包", "文本应 trim 后保存");
        assert_eq!(updated.done, false, "修改不应影响完成状态");

        let reloaded = TodoStore::load(&path).expect("重载成功");
        assert_eq!(
            reloaded.list()[0].text,
            "买牛奶和面包",
            "修改结果应已持久化"
        );
    }

    /// 场景 12：空/纯空白文本拒绝；未知 id 返回 NotFound；失败不得改变列表。
    #[test]
    fn update_rejects_empty_and_unknown_id() {
        let dir = TempDir::new("update-bad");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");
        let item = store.add("开会".to_string()).expect("添加成功");

        for blank in ["", "   ", "\t", "\n"] {
            let err = store
                .update(&item.id, blank.to_string())
                .expect_err("空白文本应被拒绝");
            assert!(
                matches!(err, TodoError::InvalidInput),
                "应为 InvalidInput，实际: {err:?}"
            );
        }
        assert!(matches!(
            store.update("不存在", "有效文本".to_string()).unwrap_err(),
            TodoError::NotFound
        ));
        assert_eq!(store.list()[0].text, "开会", "失败操作不得改变文本");
    }

    /// 场景 13：对不存在的 id（含已删除 id）执行 toggle/remove → NotFound。
    #[test]
    fn toggle_remove_unknown_id_returns_not_found() {
        let dir = TempDir::new("notfound");
        let path = dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");

        assert!(matches!(
            store.toggle("不存在").unwrap_err(),
            TodoError::NotFound
        ));
        assert!(matches!(
            store.remove("不存在").unwrap_err(),
            TodoError::NotFound
        ));
        assert!(store.list().is_empty(), "未知 id 操作不得改变列表");
    }

    /// 场景 12：persist 失败（数据目录被文件占位）→ 内存状态必须回滚，
    /// 不允许出现「命令返回 Err 但内存已变更」的半更新状态（INV-002）。
    #[test]
    fn mutation_rolls_back_when_persist_fails() {
        let dir = TempDir::new("persist-fail");
        let data_dir = dir.join("data");
        let path = data_dir.join("todos.json");
        let mut store = TodoStore::load(&path).expect("空库可加载");
        let item = store.add("任务A".to_string()).expect("正常添加并落盘");

        // 用文件占据原数据目录：后续 persist 的 create_dir_all 会失败
        fs::remove_dir_all(&data_dir).expect("移除数据目录");
        fs::write(&data_dir, "blocked").expect("用文件占位原数据目录");

        // toggle 失败：done 不变
        let err = store.toggle(&item.id).expect_err("persist 应失败");
        assert!(
            matches!(err, TodoError::Io(_)),
            "应为 Io 错误，实际: {err:?}"
        );
        assert_eq!(store.list()[0].done, false, "toggle 失败后 done 应保持不变");

        // remove 失败：条目仍在且顺序不变
        let err = store.remove(&item.id).expect_err("persist 应失败");
        assert!(
            matches!(err, TodoError::Io(_)),
            "应为 Io 错误，实际: {err:?}"
        );
        assert_eq!(store.list().len(), 1, "remove 失败后条目应保留");
        assert_eq!(store.list()[0].id, item.id, "remove 失败后条目 id 不变");

        // update 失败：文本不变
        let err = store
            .update(&item.id, "改名".to_string())
            .expect_err("persist 应失败");
        assert!(
            matches!(err, TodoError::Io(_)),
            "应为 Io 错误，实际: {err:?}"
        );
        assert_eq!(store.list()[0].text, "任务A", "update 失败后文本应保持不变");
    }

    // ===== 多进程协作：reload / 跨进程锁 / 路径解析 =====

    /// 场景：另一实例写入后，reload 能感知磁盘上的变更（MCP 与 App 共享数据文件）。
    #[test]
    fn reload_picks_up_external_writes() {
        let dir = TempDir::new("reload");
        let path = dir.join("todos.json");
        let mut app_store = TodoStore::load(&path).expect("空库可加载");
        app_store.add("应用添加".to_string()).expect("添加成功");

        // 模拟 MCP 进程：同一文件的另一个实例追加写入
        let mut mcp_store = TodoStore::load(&path).expect("重载成功");
        mcp_store.add("AI 添加".to_string()).expect("添加成功");

        assert_eq!(app_store.list().len(), 1, "reload 前内存态不变");
        app_store.reload().expect("reload 成功");
        let items = app_store.list();
        assert_eq!(items.len(), 2, "reload 后应看到对方写入");
        assert_eq!(items[1].text, "AI 添加");
    }

    /// 场景：lock_and_reload 持锁期间完成读-改-写，两个协作实例互不丢失更新。
    #[test]
    fn lock_and_reload_serializes_read_modify_write() {
        let dir = TempDir::new("lock");
        let path = dir.join("todos.json");
        let mut a = TodoStore::load(&path).expect("空库可加载");
        let mut b = TodoStore::load(&path).expect("空库可加载");

        // 同一进程顺序持锁：a 写入后释放，b 持锁 reload 应看到 a 的写入
        {
            let _guard = a.lock_and_reload().expect("a 持锁成功");
            a.add("来自 A".to_string()).expect("a 添加成功");
        }
        {
            let _guard = b.lock_and_reload().expect("b 持锁成功");
            b.add("来自 B".to_string()).expect("b 添加成功");
        }

        let reloaded = TodoStore::load(&path).expect("重载成功");
        let texts: Vec<String> = reloaded.list().iter().map(|i| i.text.clone()).collect();
        assert_eq!(texts, ["来自 A", "来自 B"], "两次写入都不得丢失");

        // 锁占位文件生成在数据文件旁
        assert!(dir.join("todos.json.lock").exists(), "应生成锁占位文件");
    }

    /// 场景：TODOS_DATA_PATH 环境变量覆盖默认路径解析。
    /// 注意：环境变量为进程级状态，本测试是唯一读写该变量的测试。
    #[test]
    fn data_file_path_respects_env_override() {
        let dir = TempDir::new("env-override");
        let custom = dir.join("custom.json");
        std::env::set_var(DATA_PATH_ENV, &custom);
        let resolved = data_file_path("com.example.app").expect("env 覆盖应直接成功");
        std::env::remove_var(DATA_PATH_ENV);
        assert_eq!(resolved, custom, "应原样返回环境变量指定的路径");
    }
}
