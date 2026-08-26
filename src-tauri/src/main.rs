// 应用入口：调用装配函数启动（桌面平台不弹出控制台窗口）。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tray_todo_lib::run();
}
