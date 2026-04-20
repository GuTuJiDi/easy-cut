// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config_manager;
mod marker_manager;
mod video_processor;
mod settings_manager;
mod trash_manager;
mod auth;
mod workflow_orchestrator; // 🌟 引入中枢大脑

use marker_manager::EasyCutProject;
use std::path::PathBuf;
use std::process::Command; // 仅保留给探测操作
use tauri::{AppHandle, Manager, State}; // 🟢 新增了 State 用于依赖注入
use tokio::sync::broadcast;             // 🟢 新增：广播频道，用于发送取消信号
use settings_manager::{AppSettings, load_settings, save_settings_logic};

// ==========================================
// 🛡️ 新增：全局任务控制大闸 (The Kill Switch)
// ==========================================

/// 通用包裹器：让任意长时间运行的异步任务具备“可被取消”的能力
async fn run_with_cancellation<T>(
    cancel_tx: State<'_, broadcast::Sender<()>>,
    task_future: impl std::future::Future<Output = Result<T, String>>,
) -> Result<T, String> {
    let mut rx = cancel_tx.subscribe();
    tokio::select! {
        // 正常执行路径：如果 task_future 先完成，返回其结果
        res = task_future => res,
        // 取消信号路径：如果先收到前端的取消广播，强制中断并报错
        _ = rx.recv() => Err("🛑 任务已被用户主动取消".to_string()),
    }
}

/// 供前端调用的取消指令
#[tauri::command]
async fn cancel_active_tasks(cancel_tx: State<'_, broadcast::Sender<()>>) -> Result<(), String> {
    let _ = cancel_tx.send(()); // 发送核弹信号，所有监听此频道的任务都会被 Drop
    println!("🛑 接收到前端中止指令，正在清理底层 FFmpeg 进程...");
    Ok(())
}

// ==========================================
// 1. 基础环境探针接口 (保持原样，毫秒级任务无需取消机制)
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
// 2. Tauri API 网关 (Controller) - 🟢 已接入安全大闸
// ==========================================

/// 🟢 基础免费功能：按时长切割
#[tauri::command]
async fn batch_split_by_duration(
    params: workflow_orchestrator::DurationSplitParams,
    cancel_tx: State<'_, broadcast::Sender<()>> // 注入取消信号发射器
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
// 3. 项目与工作区管理 API (保持原样)
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
// 4. 辅助工具与系统交互 API (保持原样)
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
// 🟢 新增：媒体智能探针接口
#[tauri::command]
async fn probe_media_info_cmd(video_path: String) -> Result<video_processor::MediaInfoDTO, String> {
    // 毫秒级探测任务，不需要接入 cancel_tx 中断大闸
    video_processor::probe_media_info(&video_path).await
}

// 🌟 修复：补充自动化脚本 1 的 API 包裹器 (带取消大闸)
#[tauri::command]
async fn run_extract_all_audio_cmd(
    params: workflow_orchestrator::AutoExtractAudioParams,
    cancel_tx: State<'_, broadcast::Sender<()>>
) -> Result<String, String> {
    run_with_cancellation(cancel_tx, workflow_orchestrator::run_extract_all_audio(params)).await
}

// 🌟 修复：补充自动化脚本 2 的 API 包裹器 (带取消大闸)
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

    // 🟢 初始化全局广播频道 (容量 16 足够防止消息堆积)
    let (cancel_tx, _) = broadcast::channel::<()>(16);

    tauri::Builder::default()
        .manage(cancel_tx) // 🌟 核心：将大闸的开关注入到 Tauri 全局状态中
        .plugin(tauri_plugin_dialog::init())
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
            cancel_active_tasks, // 🟢 注册新增的取消接口
            probe_media_info_cmd, // 👈 在这里注册
            run_extract_all_audio_cmd,
            run_export_pure_video_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}