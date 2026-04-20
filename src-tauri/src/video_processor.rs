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
    // 🌟 新增：处理 6 种语义的多态导出任务
    ExportMarker(MarkerExportPayload),
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
    #[serde(default)]
    pub title: Option<String>, // 🌟 新增：轨道标题
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AudioStreamDTO {
    pub index: usize,
    pub codec: String,
    pub language: String,
    #[serde(default)]
    pub title: Option<String>, // 🌟 新增：轨道标题
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SubtitleStreamDTO {
    pub index: usize,
    pub codec: String,
    pub language: String,
    #[serde(default)]
    pub title: Option<String>, // 🌟 新增：轨道标题
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


// 🌟 1. 在文件顶部的 DTO 区域，增加标记导出的专用载荷
#[derive(Debug, Clone)]
pub struct MarkerExportPayload {
    pub start_time: f64,
    pub duration: f64,
    pub input_path: Arc<String>,
    pub output_path: String,
    pub export_type: String,       // 指令核心："master", "pure_video", "audio_only" 等
    pub track_index: Option<usize>, // 针对特定音轨的索引
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
        },
        // 👇 新增的多态路由分支 👇
        MediaTask::ExportMarker(p) => {
            let safe_in = to_ffmpeg_safe_path(&p.input_path);
            let safe_out = to_ffmpeg_safe_path(&p.output_path);

            cmd.arg("-ss").arg(format!("{:.3}", p.start_time))
                .arg("-i").arg(&safe_in)
                .arg("-t").arg(format!("{:.3}", p.duration));

            // 🎯 核心：根据不同语义动态挂载物理层剔除参数
            match p.export_type.as_str() {
                "no_subs" => { cmd.args(["-sn", "-c", "copy"]); },
                "pure_video" => { cmd.args(["-an", "-sn", "-c:v", "copy"]); },
                "iso_track" => {
                    let idx = p.track_index.unwrap_or(0);
                    // 🌟 修复：使用绝对索引 0:{}，精准保留视频和指定的某一条音轨
                    cmd.args(["-map", "0:v", "-map", &format!("0:{}", idx), "-c", "copy"]);
                },
                "audio_only" => {
                    let idx = p.track_index.unwrap_or(0);
                    // 🌟 修复：去除 0:a:{} 相对映射，使用绝对索引 0:{}
                    cmd.args(["-vn", "-map", &format!("0:{}", idx), "-c:a", "copy"]);
                },
                "subs_only" => {
                    let idx = p.track_index.unwrap_or(0);
                    // 🌟 修复：精确提取特定的字幕轨，而不是提取全部字幕
                    cmd.args(["-vn", "-an", "-map", &format!("0:{}", idx), "-c:s", "copy"]);
                },
                _ => { cmd.args(["-c", "copy"]); } // 默认 master 态
            }

            cmd.arg("-avoid_negative_ts").arg("1");
            cmd.arg(&safe_out);

            let out = cmd.output().await.map_err(|e| format!("进程异常: {}", e))?;
            (&p.output_path, out)
        }
    };

    if output.status.success() {
        Ok(format!("✅ 生成: {}", output_path))
    } else {
        // 🌟 核心修复：如果执行失败，立即把残次品文件删掉，防止下次触发幽灵缓存！
        let _ = std::fs::remove_file(&output_path);
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

/*pub fn plan_marker_splits(
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
*/
// 🌟 找到 video_processor.rs 中的 plan_marker_splits 函数，替换为以下完整代码：

pub fn plan_marker_splits(
    input_path: Arc<String>,
    target_dir: &Path,
    video_stem: &str,
    default_ext: &str,
    markers: Vec<Marker>,
) -> Vec<MediaTask> {
    let mut tasks = Vec::new();
    let active_markers: Vec<Marker> = markers.into_iter().filter(|m| !m.is_deleted).collect();

    for (index, marker) in active_markers.into_iter().enumerate() {
        let duration = marker.end_time - marker.start_time;
        if duration <= 0.0 { continue; }

        let safe_label = sanitize_filename(&marker.label);

        let mut export_type = "master".to_string();
        let mut track_index = None;

        // 🌟 核心修复：完美兼容前端嵌套包 (payload.payload.export_strategy) 与扁平包
        let strategy_opt = marker.payload.get("export_strategy")
            .or_else(|| marker.payload.get("payload").and_then(|p| p.get("export_strategy")));

        if let Some(strategy) = strategy_opt {
            if let Some(t) = strategy.get("type").and_then(|v| v.as_str()) {
                export_type = t.to_string();
            }
            if let Some(idx) = strategy.get("target_audio_stream").and_then(|v| v.as_u64()) {
                track_index = Some(idx as usize);
            }
        }

        // 📂 智能路由：根据读取到的精准语义，分配子文件夹与物理后缀名
        let (sub_dir_name, ext) = match export_type.as_str() {
            "master" => ("01_Master_Clips", default_ext),
            "no_subs" => ("02_Clean_Feed", default_ext),
            "pure_video" => ("03_B_Roll", default_ext),
            "iso_track" => ("04_Iso_Tracks", default_ext),
            "audio_only" => ("05_Audio", "m4a"), // 🎧 纯物理音频必然是 m4a
            "subs_only" => ("06_Subtitles", "srt"), // 📝 纯字幕提取
            _ => ("00_Uncategorized", default_ext),
        };

        // 自动创建对应的分类子目录
        let sub_dir = target_dir.join(sub_dir_name);
        let _ = std::fs::create_dir_all(&sub_dir);

        // 组装最终的安全文件名与路径
        let out_file_name = format!("[{:02}]_{}_{}.{}", index + 1, video_stem, safe_label, ext);
        let out_file_path = sub_dir.join(&out_file_name);

        tasks.push(MediaTask::ExportMarker(MarkerExportPayload {
            start_time: marker.start_time,
            duration,
            input_path: Arc::clone(&input_path),
            output_path: out_file_path.to_string_lossy().to_string(),
            export_type,
            track_index,
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

        // 抓取语言标签（兼容大小写）
        let lang = tags.get("language")
            .or_else(|| tags.get("LANGUAGE"))
            .cloned()
            .unwrap_or_else(|| "und".to_string()); // und = undefined

        // 🌟 核心升级：抓取轨道标题标签（兼容大小写）
        let title = tags.get("title")
            .or_else(|| tags.get("TITLE"))
            .cloned();

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
                        title, // 👈 将标题注入 DTO
                    });
                }
            },
            "audio" => {
                audio_streams.push(AudioStreamDTO {
                    index: stream.index,
                    codec,
                    language: lang,
                    title  // 👈 将标题注入 DTO
                });
            },
            "subtitle" => {
                subtitle_streams.push(SubtitleStreamDTO {
                    index: stream.index,
                    codec,
                    language: lang,
                    title  // 👈 将标题注入 DTO
                });
            },
            _ => {} // 忽略 attachment 或 data 等无用数据流
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