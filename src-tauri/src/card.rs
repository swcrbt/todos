// 卡片窗口模块：无边框 320×440 小窗口，初始隐藏；
// 托盘左键切换显隐；显示前按鼠标所在屏幕定位；失焦防抖复核后隐藏；
// 关闭请求（如系统级关闭）转为隐藏而非销毁，保持 WebView 实例复用。

use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{
    Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

use crate::tray::LAST_TRAY_CLICK_MS;

/// 卡片窗口 id（进程内全局唯一，不重建实例）。
pub const CARD_WINDOW_ID: &str = "card";
/// 失焦隐藏前的防抖复核等待时长（毫秒）。
const BLUR_DEBOUNCE_MS: u64 = 120;
/// 托盘点击守卫窗口期（毫秒）：该窗口期内的失焦事件不触发隐藏。
const TRAY_CLICK_GUARD_MS: u64 = 120;

/// 当前 unix 毫秒。
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 创建卡片窗口（初始隐藏）。窗口事件：
/// - CloseRequested：拦截阻止关闭，转为隐藏（隐藏语义而非退出）
/// - Focused(false)：防抖延时后复核焦点状态与托盘点击窗口期，确认后隐藏并丢弃未提交输入
pub fn create_card(app: &tauri::AppHandle) -> tauri::Result<WebviewWindow> {
    let win = WebviewWindowBuilder::new(app, CARD_WINDOW_ID, WebviewUrl::App("index.html".into()))
        .title("待办")
        .decorations(false)
        .resizable(false)
        .inner_size(320.0, 440.0)
        .visible(false)
        .build()?;

    let handle_for_event = app.clone();
    win.on_window_event(move |event| match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = win.hide();
            // 通知前端：隐藏时丢弃未提交输入并清除错误横幅
            let _ = handle_for_event.emit("card-hidden", ());
        }
        WindowEvent::Focused(false) => {
            // 失焦不立即隐藏：等待防抖窗口期，复核窗口是否重新聚焦（用户可能
            // 只是短暂切换焦点、或点击的是托盘图标——托盘点击先记录时间戳）。
            // 复核通过则隐藏并提示前端清理卡片状态。
            let handle = handle_for_event.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(BLUR_DEBOUNCE_MS));
                let still_blurred = handle
                    .get_webview_window(CARD_WINDOW_ID)
                    .map(|w| !w.is_focused().unwrap_or(false))
                    .unwrap_or(false);
                let within_tray_guard = now_ms()
                    .saturating_sub(LAST_TRAY_CLICK_MS.load(Ordering::Relaxed))
                    < TRAY_CLICK_GUARD_MS;
                if still_blurred && !within_tray_guard {
                    if let Some(win) = handle.get_webview_window(CARD_WINDOW_ID) {
                        let _ = win.hide();
                        let _ = handle.emit("card-hidden", ());
                    }
                }
            });
        }
        _ => {}
    });
    Ok(win)
}

/// 切换卡片显隐：可见 → 隐藏；隐藏 → 定位后显示并聚焦（SCN-TRAY-002-A/B）。
pub fn toggle_card(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window(CARD_WINDOW_ID) {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
            let _ = app.emit("card-hidden", ());
        } else {
            position_card(&win, app);
            let _ = win.show();
            let _ = win.set_focus();
            let _ = app.emit("card-shown", ());
        }
    }
}

/// 显示前定位：取鼠标所在显示器，macOS 置于工作区顶部下方、Windows 置于任务栏上方，
/// 水平对齐鼠标 x 并夹紧于工作区内（按屏幕缩放系数换算物理像素）。
fn position_card(win: &WebviewWindow, app: &tauri::AppHandle) {
    let Ok(cursor) = app.cursor_position() else {
        return;
    };
    let monitor = app
        .monitor_from_point(cursor.x, cursor.y)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(mon) = monitor else {
        return;
    };
    let scale = mon.scale_factor();
    let mpos = mon.position();
    let msize = mon.size();
    let w = (320.0 * scale) as i32;
    let h = (440.0 * scale) as i32;

    // macOS 菜单栏在下，卡片贴近工作区顶部；Windows 任务栏在下，卡片贴近工作区底部
    #[cfg(target_os = "macos")]
    let y = mpos.y + (8.0 * scale) as i32;
    #[cfg(not(target_os = "macos"))]
    let y = mpos.y + msize.height as i32 - h - (8.0 * scale) as i32;

    let min_x = mpos.x + (4.0 * scale) as i32;
    let max_x = mpos.x + msize.width as i32 - w - (4.0 * scale) as i32;
    let x = if max_x < min_x {
        min_x
    } else {
        (cursor.x as i32 - w / 2).clamp(min_x, max_x)
    };
    let _ = win.set_position(PhysicalPosition::new(x, y));
}
