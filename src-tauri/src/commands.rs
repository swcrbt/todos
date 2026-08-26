// Tauri command 层：薄包装 todo-store 数据层，参数与错误到中文可读字符串的映射。
// 本层只做转发与错误翻译，不持有持久化实现细节。

use tauri::State;
use todo_store::{TodoError, TodoItem, TodoStore};

/// 将数据层错误映射为前端可展示的中文错误信息。
fn error_message(err: TodoError) -> String {
    match err {
        TodoError::InvalidInput => "内容为空，无法添加".to_string(),
        TodoError::NotFound => "待办不存在".to_string(),
        TodoError::Io(_) => "保存失败：写入磁盘出错".to_string(),
        TodoError::CorruptData => "数据文件损坏".to_string(),
        TodoError::Serde(_) => "数据序列化失败".to_string(),
    }
}

/// 列出全部待办（追加顺序）。
#[tauri::command]
pub fn list_todos(state: State<'_, TodoStore>) -> Vec<TodoItem> {
    state.list()
}

/// 添加待办：非空文本追加队尾并立即落盘；空文本由数据层拒绝。
#[tauri::command]
pub fn add_todo(state: State<'_, TodoStore>, text: String) -> Result<TodoItem, String> {
    state.add(text).map_err(error_message)
}

/// 切换完成状态：返回切换后的条目供前端渲染；id 不存在时返回英文错误映射前的中文提示。
#[tauri::command]
pub fn toggle_todo(state: State<'_, TodoStore>, id: String) -> Result<TodoItem, String> {
    state.toggle(&id).map_err(error_message)
}

/// 删除待办：直接删除并落盘，无确认、无撤回。
#[tauri::command]
pub fn remove_todo(state: State<'_, TodoStore>, id: String) -> Result<(), String> {
    state.remove(&id).map_err(error_message)
}
