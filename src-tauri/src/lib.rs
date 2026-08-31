// 应用装配入口：单实例插件、数据加载、托盘、卡片窗口、命令注册与 macOS 激活策略。
// 依赖方向：本模块聚合数据层（todo-store）、命令层、托盘与卡片窗口，不承载业务规则。

mod card;
mod commands;
mod tray;

use std::sync::Mutex;

use tauri::Manager;
use todo_store::TodoStore;

pub fn run() {
    tauri::Builder::default()
        // 单实例：重复启动只保留一个托盘图标与一份数据；二次启动唤起既有进程显示卡片
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            card::show_card(app);
        }))
        .setup(|app| {
            // macOS：仅菜单栏常驻，无 Dock 图标（Accessory 激活策略）
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // 数据路径：平台应用数据目录/todos.json；文件缺失→空库，损坏→备份并空库
            let data_dir = app.path().app_data_dir()?;
            let store = TodoStore::load(data_dir.join("todos.json"))?;
            // Tauri State 为共享引用，可变数据层需经 Mutex 单写者访问（INV-001）
            app.manage(Mutex::new(store));

            tray::create_tray(app.handle())?;
            card::create_card(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_todos,
            commands::add_todo,
            commands::toggle_todo,
            commands::update_todo,
            commands::remove_todo
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
