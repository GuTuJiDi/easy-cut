// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 引入刚刚新建的模块
mod video_processor;
mod marker_manager;
mod config_manager;



use std::process::Command;
use video_processor::{
    execute_ffmpeg_split, get_video_duration, plan_fixed_duration_splits, SplitTask,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use marker_manager::{Marker, save_markers_logic, load_markers_logic};
use std::path::PathBuf;
use tauri::{AppHandle, Manager}; // <--- 引入 AppHandle




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
    output_dir: String,  //只接收目录路径
    video_name: String, //视频基础名称，不带扩展名
    segment_duration: u32,    // 每段时长（秒），比如 30
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
    let base_name = target_folder.join(&video_name).to_string_lossy().to_string();
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
    let mut data_dir = app.path().app_local_data_dir()
        .map_err(|_| "无法获取系统应用数据目录".to_string())?;

    // 追加子目录: /Markers
    data_dir.push("Markers");
    Ok(data_dir)
}


// --- 暴露给前端：获取当前工作区路径 ---
#[tauri::command]
fn get_workspace_path() -> String {
    config_manager::get_workspace_dir().to_string_lossy().to_string()
}

// --- 更新保存标记的接口（直接使用 ConfigManager 分配的目录） ---
#[tauri::command]
async fn save_markers(video_path: String, markers: Vec<Marker>) -> Result<String, String> {
    let storage_dir = config_manager::get_markers_dir();
    save_markers_logic(&storage_dir, &video_path, markers)
}

// --- 更新加载标记的接口 ---
#[tauri::command]
async fn load_markers(video_path: String) -> Result<Vec<Marker>, String> {
    let storage_dir = config_manager::get_markers_dir();
    load_markers_logic(&storage_dir, &video_path)
}





fn main() {
    // 软件启动时，立刻初始化一次工作区目录，确保文件夹被创建
    let workspace = config_manager::get_workspace_dir();
    println!("易剪启动成功！当前工作区路径: {}", workspace.display());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // 记得把新接口注册进来！
        .invoke_handler(tauri::generate_handler![
            check_ffmpeg_status,
            split_video,
            batch_split_by_duration,
            save_markers,
            load_markers,  //暴漏给前端的加载标记接口
            get_workspace_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
