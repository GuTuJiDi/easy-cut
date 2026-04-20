use futures::stream::{self, StreamExt};
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use crate::marker_manager::Marker;
use std::process::Stdio; // 用于控制底层输入输出管道
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
// ==========================================
// 1. 领域模型：万能任务载荷 (Payloads)
// ==========================================

#[derive(Debug, Clone)]
pub struct SplitPayload {
    pub start_time: f64,
    pub duration: f64,
    pub input_path: Arc<String>,
    pub output_path: String,
}

#[derive(Debug, Clone)]
pub struct AudioExtractPayload {
    pub start_time: f64,
    pub duration: f64,
    pub input_path: Arc<String>,
    pub output_path: String,
    // pub audio_bitrate: String,
}

#[derive(Debug, Clone)]
pub struct FormatConvertPayload {
    pub input_path: Arc<String>,
    pub output_path: String,
}

#[derive(Debug, Clone)]
pub enum MediaTask {
    Split(SplitPayload),
    ExtractAudio(AudioExtractPayload),
    ConvertFormat(FormatConvertPayload),
    ExtractSpecificAudio(SpecificAudioPayload),
    ExportPureVideo(PureVideoPayload),
}

#[derive(serde::Serialize)]
pub struct ExportResult {
    pub logs: String,
    pub target_dir: String,
}
// ==========================================
// 内部映射：FFprobe 原始 JSON 结构
// ==========================================
#[derive(Deserialize, Debug)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: FfprobeFormat,
}

#[derive(Deserialize, Debug)]
struct FfprobeStream {
    index: usize,
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    tags: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
struct FfprobeFormat {
    format_name: Option<String>,
    duration: Option<String>,
    size: Option<String>,
}

// ==========================================
// 前端 DTO：提纯后的媒体信息大满贯
// ==========================================
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MediaInfoDTO {
    pub format_name: String,
    pub duration_sec: f64,
    pub size_bytes: u64,
    pub video_streams: Vec<VideoStreamDTO>,
    pub audio_streams: Vec<AudioStreamDTO>,
    pub subtitle_streams: Vec<SubtitleStreamDTO>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct VideoStreamDTO {
    pub index: usize,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AudioStreamDTO {
    pub index: usize,
    pub codec: String,
    pub language: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SubtitleStreamDTO {
    pub index: usize,
    pub codec: String,
    pub language: String,
}
// ==========================================
// 1. 补充 Payload 定义
// ==========================================
#[derive(Debug, Clone)]
pub struct SpecificAudioPayload {
    pub input_path: Arc<String>,
    pub output_path: String,
    pub stream_index: usize, // 绝对流索引，由探测器提供
}

#[derive(Debug, Clone)]
pub struct PureVideoPayload {
    pub input_path: Arc<String>,
    pub output_path: String,
}
// ==========================================
// 🌟 核心防线：FFmpeg 路径安全包裹器
// ==========================================
/// 防止以 "-" 开头的文件名被 FFmpeg 误解析为指令参数 (Option Injection)
fn to_ffmpeg_safe_path(path: &str) -> String {
    let p = Path::new(path);
    if p.is_absolute() {
        // 绝对路径天然不会以 - 开头，绝对安全
        path.to_string()
    } else if path.starts_with('-') {
        // 如果是相对路径且以 - 开头，强制加上 ./ 伪装成目录指引，打破破壳条件
        format!("./{}", path)
    } else {
        path.to_string()
    }
}

// ==========================================
// 2. 原子执行层：智能策略路由 (Worker)
// ==========================================
pub async fn execute_media_task_async(task: &MediaTask) -> Result<String, String> {
    let mut cmd = Command::new("ffmpeg");

    cmd.kill_on_drop(true);   // 🌟 确保防僵尸进程大闸存在！
    cmd.stdin(Stdio::null()); // 物理层：拔掉输入管道
    cmd.arg("-nostdin");      // 逻辑层：明确告诉 FFmpeg 禁用交互模式
    cmd.arg("-y");            // 全局：强制覆盖输出文件

    // ⚡ 模式匹配：动态组装指令，并为所有输入输出注入路径安全护盾
    let (output_path, output) = match task {
        MediaTask::Split(p) => {
            let safe_in = to_ffmpeg_safe_path(&p.input_path);
            let safe_out = to_ffmpeg_safe_path(&p.output_path);

            let out = cmd
                .arg("-ss").arg(format!("{:.3}", p.start_time))
                .arg("-i").arg(&safe_in)
                .arg("-t").arg(format!("{:.3}", p.duration))
                .arg("-c").arg("copy")
                .arg("-avoid_negative_ts").arg("1")
                .arg(&safe_out)
                .output()
                .await.map_err(|e| format!("进程异常: {}", e))?;
            (&p.output_path, out)
        },
        // 修复 1：格式无损转换 (秒级)
        MediaTask::ConvertFormat(p) => {
            let safe_in = to_ffmpeg_safe_path(&p.input_path);
            let safe_out = to_ffmpeg_safe_path(&p.output_path);

            let out = cmd
                .arg("-i").arg(&safe_in)
                .arg("-c").arg("copy") // 🌟 核心：移除 libx264 和 aac，直接 copy
                .arg(&safe_out)
                .output()
                .await.map_err(|e| format!("进程异常: {}", e))?;
            (&p.output_path, out)
        },

        // 修复 2：音频物理提取 (秒级)
        MediaTask::ExtractAudio(p) => {
            let safe_in = to_ffmpeg_safe_path(&p.input_path);
            let safe_out = to_ffmpeg_safe_path(&p.output_path);

            let out = cmd
                .arg("-ss").arg(format!("{:.3}", p.start_time))
                .arg("-i").arg(&safe_in)
                .arg("-t").arg(format!("{:.3}", p.duration))
                .arg("-vn") // 丢弃视频流
                .arg("-c:a").arg("copy") // 🌟 核心：直接剥离原音轨。注意：前端产物后缀需改为 .m4a 或原格式
                .arg(&safe_out)
                .output()
                .await.map_err(|e| format!("进程异常: {}", e))?;
            (&p.output_path, out)
        },
        // ==========================================
        // 2. 在 execute_media_task_async 的 match 分支中补充指令
        // ==========================================
        MediaTask::ExtractSpecificAudio(p) => {
            let safe_in = to_ffmpeg_safe_path(&p.input_path);
            let safe_out = to_ffmpeg_safe_path(&p.output_path);

            let out = cmd
                .arg("-i").arg(&safe_in)
                // 🎯 精确制导：按探测到的绝对索引提取 (如 0:1)
                .arg("-map").arg(format!("0:{}", p.stream_index))
                .arg("-vn") // 丢弃视频
                .arg("-c:a").arg("copy") // 无损拷贝
                .arg(&safe_out)
                .output().await.map_err(|e| format!("进程异常: {}", e))?;
            (&p.output_path, out)
        },
        MediaTask::ExportPureVideo(p) => {
            let safe_in = to_ffmpeg_safe_path(&p.input_path);
            let safe_out = to_ffmpeg_safe_path(&p.output_path);

            let out = cmd
                .arg("-i").arg(&safe_in)
                .arg("-an") // 🔇 斩断所有音频流
                .arg("-sn") // 🚫 斩断所有软字幕流
                .arg("-c:v").arg("copy") // 画面无损拷贝
                .arg(&safe_out)
                .output().await.map_err(|e| format!("进程异常: {}", e))?;
            (&p.output_path, out)
        }
    };

    if output.status.success() {
        Ok(format!("✅ 生成: {}", output_path))
    } else {
        let file_name = Path::new(output_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let err_msg = String::from_utf8_lossy(&output.stderr);
        Err(format!("❌ 失败 [{}]: {}", file_name, err_msg))
    }
}

// ==========================================
// 3. 终极并发调度层 (Orchestrator)
// ==========================================
pub async fn run_concurrent_tasks(tasks: Vec<MediaTask>, max_concurrent: usize) -> Vec<String> {
    let concurrency_limit = max_concurrent.clamp(1, 32);
    let logs = Arc::new(Mutex::new(Vec::new()));

    stream::iter(tasks)
        .for_each_concurrent(concurrency_limit, |task| {
            let logs_clone = Arc::clone(&logs);
            async move {
                let result = execute_media_task_async(&task).await;
                let mut logs_lock = logs_clone.lock().await;
                match result {
                    Ok(msg) => logs_lock.push(msg),
                    Err(msg) => logs_lock.push(msg),
                }
            }
        })
        .await;

    let mut final_logs = logs.lock().await.clone();
    final_logs.sort();
    final_logs
}

// ==========================================
// 4. 业务逻辑层 (任务流水线 Planner)
// ==========================================
// 🌟 新增：智能音频编码探针
pub async fn probe_audio_codec(input_path: &str) -> Result<String, String> {
    let safe_in = to_ffmpeg_safe_path(input_path);

    let output = Command::new("ffprobe")
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .args([
            "-v", "error",
            "-select_streams", "a:0", // 只看第一条音轨
            "-show_entries", "stream=codec_name",
            "-of", "default=noprint_wrappers=1:nokey=1",
            &safe_in
        ])
        .output()
        .await
        .map_err(|e| format!("无法执行 FFprobe: {}", e))?;

    if output.status.success() {
        let codec = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if codec.is_empty() {
            Err("源视频中未检测到音频流".to_string())
        } else {
            Ok(codec) // 返回例如: "aac", "mp3", "ac3"
        }
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
pub fn plan_fixed_duration_splits(
    input_path: Arc<String>,
    total_duration: f64,
    segment_duration: f64,
    base_name: &str,
) -> Vec<MediaTask> {
    let mut tasks = Vec::new();
    let mut current_time = 0.0;
    let mut index = 1;

    while current_time < total_duration {
        let remain = total_duration - current_time;
        let duration = if remain < segment_duration { remain } else { segment_duration };

        tasks.push(MediaTask::Split(SplitPayload {
            start_time: current_time,
            duration,
            input_path: Arc::clone(&input_path),
            output_path: format!("{}_part{}.mp4", base_name, index),
        }));

        current_time += segment_duration;
        index += 1;
    }
    tasks
}

pub fn plan_marker_splits(
    input_path: Arc<String>,
    target_dir: &Path,
    video_stem: &str,
    extension: &str,
    markers: Vec<Marker>,
) -> Vec<MediaTask> {
    let mut tasks = Vec::new();
    let active_markers: Vec<Marker> = markers.into_iter().filter(|m| !m.is_deleted).collect();

    for (index, marker) in active_markers.into_iter().enumerate() {
        let duration = marker.end_time - marker.start_time;
        if duration <= 0.0 { continue; }

        let safe_label = sanitize_filename(&marker.label);
        let out_file_name = format!("[{:02}]_{}_{}.{}", index + 1, video_stem, safe_label, extension);
        let out_file_path = target_dir.join(&out_file_name);

        tasks.push(MediaTask::Split(SplitPayload {
            start_time: marker.start_time,
            duration,
            input_path: Arc::clone(&input_path),
            output_path: out_file_path.to_string_lossy().to_string(),
        }));
    }
    tasks
}

fn sanitize_filename(name: &str) -> String {
    name.replace(&['\\', '/', ':', '*', '?', '"', '<', '>', '|'][..], "_")
}

// 异步探测时长
pub async fn get_video_duration(input_path: &str) -> Result<f64, String> {
    let safe_in = to_ffmpeg_safe_path(input_path); // 🌟 探针也必须使用安全路径护盾

    let output = Command::new("ffprobe")
        .kill_on_drop(true)   // 🌟 为探针补齐防僵尸进程保护
        .stdin(Stdio::null())
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", &safe_in])
        .output()
        .await
        .map_err(|e| format!("无法执行 FFprobe: {}", e))?;

    if output.status.success() {
        let duration_str = String::from_utf8_lossy(&output.stdout);
        duration_str.trim().parse::<f64>().map_err(|_| "解析视频时长数据失败".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// ==========================================
// 探针执行引擎
// ==========================================
pub async fn probe_media_info(input_path: &str) -> Result<MediaInfoDTO, String> {
    let safe_in = to_ffmpeg_safe_path(input_path);

    let output = Command::new("ffprobe")
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            &safe_in
        ])
        .output()
        .await
        .map_err(|e| format!("FFprobe 启动失败: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let json_str = String::from_utf8_lossy(&output.stdout);

    // 1. 反序列化原始数据
    let probe_data: FfprobeOutput = serde_json::from_str(&json_str)
        .map_err(|e| format!("解析 FFprobe JSON 失败: {}", e))?;

    // 2. 初始化分类容器
    let mut video_streams = Vec::new();
    let mut audio_streams = Vec::new();
    let mut subtitle_streams = Vec::new();

    // 3. 遍历并分类提取轨道信息
    for stream in probe_data.streams {
        let codec = stream.codec_name.unwrap_or_else(|| "unknown".to_string());
        let tags = stream.tags.unwrap_or_default();
        let lang = tags.get("language").cloned().unwrap_or_else(|| "und".to_string()); // und = undefined

        match stream.codec_type.as_str() {
            "video" => {
                // 计算帧率 (例如 "60000/1001" -> 59.94)
                let mut fps = 0.0;
                if let Some(rate_str) = stream.r_frame_rate {
                    let parts: Vec<&str> = rate_str.split('/').collect();
                    if parts.len() == 2 {
                        let num: f64 = parts[0].parse().unwrap_or(0.0);
                        let den: f64 = parts[1].parse().unwrap_or(1.0);
                        if den > 0.0 { fps = num / den; }
                    }
                }

                // 排除封面图 (mjpeg/png/bmp 经常被误识别为视频流)
                if codec != "mjpeg" && codec != "png" && codec != "bmp" {
                    video_streams.push(VideoStreamDTO {
                        index: stream.index,
                        codec,
                        width: stream.width.unwrap_or(0),
                        height: stream.height.unwrap_or(0),
                        fps,
                    });
                }
            },
            "audio" => {
                audio_streams.push(AudioStreamDTO { index: stream.index, codec, language: lang });
            },
            "subtitle" => {
                subtitle_streams.push(SubtitleStreamDTO { index: stream.index, codec, language: lang });
            },
            _ => {} // 忽略 attachment 或 data 数据流
        }
    }

    // 4. 提取全局格式元数据
    let format = probe_data.format;
    let duration_sec = format.duration.unwrap_or_default().parse::<f64>().unwrap_or(0.0);
    let size_bytes = format.size.unwrap_or_default().parse::<u64>().unwrap_or(0);
    let format_name = format.format_name.unwrap_or_else(|| "unknown".to_string());

    Ok(MediaInfoDTO {
        format_name,
        duration_sec,
        size_bytes,
        video_streams,
        audio_streams,
        subtitle_streams,
    })
}