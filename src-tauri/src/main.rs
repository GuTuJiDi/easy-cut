// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 引入刚刚新建的模块
mod video_processor;
use video_processor::{plan_fixed_duration_splits, execute_ffmpeg_split, SplitTask};
use std::process::Command;

// 定义一个暴露给前端的 Tauri Command
#[tauri::command]
async fn check_ffmpeg_status() -> Result<String, String> {
    // 尝试调用本地的 ffmpeg 命令
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output();

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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![check_ffmpeg_status, split_video])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}