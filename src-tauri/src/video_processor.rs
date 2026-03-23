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
        let duration = if remain < segment_duration { remain } else { segment_duration };

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
pub fn execute_ffmpeg_split(
    input_path: &str,
    task: &SplitTask,
) -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .arg("-y")
        .arg("-ss").arg(task.start_time.to_string())
        .arg("-i").arg(input_path)
        .arg("-t").arg(task.duration.to_string())
        .arg("-c").arg("copy")
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

// ==========================================
// 测试左移：Bob 的后端单元测试 (Unit Tests)
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

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
}