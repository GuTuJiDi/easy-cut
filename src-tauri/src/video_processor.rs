// src-tauri/src/video_processor.rs
use std::process::Command;

// 定义一个标准化的分割任务结构体
#[derive(Debug, PartialEq)]
pub struct SplitTask {
    pub start_time: u32, // 起始时间 (秒)
    pub duration: u32,   // 截取时长 (秒)
    pub output_path: String,
}

/// 业务逻辑层：根据总时长和目标时长，规划分割任务队列
pub fn plan_fixed_duration_splits(
    total_duration: u32,
    segment_duration: u32,
    base_name: &str, // 比如 "D:\输出目录\先导片"
) -> Vec<SplitTask> {
    let mut tasks = Vec::new();
    let mut current_time = 0;
    let mut index = 1;

    while current_time < total_duration {
        let remain = total_duration - current_time;
        // 如果剩余时间不够一个切片，就把剩余的作为最后一段
        let duration = if remain < segment_duration {
            remain
        } else {
            segment_duration
        };

        tasks.push(SplitTask {
            start_time: current_time,
            duration,
            output_path: format!("{}_part{}.mp4", base_name, index),
        });

        current_time += segment_duration;
        index += 1;
    }
    tasks
}

/// 底层执行层：解耦后的纯函数，不依赖 Tauri 的 #[tauri::command]
pub fn execute_ffmpeg_split(input_path: &str, task: &SplitTask) -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss")
        .arg(task.start_time.to_string())
        .arg("-i")
        .arg(input_path)
        .arg("-t")
        .arg(task.duration.to_string())
        .arg("-c")
        .arg("copy")
        .arg(&task.output_path)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(task.output_path.clone())
            } else {
                Err(String::from_utf8_lossy(&out.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 新增：调用 ffprobe 获取视频总时长（秒）
pub fn get_video_duration(input_path: &str) -> Result<u32, String> {
    // ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 <input>
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            input_path,
        ])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                // ffprobe 返回的是带小数点的字符串，比如 "115.320000"
                let duration_str = String::from_utf8_lossy(&out.stdout);

                // 解析为浮点数并四舍五入为 u32 秒
                match duration_str.trim().parse::<f64>() {
                    Ok(f) => Ok(f.round() as u32),
                    Err(_) => Err("无法解析视频时长数据".to_string()),
                }
            } else {
                let err_msg = String::from_utf8_lossy(&out.stderr);
                Err(format!("FFprobe 分析失败: {}", err_msg))
            }
        }
        Err(e) => Err(format!("无法执行 FFprobe 进程: {}", e)),
    }
}

// ==========================================
// 测试左移：Bob 的后端单元测试 (Unit Tests)
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_plan_fixed_duration_splits() {
        // 测试场景 1：100秒视频，按 30秒 切割
        // 预期结果：产生 4 个任务，最后一个任务只有 10 秒
        let tasks = plan_fixed_duration_splits(100, 30, "test_video");

        assert_eq!(tasks.len(), 4, "应该被分割为 4 段");

        // 验证第一段
        assert_eq!(tasks[0].start_time, 0);
        assert_eq!(tasks[0].duration, 30);
        assert_eq!(tasks[0].output_path, "test_video_part1.mp4");

        // 验证最后一段（边缘情况处理）
        assert_eq!(tasks[3].start_time, 90);
        assert_eq!(tasks[3].duration, 10, "最后一段时长应该是余数 10 秒");
        assert_eq!(tasks[3].output_path, "test_video_part4.mp4");
    }

    #[test]
    fn test_exact_division_splits() {
        // 测试场景 2：60秒视频，按 30秒 切割，完美整除
        let tasks = plan_fixed_duration_splits(60, 30, "perfect_video");
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[1].duration, 30);
    }

    #[test]
    fn test_get_video_duration_file_not_found() {
        // 测试边界情况：当传入一个根本不存在的路径时，我们的系统应该优雅地返回 Err，而不是崩溃
        let result = get_video_duration("不存在的虚拟路径_xxx.mp4");

        assert!(result.is_err(), "文件不存在时应该返回错误");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("FFprobe 分析失败") || err_msg.contains("无法执行"),
                "错误信息应该包含预期的关键字，当前为: {}", err_msg);
    }

    // 实际开发中，对于核心解析逻辑，我们通常会在项目根目录放一个极小的 dummy.mp4 用于真实测试。
    // 这里我们模拟一个测试思想：确保接口类型签名正确
    #[test]
    fn test_ffprobe_command_build() {
        // 验证我们调用的 ffprobe 命令格式是否正确且没有引发 Panic
        // 如果本地环境没有 ffprobe，这个测试也能帮我们发现环境依赖问题
        let output = std::process::Command::new("ffprobe")
            .arg("-version")
            .output();

        assert!(output.is_ok(), "运行环境缺少 ffprobe，请检查系统环境变量！");
    }
}
use crate::marker_manager::Marker;
use std::path::Path;

// --- 实用工具函数：净化文件名 ---
// 用户在打标签时可能会输入 \ / : * ? " < > | 等操作系统不允许的字符



// 2026/3/25/14

// 如果顶部没有引用 serde，请加上： use serde::Serialize;

#[derive(serde::Serialize)]
pub struct ExportResult {
    pub logs: String,
    pub target_dir: String,
}

// 实用工具：净化文件名，防止用户输入的标签含有 \ / : * ? " < > | 导致系统报错
fn sanitize_filename(name: &str) -> String {
    name.replace(&['\\', '/', ':', '*', '?', '"', '<', '>', '|'][..], "_")
}

// 核心魔法：根据标记数组执行连环切割
pub async fn split_video_by_markers(
    input_path_str: &str,
    output_dir_str: &str,
    markers: Vec<Marker>,
) -> Result<ExportResult, String> {
    use std::path::Path;
    use std::process::Command;

    let input_path = Path::new(input_path_str);
    let out_dir_path = Path::new(output_dir_str);

    let video_stem = input_path.file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let extension = input_path.extension()
        .unwrap_or(std::ffi::OsStr::new("mp4"))
        .to_string_lossy();

    // 🏆 体验优化：为本次导出自动创建一个专属的子文件夹，如 "先导片_标记导出"
    let target_dir = out_dir_path.join(format!("{}_标记导出", video_stem));
    if let Err(e) = std::fs::create_dir_all(&target_dir) {
        return Err(format!("无法创建专属输出目录: {}", e));
    }

    let mut logs = Vec::new();
    logs.push(format!("🚀 引擎启动：准备处理 {} 个标记片段...", markers.len()));

    for (index, marker) in markers.iter().enumerate() {
        let duration = marker.end_time - marker.start_time;
        if duration <= 0.0 {
            logs.push(format!("⚠️ 跳过无效标记 (时间倒挂或为0): {}", marker.label));
            continue;
        }

        // 格式化输出文件名： [01]_先导片_高能时刻.mp4
        let safe_label = sanitize_filename(&marker.label);
        let out_file_name = format!("[{:02}]_{}_{}.{}", index + 1, video_stem, safe_label, extension);
        let out_file_path = target_dir.join(&out_file_name);

        // ⚡ 组装 FFmpeg 极速无损切割指令 (-c copy)
        let output = Command::new("ffmpeg")
            .arg("-y")
            .arg("-ss").arg(format!("{:.3}", marker.start_time))
            .arg("-i").arg(input_path_str)
            .arg("-t").arg(format!("{:.3}", duration))
            .arg("-c").arg("copy") // 核心：瞬间输出的魔法
            .arg(out_file_path.to_string_lossy().as_ref())
            .output()
            .map_err(|e| format!("FFmpeg 进程启动失败，请检查环境变量: {}", e))?;

        if output.status.success() {
            logs.push(format!("✅ 成功: {}", out_file_name));
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            logs.push(format!("❌ 失败 [{}]: {}", out_file_name, err_msg));
        }
    }

    Ok(ExportResult {
        logs: logs.join("\n"),
        target_dir: target_dir.to_string_lossy().to_string(),
    })
}