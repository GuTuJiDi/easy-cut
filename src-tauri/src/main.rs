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
async fn handle_external_marker(
    AxumState(app): AxumState<AppHandle>, // 注入 Tauri 的大管家
    Json(payload): Json<ExternalMarkerPayload>,
) -> &'static str {
    println!("📥 收到外部打轴数据: {:?}", payload);

    // 核心：通过 Tauri v2 的 Emitter 接口，将数据穿透发送给 Vue 前端
    if let Err(e) = app.emit("external-marker-received", payload.clone()) {
        eprintln!("⚠️ 向前端推送打轴数据失败: {}", e);
    }

    // TODO: 未来可在此处将 payload 写入本地 JSON 文件，以便主界面未开启时暂存数据

    "OK"
}

/// 启动本地守护 Server (Local Hub)
fn spawn_local_hub(app_handle: AppHandle) {
    // 🌟 核心修复：将 tokio::spawn 替换为 tauri::async_runtime::spawn
    // 这样任务就会被精准投递到 Tauri 官方维护的后台 Tokio 线程池中
    tauri::async_runtime::spawn(async move {
        // 🛡️ 极其关键的 CORS 配置：允许任何浏览器插件跨域 POST
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/api/marker", post(handle_external_marker))
            .layer(cors)
            .with_state(app_handle); // 把 Tauri 实例安全地传给 Axum

        let addr = SocketAddr::from(([127, 0, 0, 1], 13456));
        println!("🚀 外部消息枢纽已启动，全场景监听端口: {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
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

#[tauri::command]
async fn check_ffmpeg_status() -> Result<String, String> {
    let output = Command::new("ffmpeg").arg("-version").output();
    match output {
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
    }
}

#[tauri::command]
async fn extract_single_segment(params: workflow_orchestrator::SingleExtractParams) -> Result<String, String> {
    workflow_orchestrator::extract_single_segment(params).await
}

// ==========================================
// 2. Tauri API 网关 (Controller)
// ==========================================
/// 🟢 基础免费功能：按时长切割
#[tauri::command]
async fn batch_split_by_duration(
    params: workflow_orchestrator::DurationSplitParams,
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_duration_split(params)).await
}
/// 👑 旗舰 PRO 功能：按数量均分 (内含商业鉴权)
#[tauri::command]
async fn batch_split_by_count(
    params: workflow_orchestrator::CountSplitParams,
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_count_split(params)).await
}
/// 🟢/👑 智能打轴片段导出
#[tauri::command]
async fn execute_marker_split_task(
    params: workflow_orchestrator::MarkerSplitParams,
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<video_processor::ExportResult, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_marker_split(params)).await
}
/// 👑 旗舰 PRO：执行音频提取
#[tauri::command]
async fn execute_audio_extract(
    params: workflow_orchestrator::AudioExtractParams,
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_audio_extract(params)).await
}
/// 👑 旗舰 PRO：执行格式转换
#[tauri::command]
async fn execute_format_convert(
    params: workflow_orchestrator::FormatConvertParams,
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_format_convert(params)).await
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
    #[cfg(target_os = "windows")]
    { std::process::Command::new("cmd").args(["/C", "start", "", &path]).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "linux")]
    { std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?; }
    Ok(())
}

#[tauri::command]
fn get_app_settings() -> AppSettings {
    load_settings()
}

#[tauri::command]
fn update_app_settings(settings: AppSettings) -> Result<(), String> {
    save_settings_logic(&settings)
}

#[tauri::command]
async fn probe_media_info_cmd(video_path: String) -> Result<video_processor::MediaInfoDTO, String> {
    video_processor::probe_media_info(&video_path).await
}

#[tauri::command]
async fn run_extract_all_audio_cmd(
    params: workflow_orchestrator::AutoExtractAudioParams,
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_extract_all_audio(params)).await
}

#[tauri::command]
async fn run_export_pure_video_cmd(
    params: workflow_orchestrator::PureVideoParams,
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_export_pure_video(params)).await
}

// ==========================================
// 应用主入口
// ==========================================
fn main() {
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
            widget_manager::toggle_widget,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}