// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config_manager;
mod marker_manager;
mod video_processor;
mod settings_manager;
mod trash_manager;
mod auth;
mod workflow_orchestrator; // 🌟 引入中枢大脑
mod widget_manager; // 👈 引入新模块
use marker_manager::EasyCutProject;
use std::path::PathBuf;
use std::process::Command; // 仅保留给探测操作
use tauri::{AppHandle, Manager, State, Emitter}; // 🟢 新增了 Emitter 用于发送事件
use tokio::sync::broadcast;

use settings_manager::{AppSettings, load_settings, save_settings_logic};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
// ==========================================
// 🌟 新增：Phase 1 全场景打轴 Local Hub
// ==========================================
use axum::{
    extract::State as AxumState,
    http::{StatusCode, HeaderMap}, // 🌟 新增 HeaderMap 解析
    routing::post,
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// 外部打轴数据契约 (Payload Protocol)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExternalMarkerPayload {
    pub source: String,              // 来源：例如 "browser_bilibili", "desktop_widget"
    pub video_title: Option<String>, // 视频标题，用于后续的自动匹配
    pub timestamp: f64,              // 精确到毫秒的打轴时间戳
    pub label: Option<String>,       // 用户顺手打的标签（如空则默认为"外部标记"）
}

/// Axum 路由处理器：接收外部 POST 请求并穿透至 Vue 前端
// 🌟 核心重构：注入 Header 解析，实施 Token 拦截
async fn handle_external_marker(
    headers: HeaderMap,
    AxumState(app): AxumState<AppHandle>,
    Json(payload): Json<ExternalMarkerPayload>,
) -> Result<&'static str, (StatusCode, String)> {
    // 1. 尝试从 HTTP 请求头中获取 Authorization
    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let client_token = auth_header.replace("Bearer ", "");

    // 2. 从 Rust 内存中拿出真正的 Token 进行核对
    let guardian = app.state::<auth::SecurityGuardian>();
    let server_token = guardian.session_token.lock().unwrap().clone();

    // 3. 🛡️ 铁穹门禁：空 Token 或 Token 错误，直接拉黑！
    if server_token.is_empty() || client_token != server_token {
        println!("🚨 拦截！恶意脚本尝试通过 HTTP 注入打轴数据！");
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    println!("📥 验证通过，收到外部打轴数据: {:?}", payload);

    if let Err(e) = app.emit("external-marker-received", payload.clone()) {
        eprintln!("⚠️ 向前端推送打轴数据失败: {}", e);
    }

    Ok("OK")
}

/// 启动本地守护 Server (Local Hub)
fn spawn_local_hub(app_handle: AppHandle) {
    // 🌟 核心修复：将 tokio::spawn 替换为 tauri::async_runtime::spawn
    // 这样任务就会被精准投递到 Tauri 官方维护的后台 Tokio 线程池中
    tauri::async_runtime::spawn(async move {
        // 🛡️ 极其关键的 CORS 配置：允许任何浏览器插件跨域 POST
        // 因为已经有了强力的 Bearer Token 拦截，CORS 适当放宽给本地扩展也是安全的
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([axum::http::Method::POST])
            .allow_headers(Any);

        let app = Router::new()
            .route("/api/marker", post(handle_external_marker))
            .layer(cors)
            .with_state(app_handle.clone()); // 克隆给 Axum
        // 🌟 核心防御：绑定 127.0.0.1:0，由操作系统分配随机高位端口
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let assigned_port = listener.local_addr().unwrap().port();
        // 将系统分配的端口写入内核守护者
        let guardian = app_handle.state::<auth::SecurityGuardian>();
        *guardian.hub_port.lock().unwrap() = assigned_port;


        println!("🚀 外部消息枢纽已启动，动态幽灵端口: {}", assigned_port);

        axum::serve(listener, app).await.unwrap();
    });
}

// ==========================================
// 🛡️ 全局任务控制大闸 (The Kill Switch)
// ==========================================

/// 通用包裹器：让任意长时间运行的异步任务具备“可被取消”的能力
async fn run_with_cancellation<T>(
    cancel_tx: State<'_, broadcast::Sender<()>>,
    task_future: impl std::future::Future<Output = Result<T, String>>,
) -> Result<T, String> {
    let mut rx = cancel_tx.subscribe();
    tokio::select! {
        res = task_future => res,
        _ = rx.recv() => Err("🛑 任务已被用户主动取消".to_string()),
    }
}

#[tauri::command]
async fn cancel_active_tasks(cancel_tx: State<'_, broadcast::Sender<()>>) -> Result<(), String> {
    let _ = cancel_tx.send(());
    println!("🛑 接收到前端中止指令，正在清理底层 FFmpeg 进程...");
    Ok(())
}

// ==========================================
// 1. 基础环境探针接口
// ==========================================
// 🌟 升级启动自检指令
/*#[tauri::command]
async fn check_ffmpeg_status() -> Result<String, String> {
    // 获取绝对路径
    let ffmpeg_path = video_processor::get_secure_executable_path("ffmpeg")?;
    let ffprobe_path = video_processor::get_secure_executable_path("ffprobe")?;
    // 🌟 在启动时执行极其严格的物理基因比对
    video_processor::verify_ffmpeg_integrity(&ffmpeg_path).await?;
    // 如果有 FFprobe 的哈希，也可以在这里一起校验
    // 校验通过，测试底层调用是否可用
    let output = tokio::process::Command::new(&ffmpeg_path)
        .arg("-version")
        .output()
        .await
        .map_err(|e| format!("FFmpeg 引擎无法唤醒: {}", e))?;
    /*match output {
        Ok(out) => {
            if out.status.success() {
                let result = String::from_utf8_lossy(&out.stdout);
                let first_line = result.lines().next().unwrap_or("FFmpeg OK");
                Ok(format!("FFmpeg 引擎已就绪: {}", first_line))
            } else {
                Err("找到 FFmpeg，但执行异常".to_string())
            }
        }
        Err(_) => Err("未找到 FFmpeg 引擎，请确认可执行文件位置".to_string()),
    }*/
    if output.status.success() {
        Ok("引擎完整性校验通过，状态正常".to_string())
    } else {
        Err("引擎受损或权限不足".to_string())
    }
}
*/

#[tauri::command]
async fn check_ffmpeg_status() -> Result<String, String> {
    // 🛡️ 调用 video_processor 封装好的全量自检
    video_processor::perform_full_engine_check().await?;

    Ok("核心引擎完整性校验通过，铁穹系统运行正常".to_string())
}
#[tauri::command]
async fn extract_single_segment(
    params: workflow_orchestrator::SingleExtractParams,
    session_token: String, // 👈 必须加
    guardian: tauri::State<'_, auth::SecurityGuardian>, // 👈 必须加
) -> Result<String, String> {
    // 🛡️ 强制关卡
    auth::check_pro_gate(&guardian, &session_token)?;
    workflow_orchestrator::extract_single_segment(params).await
}

// ==========================================
// 2. Tauri API 网关 (Controller)
// ==========================================
/// 🟢 基础免费功能：按时长切割
#[tauri::command]
async fn batch_split_by_duration(
    params: workflow_orchestrator::DurationSplitParams,
    guardian: tauri::State<'_, auth::SecurityGuardian>, // 👈 增加
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_duration_split(params,&guardian)).await
}
/// 👑 旗舰 PRO 功能：按数量均分 (内含商业鉴权)
#[tauri::command]
async fn batch_split_by_count(
    params: workflow_orchestrator::CountSplitParams,
    session_token: String, // 接收前端传来的令牌
    guardian: tauri::State<'_, auth::SecurityGuardian>, // 获取内核守护者状态
    cancel_tx: tauri::State<'_, tokio::sync::broadcast::Sender<()>>
) -> Result<String, String> {
    // 调用重构后的业务流，传递安全上下文
    run_with_cancellation(
        cancel_tx,
        workflow_orchestrator::run_count_split(params, &guardian, &session_token)
    ).await
}
/// 🟢/👑 智能打轴片段导出
#[tauri::command]
async fn execute_marker_split_task(
    params: workflow_orchestrator::MarkerSplitParams,
    session_token: String, // 👈 增加前端传参
    guardian: tauri::State<'_, auth::SecurityGuardian>, // 👈 增加
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<video_processor::ExportResult, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_marker_split(params,&guardian,&session_token)).await
}
/// 👑 旗舰 PRO：执行音频提取
#[tauri::command]
async fn execute_audio_extract(
    params: workflow_orchestrator::AudioExtractParams,
    session_token: String, // 接收前端传来的令牌
    guardian: tauri::State<'_, auth::SecurityGuardian>, // 获取内核守护者状态
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_audio_extract(params,&guardian,&session_token)).await
}
/// 👑 旗舰 PRO：执行格式转换
#[tauri::command]
async fn execute_format_convert(
    params: workflow_orchestrator::FormatConvertParams,
    session_token: String, // 接收前端传来的令牌
    guardian: tauri::State<'_, auth::SecurityGuardian>, // 获取内核守护者状态
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_format_convert(params,&guardian,&session_token)).await
}

// ==========================================
// 3. 项目与工作区管理 API
// ==========================================

#[tauri::command]
fn get_workspace_path() -> String {
    config_manager::get_workspace_dir().to_string_lossy().to_string()
}

#[tauri::command]
async fn save_project(project: EasyCutProject) -> Result<(), String> {
    let storage_dir = config_manager::get_markers_dir();
    marker_manager::save_project_logic(&storage_dir, &project).await
}

#[tauri::command]
async fn load_project(video_path: String) -> Result<EasyCutProject, String> {
    let storage_dir = config_manager::get_markers_dir();
    marker_manager::load_project_logic(&storage_dir, &video_path).await
}

#[tauri::command]
async fn move_marker_file_to_trash(video_path: String) -> Result<(), String> {
    let storage_dir = config_manager::get_markers_dir();
    let fingerprint = marker_manager::calculate_video_fingerprint(&video_path).await?;
    let json_path = marker_manager::get_json_path(&storage_dir, &fingerprint);
    trash_manager::move_to_trash(&json_path).await
}

// ==========================================
// 4. 辅助工具与系统交互 API
// ==========================================

#[tauri::command]
async fn get_video_duration_cmd(video_path: String) -> Result<f64, String> {
    video_processor::get_video_duration(&video_path).await
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    match open::that(&path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("无法打开文件夹: {}", e)),
    }
}

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("无法打开文件: {}", e))
}

#[tauri::command]
fn get_app_settings() -> AppSettings {
    load_settings()
}

#[tauri::command]
fn update_app_settings(settings: AppSettings) -> Result<(), String> {
    save_settings_logic(&settings)
}

// src-tauri/src/main.rs
#[tauri::command]
async fn probe_media_info_cmd(
    video_path: String,
    session_token: String, // 🌟 新增
    guardian: tauri::State<'_, auth::SecurityGuardian> // 🌟 新增
) -> Result<video_processor::MediaInfoDTO, String> {
    // 🛡️ 铁穹门禁：因为轨道检查是 PRO 功能，必须查票！
    auth::check_pro_gate(&guardian, &session_token)?;

    video_processor::probe_media_info(&video_path).await
}
#[tauri::command]
async fn run_extract_all_audio_cmd(
    params: workflow_orchestrator::AutoExtractAudioParams,
    session_token: String, // 👈 必须加
    guardian: tauri::State<'_, auth::SecurityGuardian>, // 👈 必须加
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    // 🛡️ 强制关卡
    auth::check_pro_gate(&guardian, &session_token)?;
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_extract_all_audio(params,&guardian,&session_token)).await
}

#[tauri::command]
async fn run_export_pure_video_cmd(
    params: workflow_orchestrator::PureVideoParams,
    session_token: String, // 👈 必须加
    guardian: tauri::State<'_, auth::SecurityGuardian>, // 👈 必须加
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    // 🛡️ 强制关卡
    auth::check_pro_gate(&guardian, &session_token)?;
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_export_pure_video(params,&guardian,&session_token)).await
}
// 悬浮窗开关也要适配，调用我们之前在 widget_manager 中定义的 safe 版本
#[tauri::command]
async fn toggle_widget_safe(
    app: tauri::AppHandle,
    session_token: String,
    guardian: tauri::State<'_, auth::SecurityGuardian>
) -> Result<String, String> {
    widget_manager::toggle_widget_safe_v(app, guardian, session_token).await
}

// 新增：向前端报告真实的硬件能力，实现 UI 联动
#[tauri::command]
fn get_hardware_capabilities() -> usize {
    num_cpus::get()
}


// ==========================================
// 应用主入口
// ==========================================
fn main() {
    let guardian = auth::SecurityGuardian::new(); // 初始化守护者
    let workspace = config_manager::get_workspace_dir();
    println!("易剪启动成功！当前工作区路径: {}", workspace.display());

    let settings = load_settings();
    match trash_manager::clean_expired_trash(settings.trash_retention_days) {
        Ok(count) if count > 0 => println!("🧹 启动清理：自动移除了 {} 个过期废弃的标记文件。", count),
        Err(e) => eprintln!("⚠️ 回收站清理异常: {}", e),
        _ => {}
    }

    let (cancel_tx, _) = broadcast::channel::<()>(16);
    // 🌟 定义快捷键对象
    let ctrl_shift_m = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM);
    let alt_w = Shortcut::new(Some(Modifiers::ALT), Code::KeyW);
    tauri::Builder::default()
        .manage(guardian) // 注入全局安全状态
        .manage(cancel_tx)
        .manage(widget_manager::WidgetState::new()) // 👈 注入悬浮球状态机
        .plugin(tauri_plugin_dialog::init())
        // 🌟 统一且唯一的快捷键插件注册入口
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts([ctrl_shift_m, alt_w])
                .unwrap()
                .with_handler(move |app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyM) {
                            widget_manager::handle_global_shortcut(app);
                        } else if shortcut.matches(Modifiers::ALT, Code::KeyW) {
                            let handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = widget_manager::toggle_widget(handle).await;
                            });
                        }
                    }
                })
                .build(),
        )
        // 👇 🌟 核心突破：在应用初始化时，启动本地消息枢纽并注入 AppHandle
        .setup(|app| {
            let app_handle = app.handle().clone();
            spawn_local_hub(app_handle);
           /* // 注册全局快捷键 Ctrl+Shift+M
            #[cfg(desktop)]
            {
                let ctrl_shift_m = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM);
                let alt_w = Shortcut::new(Some(Modifiers::ALT), Code::KeyW); // 🌟 新增
                let _ = app.handle().plugin(tauri_plugin_global_shortcut::Builder::new().build());
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts([ctrl_shift_m, alt_w]) // 🌟 注册两者
                        .unwrap()
                        .build()
                ).expect("快捷键注册失败");
            }*/
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_ffmpeg_status,
            batch_split_by_duration,
            batch_split_by_count,
            execute_marker_split_task,
            get_workspace_path,
            save_project,
            load_project,
            move_marker_file_to_trash,
            get_video_duration_cmd,
            open_folder,
            open_file,
            get_app_settings,
            update_app_settings,
            auth::get_machine_code,
            auth::verify_license_cmd,
            extract_single_segment,
            execute_audio_extract,
            execute_format_convert,
            cancel_active_tasks,
            probe_media_info_cmd,
            run_extract_all_audio_cmd,
            run_export_pure_video_cmd,
            toggle_widget_safe,
            auth::init_security_session,           // 🌟 新增
            auth::verify_license_and_activate,      // 🌟 重构
            auth::get_pro_status_safe,
            auth::deactivate_license_safe,
            get_hardware_capabilities,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}