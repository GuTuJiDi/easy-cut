// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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

// ===  新增的核心切割逻辑 ===
#[tauri::command]
async fn split_video(
    input_path: String,
    output_path: String,
    start_time: String,
    duration: String,
) -> Result<String, String> {
    // 补齐了最后一个 arg(&output_path)
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss").arg(&start_time)
        .arg("-i").arg(&input_path)
        .arg("-t").arg(&duration)
        .arg("-c").arg("copy")
        .arg(&output_path) // <-- 就是漏了这一行！把输出路径传给 FFmpeg
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(format!("视频分割成功！已保存至: {}", output_path))
            } else {
                let err_msg = String::from_utf8_lossy(&out.stderr);
                Err(format!("分割失败: {}", err_msg))
            }
        }
        Err(e) => Err(format!("无法执行 FFmpeg 进程: {}", e)),
    }
}

fn main() {
    tauri::Builder::default()
        // 注册刚刚编写的命令
        .invoke_handler(tauri::generate_handler![check_ffmpeg_status,split_video])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}