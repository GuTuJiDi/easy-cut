// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 引入刚刚新建的模块
mod config_manager;
mod marker_manager;
mod video_processor;
mod settings_manager; // <--- 引入模块
mod trash_manager;    // <--- 引入模块
use marker_manager::EasyCutProject;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};
use video_processor::{
    execute_ffmpeg_split, get_video_duration, plan_fixed_duration_splits, SplitTask,
}; // <--- 引入 AppHandle
use settings_manager::{AppSettings, load_settings, save_settings_logic};
use video_processor::ExportResult;

// 定义一个暴露给前端的 Tauri Command
#[tauri::command]
async fn check_ffmpeg_status() -> Result<String, String> {
    // 尝试调用本地的 ffmpeg 命令
    let output = Command::new("ffmpeg").arg("-version").output();

    match output {
        Ok(out) => {
            if out.status.success() {
                // 提取第一行版本信息并返回给前端
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
async fn split_video(
    input_path: String,
    output_path: String, // 现在前端传过来的可以是基础名称，如 D:\test
    start_time: String,
    duration: String,
) -> Result<String, String> {
    // 将前端传来的 String 转为 u32，实际开发中这里要加错误处理，暂略
    let start: u32 = start_time.parse().unwrap_or(0);
    let dur: u32 = duration.parse().unwrap_or(10);

    // 组装一个单一任务
    let task = SplitTask {
        start_time: start,
        duration: dur,
        output_path: output_path,
    };

    // 调用解耦后的底层函数
    execute_ffmpeg_split(&input_path, &task)
}
/// 新增：打通闭环的业务接口 —— 按固定时长批量分割视频
#[tauri::command]
async fn batch_split_by_duration(
    input_path: String,
    // base_output_name: String, // 比如 "D:\Desktop\先导片"，代码会自动加上 _part1.mp4
    output_dir: String,    //只接收目录路径
    video_name: String,    //视频基础名称，不带扩展名
    segment_duration: u32, // 每段时长（秒），比如 30
) -> Result<String, String> {
    println!("开始分析视频: {}", input_path);

    // 第一步：获取总时长
    let total_duration = match get_video_duration(&input_path) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };
    println!("视频总时长: {} 秒", total_duration);

    // 2. 动态创建专属文件夹: D:\Desktop\jianji\20231024.先导片
    let target_folder = Path::new(&output_dir).join(&video_name);
    if !target_folder.exists() {
        if let Err(e) = fs::create_dir_all(&target_folder) {
            return Err(format!("创建专属输出目录失败: {}", e));
        }
    }

    // 3. 构建任务并指定最终的输出基础路径
    // base_name 会变成: D:\Desktop\jianji\20231024.先导片\20231024.先导片
    let base_name = target_folder
        .join(&video_name)
        .to_string_lossy()
        .to_string();
    let tasks = plan_fixed_duration_splits(total_duration, segment_duration, &base_name);

    // 4. 循环执行切割
    let mut success_logs = Vec::new();
    for task in tasks {
        match execute_ffmpeg_split(&input_path, &task) {
            Ok(out_path) => success_logs.push(format!("✅ 生成: {}", out_path)),
            Err(e) => return Err(format!("❌ 切割 {} 失败: {}", task.output_path, e)),
        }
    }

    Ok(success_logs.join("\n"))
}
/// 辅助函数：获取本软件专属的标记数据存储目录
fn get_app_marker_dir(app: &AppHandle) -> Result<PathBuf, String> {
    // 获取标准的 App Local Data 目录
    let mut data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "无法获取系统应用数据目录".to_string())?;

    // 追加子目录: /Markers
    data_dir.push("Markers");
    Ok(data_dir)
}

// --- 暴露给前端：获取当前工作区路径 ---
#[tauri::command]
fn get_workspace_path() -> String {
    config_manager::get_workspace_dir()
        .to_string_lossy()
        .to_string()
}

// --- 更新保存标记的接口（直接使用 ConfigManager 分配的目录） ---

// --- 更新：V1.1 信封读写 API ---

#[tauri::command]
async fn save_project(project: EasyCutProject) -> Result<(), String> {
    let storage_dir = config_manager::get_markers_dir();
    marker_manager::save_project_logic(&storage_dir, &project)
}
#[tauri::command]
async fn load_project(video_path: String) -> Result<EasyCutProject, String> {
    let storage_dir = config_manager::get_markers_dir();
    marker_manager::load_project_logic(&storage_dir, &video_path)
}
// --- 更新加载标记的接口 ---

// --- 新增 1：获取视频总时长给前端展示 ---
#[tauri::command]
async fn get_video_duration_cmd(video_path: String) -> Result<u32, String> {
    // 复用我们之前写在 video_processor 里的探针函数
    video_processor::get_video_duration(&video_path)
}

// --- 新增 2：打开操作系统的目标文件夹 ---
#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    match open::that(&path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("无法打开文件夹: {}", e)),
    }
}

// 新增：使用系统默认程序打开单体文件（如直接播放视频）
#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
// --- 新增：读取设置 API ---
#[tauri::command]
fn get_app_settings() -> AppSettings {
    load_settings()
}

// --- 新增：保存设置 API ---
#[tauri::command]
fn update_app_settings(settings: AppSettings) -> Result<(), String> {
    save_settings_logic(&settings)
}


// --- 修复：增加 async 关键字，移出主线程 ---
#[tauri::command]
async fn move_marker_file_to_trash(video_path: String) -> Result<(), String> {
    let storage_dir = config_manager::get_markers_dir();
    let fingerprint = marker_manager::calculate_video_fingerprint(&video_path)?;
    let json_path = marker_manager::get_json_path(&storage_dir, &fingerprint);
    trash_manager::move_to_trash(&json_path)
}

// --- 修复之前 FFmpeg API 调用的编译报错 ---
#[tauri::command]
async fn execute_marker_split_task(video_path: String, output_dir: String) -> Result<video_processor::ExportResult, String> {
    let storage_dir = config_manager::get_markers_dir();

    // 现在加载的是 Project，我们要提取其中的 markers 传给 FFmpeg
    let project = marker_manager::load_project_logic(&storage_dir, &video_path)?;
    if project.markers.is_empty() {
        return Err("⚠️ 没有任何标记片段！".to_string());
    }

    tokio::task::spawn_blocking(move || {
        tauri::async_runtime::block_on(async {
            video_processor::split_video_by_markers(&video_path, &output_dir, project.markers).await
        })
    })
        .await
        .map_err(|e| format!("线程池执行异常: {}", e))?
}
// ... 记得在 main() 的 invoke_handler 里加上 get_video_duration_cmd 和 open_folder
fn main() {
    // 软件启动时，立刻初始化一次工作区目录，确保文件夹被创建
    let workspace = config_manager::get_workspace_dir();
    println!("易剪启动成功！当前工作区路径: {}", workspace.display());
    // 2. 读取配置，并触发后台回收站清理守护任务
    let settings = load_settings();
    match trash_manager::clean_expired_trash(settings.trash_retention_days) {
        Ok(count) if count > 0 => println!("🧹 启动清理：自动移除了 {} 个过期废弃的标记文件。", count),
        Err(e) => eprintln!("⚠️ 回收站清理异常: {}", e),
        _ => {} // count == 0 或者不清理，静默通过
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // 记得把新接口注册进来！
        .invoke_handler(tauri::generate_handler![
            check_ffmpeg_status,
            split_video,
            batch_split_by_duration,
            load_project,
            save_project,
            get_workspace_path,
            get_video_duration_cmd,
            open_folder,
            get_app_settings,
            update_app_settings,
            move_marker_file_to_trash,
            execute_marker_split_task,
            open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
