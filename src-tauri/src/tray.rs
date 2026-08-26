// 托盘图标模块：菜单栏/托盘常驻图标，左键切换卡片显隐，右键原生菜单【退出】。
// 退出路径：仅托盘右键菜单触发进程退出（应用正常退出通道）。

use std::sync::atomic::AtomicU64;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::card;

/// 最近一次托盘左键点击时间（unix 毫秒）；供窗口失焦守卫判断托盘点击窗口期。
///
/// 竞争背景：卡片可见时点击托盘图标，系统可能先派发窗口失焦事件、再派发托盘点击事件；
/// 若失焦直接隐藏，会出现「想关闭却关掉后又开」或「想打开却被隐藏」的现象。
/// 因此托盘点击先记录时间戳，失焦处理器在窗口期内不执行隐藏。
pub static LAST_TRAY_CLICK_MS: AtomicU64 = AtomicU64::new(0);

/// 当前 unix 毫秒。
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 创建并常驻托盘图标：tooltip「待办」；左键切换卡片；右键菜单含【退出】。
pub fn create_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().expect("默认窗口图标应已配置").clone())
        .tooltip("待办")
        .menu(&menu)
        // macOS 上左键点击不弹出菜单，而是切换卡片；右键才弹出【退出】菜单
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                LAST_TRAY_CLICK_MS.store(now_ms(), std::sync::atomic::Ordering::Relaxed);
                card::toggle_card(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
