// todos-mcp：todos 待办的 MCP（Model Context Protocol）服务器。
// 传输方式为 stdio（换行分隔 JSON-RPC 2.0），不监听任何端口；
// AI 客户端（Claude Desktop / Cursor 等支持 MCP 的工具）以子进程方式启动本程序，
// 通过标准输入输出操作与托盘应用共享的同一份 todos.json。
//
// 协议约束：stdout 只允许输出协议消息；任何诊断/日志一律走 stderr。

use std::io::{BufRead, Write};

use serde_json::{json, Value};
use todo_store::{data_file_path, TodoError, TodoStore};

/// 应用标识：必须与 src-tauri/tauri.conf.json 的 identifier 保持一致，
/// 否则 MCP 进程会解析到不同的数据目录。
const APP_IDENTIFIER: &str = "com.swcrbt.todos";

fn main() {
    let data_path = match data_file_path(APP_IDENTIFIER) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("todos-mcp: 数据路径解析失败：{err}");
            std::process::exit(1);
        }
    };
    eprintln!("todos-mcp: 数据文件 {}", data_path.display());

    let mut store = match TodoStore::load(&data_path) {
        Ok(store) => store,
        Err(err) => {
            eprintln!("todos-mcp: 数据加载失败：{err}");
            std::process::exit(1);
        }
    };

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(trimmed) {
            Ok(message) => handle_message(&mut store, &message),
            Err(err) => {
                eprintln!("todos-mcp: 无法解析的消息：{err}");
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": "Parse error" }
                }))
            }
        };
        if let Some(response) = response {
            // serde_json::to_string 不产出内嵌换行，满足 stdio 传输的换行分隔要求
            if let Ok(text) = serde_json::to_string(&response) {
                let _ = writeln!(stdout, "{text}");
                let _ = stdout.flush();
            }
        }
    }
}

/// 处理单条 JSON-RPC 消息；通知（无 id）返回 None（不回复）。
fn handle_message(store: &mut TodoStore, message: &Value) -> Option<Value> {
    let id = message.get("id").cloned();
    let method = message.get("method").and_then(Value::as_str).unwrap_or("");

    // 通知（initialized / cancelled 等）无需回复
    if id.is_none() {
        return None;
    }
    let id = id.unwrap_or(Value::Null);

    let result = match method {
        "initialize" => Ok(initialize_result(message)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools_list_result()),
        "tools/call" => call_tool(store, message.get("params")),
        _ => Err((-32601, format!("Method not found: {method}"))),
    };

    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, msg)) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": msg }
        }),
    })
}

/// initialize 响应：协议版本回显客户端请求值（缺失时回退到已支持的版本）。
fn initialize_result(message: &Value) -> Value {
    let requested = message
        .get("params")
        .and_then(|p| p.get("protocolVersion"))
        .and_then(Value::as_str)
        .unwrap_or("2024-11-05");
    json!({
        "protocolVersion": requested,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "todos-mcp",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": "操作系统托盘待办应用的待办列表，数据与桌面应用实时共享。"
    })
}

/// tools/list 响应：五个工具覆盖待办的完整增删改查。
fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "list_todos",
                "description": "列出全部待办事项（按添加顺序），返回 id/text/done/created_at 字段的 JSON 数组。",
                "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
            },
            {
                "name": "add_todo",
                "description": "新增一条待办事项，返回创建后的条目（含 id）。文本首尾空白会被去除，纯空白文本会被拒绝。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "待办内容（非空）" }
                    },
                    "required": ["text"],
                    "additionalProperties": false
                }
            },
            {
                "name": "toggle_todo",
                "description": "切换指定待办的完成状态（未完成 ↔ 已完成），返回切换后的条目。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "待办条目 id（由 list_todos/add_todo 返回）" }
                    },
                    "required": ["id"],
                    "additionalProperties": false
                }
            },
            {
                "name": "update_todo",
                "description": "修改指定待办的文本内容，返回修改后的条目。纯空白文本会被拒绝。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "待办条目 id" },
                        "text": { "type": "string", "description": "新的待办内容（非空）" }
                    },
                    "required": ["id", "text"],
                    "additionalProperties": false
                }
            },
            {
                "name": "remove_todo",
                "description": "删除指定待办（不可撤回，完成与未完成条目均可删除）。",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "待办条目 id" }
                    },
                    "required": ["id"],
                    "additionalProperties": false
                }
            }
        ]
    })
}

/// tools/call：校验参数后持锁执行（lock_and_reload 保证与桌面应用互不丢数据）。
fn call_tool(store: &mut TodoStore, params: Option<&Value>) -> Result<Value, (i64, String)> {
    let params = params.ok_or((-32602, "缺少 params".to_string()))?;
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or((-32602, "缺少工具名称 params.name".to_string()))?;
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    let get_string = |key: &str| -> Result<String, (i64, String)> {
        args.get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or((-32602, format!("参数 {key} 缺失或不是字符串")))
    };

    // 先持锁并 reload，再执行读-改-写；锁守卫在函数返回时释放
    let _guard = store
        .lock_and_reload()
        .map_err(|err| (-32603, format!("数据文件锁定/读取失败：{err}")))?;

    let outcome = match name {
        "list_todos" => Ok(serde_json::to_string_pretty(&store.list()).unwrap_or_default()),
        "add_todo" => {
            let text = get_string("text")?;
            store
                .add(text)
                .map(|item| serde_json::to_string_pretty(&item).unwrap_or_default())
                .map_err(tool_error)
            }
        "toggle_todo" => {
            let id = get_string("id")?;
            store
                .toggle(&id)
                .map(|item| serde_json::to_string_pretty(&item).unwrap_or_default())
                .map_err(tool_error)
        }
        "update_todo" => {
            let id = get_string("id")?;
            let text = get_string("text")?;
            store
                .update(&id, text)
                .map(|item| serde_json::to_string_pretty(&item).unwrap_or_default())
                .map_err(tool_error)
        }
        "remove_todo" => {
            let id = get_string("id")?;
            store
                .remove(&id)
                .map(|()| "已删除".to_string())
                .map_err(tool_error)
        }
        _ => Err((-32602, format!("未知工具：{name}"))),
    };

    match outcome {
        Ok(text) => Ok(json!({
            "content": [{ "type": "text", "text": text }]
        })),
        Err((code, msg)) if code == -32603 => Ok(json!({
            // 业务/数据层错误是工具执行结果而非协议错误，以 isError 内容返回
            "content": [{ "type": "text", "text": msg }],
            "isError": true
        })),
        Err(err) => Err(err),
    }
}

/// 数据层错误 → 工具执行错误（-32603 内部码，由上层转为 isError 内容返回）。
fn tool_error(err: TodoError) -> (i64, String) {
    (-32603, err.to_string())
}
