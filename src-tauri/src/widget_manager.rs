use tauri::{State,AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, Emitter};
use tokio::sync::Mutex;
use std::time::Instant;
use crate::{auth, ExternalMarkerPayload}; // 复用我们在 Phase 1 定义的数据契约
// 🌟 双模式状态机：记录秒表启动的时间点
pub struct WidgetState {
    pub stopwatch_start: Mutex<Option<Instant>>,
}

impl WidgetState {
    pub fn new() -> Self {
        Self {
            stopwatch_start: Mutex::new(None),
        }
    }
}
/// 创建或销毁透明悬浮球窗口
#[tauri::command]
pub async fn toggle_widget(app: AppHandle) -> Result<String, String> {
    let result = if let Some(window) = app.get_webview_window("widget") {
        let is_visible = window.is_visible().unwrap_or(false);

        if is_visible {
            // 🛑 关闭逻辑：隐藏窗口 + 重置后端计时起点
            window.hide().map_err(|e| e.to_string())?;
            let state = app.state::<WidgetState>();
            *state.stopwatch_start.lock().await = None;

            // 🌟 核心修复：发送“会话停止”信号，强制前端清零并停止 setInterval
            let _ = app.emit("widget-session-stop", ());
            "closed".to_string()
        } else {
            // 🚀 开启逻辑：显示窗口 + 刷新后端计时起点
            window.show().map_err(|e| e.to_string())?;
            let _ = window.set_focus(); // 唤醒焦点，确保快捷键响应

            let state = app.state::<WidgetState>();
            *state.stopwatch_start.lock().await = Some(Instant::now());

            // 🌟 核心修复：发送“会话开始”信号，通知前端重置计时为 00:00:00 并启动
            let _ = app.emit("widget-session-start", ());
            "opened".to_string()
        }
    } else {
        // 🌟 性能优化：适配 Vue Hash 模式，指定 index.html 路径
        let _widget = WebviewWindowBuilder::new(&app, "widget", WebviewUrl::App("index.html#/widget".into()))
            .title("Easy-Cut Widget")
            .inner_size(240.0, 70.0)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(true) // 放开限制，允许 Vue 前端动态调整窗口大小实现“隐藏动画”
            .maximizable(false)
            .build()
            .map_err(|e| format!("悬浮球创建失败: {}", e))?;

        let state = app.state::<WidgetState>();
        *state.stopwatch_start.lock().await = Some(Instant::now());

        let _ = app.emit("widget-session-start", ());
        "opened".to_string()
    };

    // 🌟 状态广播：同步主界面按钮的绿色亮灯状态
    let _ = app.emit("widget-state-changed", result.clone());

    Ok(result)
}

/// 全局快捷键触发的核心业务逻辑
pub fn handle_global_shortcut(app: &AppHandle) {
    let guardian = app.state::<auth::SecurityGuardian>();
    let is_pro = guardian.is_pro.lock().unwrap();

    if !*is_pro {
        println!("⚠️ 快捷键拦截：未激活 PRO 状态，忽略指令。");
        return;
    }
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        let state = app_clone.state::<WidgetState>();
        let start_time_opt = state.stopwatch_start.lock().await;

        if let Some(start_time) = *start_time_opt {
            // 模式 A：秒表模式（计算相对时间）
            let elapsed_sec = start_time.elapsed().as_secs_f64();

            let payload = ExternalMarkerPayload {
                source: "desktop_widget".to_string(),
                video_title: Some("悬浮球盲打记录".to_string()),
                timestamp: elapsed_sec,
                label: Some("盲打标记".to_string()),
            };

            // 🌟 核心修复：不再针对特定窗口发送，而是使用无差别的全局广播 emit
            // 1. 发送给悬浮球，触发其 UI 的 "+1" 动画
            let _ = app_clone.emit("widget-marker-added", payload.clone());

            // 2. 发送给主窗口，进入打轴数据池（收件箱）
            let _ = app_clone.emit("external-marker-received", payload);

            println!("⏱️ 盲打轴已记录: {:.2}s", elapsed_sec);
        } else {
            println!("⚠️ 悬浮球处于隐藏/关闭状态，快捷键请求已忽略");
        }
    });
}

#[tauri::command]
pub async fn toggle_widget_safe_v(
    app: AppHandle,
    guardian: State<'_, auth::SecurityGuardian>,
    session_token: String
) -> Result<String, String> {
    // 🌟 零信任门禁：必须令牌匹配且为 PRO 状态
    auth::check_pro_gate(&guardian, &session_token)?;

    // 逻辑通过后，才执行原有的 toggle 逻辑
    toggle_widget(app).await
}