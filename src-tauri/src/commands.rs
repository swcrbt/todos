// Tauri command 层：薄包装 todo-store 数据层，参数与错误到中文可读字符串的映射。
// 本层只做转发与错误翻译，不持有持久化实现细节。

use std::sync::Mutex;

use tauri::State;
use todo_store::{TodoError, TodoItem, TodoStore};

/// 将数据层错误映射为前端可展示的中文错误信息。
fn error_message(err: TodoError) -> String {
    match err {
        TodoError::InvalidInput => "内容为空，无法添加".to_string(),
        TodoError::NotFound => "待办不存在".to_string(),
        TodoError::Io(_) => "磁盘读写失败".to_string(),
        TodoError::CorruptData => "数据文件损坏".to_string(),
        TodoError::Serde(_) => "数据序列化失败".to_string(),
    }
}

/// 锁定共享存储；锁中毒视为内部状态异常（正常路径不会发生）。
fn lock_store<'a>(
    state: &'a State<'_, Mutex<TodoStore>>,
) -> Result<std::sync::MutexGuard<'a, TodoStore>, String> {
    state.lock().map_err(|_| "内部状态锁异常".to_string())
}

/// 列出全部待办（追加顺序）。持锁 reload 以感知 MCP 进程等外部写入。
#[tauri::command]
pub fn list_todos(state: State<'_, Mutex<TodoStore>>) -> Result<Vec<TodoItem>, String> {
    let mut store = lock_store(&state)?;
    let _guard = store.lock_and_reload().map_err(error_message)?;
    Ok(store.list())
}

/// 添加待办：非空文本追加队尾并立即落盘；空文本由数据层拒绝。
/// 持锁 reload 后写入，避免覆盖 MCP 进程的并发变更。
#[tauri::command]
pub fn add_todo(state: State<'_, Mutex<TodoStore>>, text: String) -> Result<TodoItem, String> {
    let mut store = lock_store(&state)?;
    let _guard = store.lock_and_reload().map_err(error_message)?;
    store.add(text).map_err(error_message)
}

/// 切换完成状态：返回切换后的条目；id 不存在时返回中文提示。
#[tauri::command]
pub fn toggle_todo(state: State<'_, Mutex<TodoStore>>, id: String) -> Result<TodoItem, String> {
    let mut store = lock_store(&state)?;
    let _guard = store.lock_and_reload().map_err(error_message)?;
    store.toggle(&id).map_err(error_message)
}

/// 修改待办文本：trim 后非空由数据层校验；返回修改后的条目。
#[tauri::command]
pub fn update_todo(
    state: State<'_, Mutex<TodoStore>>,
    id: String,
    text: String,
) -> Result<TodoItem, String> {
    let mut store = lock_store(&state)?;
    let _guard = store.lock_and_reload().map_err(error_message)?;
    store.update(&id, text).map_err(error_message)
}

/// 删除待办：直接删除并落盘，无确认、无撤回。
#[tauri::command]
pub fn remove_todo(state: State<'_, Mutex<TodoStore>>, id: String) -> Result<(), String> {
    let mut store = lock_store(&state)?;
    let _guard = store.lock_and_reload().map_err(error_message)?;
    store.remove(&id).map_err(error_message)
}
