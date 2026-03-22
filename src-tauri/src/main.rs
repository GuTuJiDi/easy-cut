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

fn main() {
    tauri::Builder::default()
        // 注册刚刚编写的命令
        .invoke_handler(tauri::generate_handler![check_ffmpeg_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}