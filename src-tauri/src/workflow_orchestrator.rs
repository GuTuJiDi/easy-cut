use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::video_processor;
use crate::config_manager;
use crate::marker_manager;
use crate::auth;
use crate::video_processor::{
    AudioStreamDTO, SpecificAudioPayload, PureVideoPayload, MediaTask
};
// ==========================================
// 1. 数据传输对象 (DTOs) - 严格定义每个工作流的入参
// ==========================================
// 🌟 音频极速分离入参 (PRO)
#[derive(serde::Deserialize)]
pub struct AudioExtractParams {
    pub input_path: String,
    pub output_dir: String,
    // pub output_name: String,
    // pub bitrate: String,
    pub start_time: f64,
    pub duration: f64,
    pub license_str: String, // 🛡️ 强制大闸凭证
}

// 🌟 视频格式转换入参 (PRO)
#[derive(serde::Deserialize)]
pub struct FormatConvertParams {
    pub input_path: String,
    pub output_dir: String,
    pub output_name: String, // 必须带有目标扩展名，如 .mkv
    pub license_str: String, // 🛡️ 强制大闸凭证
}

#[derive(serde::Deserialize)]
pub struct DurationSplitParams {
    pub input_path: String,
    pub output_dir: String,
    pub video_name: String,
    pub segment_duration: f64,
    pub max_concurrent_tasks: usize,
}

#[derive(serde::Deserialize)]
pub struct CountSplitParams {
    pub input_path: String,
    pub output_dir: String,
    pub video_name: String,
    pub segment_count: usize,
    pub max_concurrent_tasks: usize,
    // 🛡️ PRO 专属标识：必须携带激活码！
    pub license_str: String,
}
// 单片段提取入参结构体
#[derive(serde::Deserialize)]
pub struct SingleExtractParams {
    pub video_path: String,
    pub start_time: f64,
    pub end_time: f64,
    pub expected_batch_path: Option<String>,
    pub fallback_output_dir: String,
    pub fallback_file_name: String,
    pub export_type: String,         // 🌟 新增：接收前端传来的语义类型
    pub track_index: Option<usize>,  // 🌟 新增：接收特定的音轨索引
}


#[derive(serde::Deserialize)]
pub struct MarkerSplitParams {
    pub video_path: String,
    pub output_dir: String,
    pub max_concurrent_tasks: usize,
    // 可选：如果是免费版，可以限制打轴导出数量；PRO版无限
    pub license_str: Option<String>,
}
#[derive(serde::Deserialize)]
pub struct AutoExtractAudioParams {
    pub input_path: String,
    pub output_dir: String,
    pub audio_streams: Vec<AudioStreamDTO>, // 前端传来的探测结果
}

#[derive(serde::Deserialize)]
pub struct PureVideoParams {
    pub input_path: String,
    pub output_dir: String,
}
// ==========================================
// 2. 工作流总线 (Workflows) - 高内聚的业务编排
// ==========================================

/// 🟢 基础免费工作流：固定时长分割
pub async fn run_duration_split(params: DurationSplitParams) -> Result<String, String> {
    // 1. IO 与元数据准备
    let total_duration = video_processor::get_video_duration(&params.input_path).await?;
    let target_folder = Path::new(&params.output_dir).join(&params.video_name);
    fs::create_dir_all(&target_folder).map_err(|e| format!("IO错误: {}", e))?;

    // 2. 领域逻辑：规划任务
    let base_name = target_folder.join(&params.video_name).to_string_lossy().to_string();
    let input_arc = Arc::new(params.input_path);
    let tasks = video_processor::plan_fixed_duration_splits(
        input_arc, total_duration, params.segment_duration, &base_name
    );

    // 3. 驱动底层并发引擎
    let logs = video_processor::run_concurrent_tasks(tasks, params.max_concurrent_tasks).await;
    Ok(logs.join("\n"))
}

/// 👑 旗舰 PRO 工作流：固定数量分割 (自带商业大闸)
pub async fn run_count_split(params: CountSplitParams) -> Result<String, String> {
    // 🛡️ 商业防线：进门先查票！黑客绕不过这段 Rust 原生校验。
    let _payload = auth::verify_license_internal(&params.license_str)
        .map_err(|e| format!("🚨 核心授权拦截：未激活 PRO 版本 ({})", e))?;

    // 1. IO 与元数据准备
    let total_duration = video_processor::get_video_duration(&params.input_path).await?;
    let target_folder = Path::new(&params.output_dir).join(&params.video_name);
    fs::create_dir_all(&target_folder).map_err(|e| format!("IO错误: {}", e))?;

    // 2. 领域逻辑：组装高级等分任务
    let input_arc = Arc::new(params.input_path);
    let mut tasks = Vec::new();
    let segment_duration = total_duration / (params.segment_count as f64);

    for i in 0..params.segment_count {
        let start_time = (i as f64) * segment_duration;
        let out_path = target_folder.join(format!("{}_part{}.mp4", params.video_name, i + 1));

        // 核心算法：最后一段不限制时长，直达末尾防丢帧
        let duration_val = if i == params.segment_count - 1 { total_duration - start_time } else { segment_duration };

        tasks.push(video_processor::MediaTask::Split(video_processor::SplitPayload {
            start_time,
            duration: duration_val,
            input_path: Arc::clone(&input_arc),
            output_path: out_path.to_string_lossy().to_string(),
        }));
    }

    // 3. 驱动底层并发引擎
    let logs = video_processor::run_concurrent_tasks(tasks, params.max_concurrent_tasks).await;
    Ok(logs.join("\n"))
}

/// 🟢/👑 混合工作流：智能打轴导出
pub async fn run_marker_split(params: MarkerSplitParams) -> Result<video_processor::ExportResult, String> {
    // 1. 数据读取
    let storage_dir = config_manager::get_markers_dir();
    // 🌟 修复：补上 .await 等待异步结果
    let project = marker_manager::load_project_logic(&storage_dir, &params.video_path).await?;

    if project.markers.is_empty() {
        return Err("⚠️ 没有任何有效标记片段！".to_string());
    }

    // 此处预留了商业化空间：比如 project.markers.len() > 5 且 license_str 无效时，报错拦截
    // 🌟 修复：补齐打轴商业化大闸
    let marker_count = project.markers.len();
    if marker_count > 5 {
        let is_pro = match &params.license_str {
            Some(key) => auth::verify_license_internal(key).is_ok(),
            None => false,
        };

        if !is_pro {
            return Err(format!(
                "🚨 免费版单次最多支持导出 5 个片段（当前: {}）。请升级 PRO 旗舰版解锁无限制并发导出！",
                marker_count
            ));
        }
    }
    // 2. IO 与元数据准备
    let input_arc = Arc::new(params.video_path);
    let input_path_obj = Path::new(&*input_arc);

    let video_stem = input_path_obj.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let extension = input_path_obj.extension().unwrap_or(std::ffi::OsStr::new("mp4")).to_string_lossy().to_string();

    /*let target_dir = Path::new(&params.output_dir).join(format!("{}_标记导出", video_stem));
    fs::create_dir_all(&target_dir).map_err(|e| format!("无法创建输出目录: {}", e))?;
*/
    // 👇 将其替换为更符合专业工作流的命名 👇
    let target_dir = Path::new(&params.output_dir).join(format!("{}_素材包", video_stem));
    fs::create_dir_all(&target_dir).map_err(|e| format!("无法创建素材包目录: {}", e))?;
    // 3. 领域逻辑：规划打轴任务
    let tasks = video_processor::plan_marker_splits(
        Arc::clone(&input_arc), &target_dir, &video_stem, &extension, project.markers
    );

    // 4. 驱动底层并发引擎
    let task_count = tasks.len();
    let mut logs = video_processor::run_concurrent_tasks(tasks, params.max_concurrent_tasks).await;
    logs.insert(0, format!("🚀 引擎启动：成功并发处理 {} 个片段...", task_count));

    Ok(video_processor::ExportResult {
        logs: logs.join("\n"),
        target_dir: target_dir.to_string_lossy().to_string(),
    })
}


/// 🌟 独立片段智能提取 (供工作流无缝跳转使用)
pub async fn extract_single_segment(params: SingleExtractParams) -> Result<String, String> {
    // 1. 智能探针：检查是否已经在“一键批量导出”中生成过了
    if let Some(batch_path) = &params.expected_batch_path {
        let path = Path::new(batch_path);
        if path.exists() {
            return Ok(path.to_string_lossy().to_string()); // 文件已存在，耗时0ms直接复用！
        }
    }

    // 2. 检查后备目录下是否已经单独提取过（缓存判定）
    let out_path = Path::new(&params.fallback_output_dir).join(&params.fallback_file_name);
    let out_path_str = out_path.to_string_lossy().to_string();
    if out_path.exists() {
        return Ok(out_path_str); // 文件已存在，耗时0ms直接复用！
    }

    // 🌟 3. 核心修复：如果都不存在，正式调用物理切片提取 (改为使用全新的多态执行器)
    let task = video_processor::MediaTask::ExportMarker(video_processor::MarkerExportPayload {
        start_time: params.start_time,
        duration: params.end_time - params.start_time,
        input_path: Arc::new(params.video_path),
        output_path: out_path_str.clone(),
        export_type: params.export_type, // 👈 动态应用前端传来的策略
        track_index: params.track_index, // 👈 动态应用目标音轨
    });

    video_processor::execute_media_task_async(&task).await?;
    Ok(out_path_str)
}

/// 👑 旗舰 PRO 工作流：音频极速分离 (智能探针版)
pub async fn run_audio_extract(params: AudioExtractParams) -> Result<String, String> {
    // 🛡️ 商业防线：查票！
    let _payload = auth::verify_license_internal(&params.license_str)
        .map_err(|e| format!("🚨 音频分离授权拦截：未激活 PRO 版本 ({})", e))?;

    let target_folder = Path::new(&params.output_dir);
    fs::create_dir_all(&target_folder).map_err(|e| format!("IO错误: {}", e))?;

    // 🌟 1. 探针介入：获取真实的底层音频编码
    let codec = video_processor::probe_audio_codec(&params.input_path).await?;

    // 🌟 2. 智能匹配：将编码映射为正确的物理文件后缀
    let ext = match codec.as_str() {
        "aac" => "m4a",
        "mp3" => "mp3",
        "ac3" => "ac3",
        "flac" => "flac",
        "wav" | "pcm_s16le" | "pcm_s24le" => "wav",
        "vorbis" => "ogg",
        "opus" => "opus",
        _ => &codec, // 保底策略：直接用编码名做后缀
    };

    // 🌟 3. 动态生成最终的安全文件名
    let input_path_obj = Path::new(&params.input_path);
    let video_stem = input_path_obj.file_stem().unwrap_or_default().to_string_lossy();
    let safe_output_name = format!("{}_物理原轨.{}", video_stem, ext);
    let out_path = target_folder.join(&safe_output_name);

    let task = video_processor::MediaTask::ExtractAudio(video_processor::AudioExtractPayload {
        start_time: params.start_time,
        duration: params.duration,
        input_path: Arc::new(params.input_path),
        output_path: out_path.to_string_lossy().to_string(),
    });

    let logs = video_processor::run_concurrent_tasks(vec![task], 1).await;
    Ok(logs.join("\n"))
}
/// 👑 旗舰 PRO 工作流：视频格式转换
pub async fn run_format_convert(params: FormatConvertParams) -> Result<String, String> {
    // 🛡️ 商业防线：查票！
    let _payload = auth::verify_license_internal(&params.license_str)
        .map_err(|e| format!("🚨 格式转换授权拦截：未激活 PRO 版本 ({})", e))?;

    let target_folder = Path::new(&params.output_dir);
    fs::create_dir_all(&target_folder).map_err(|e| format!("IO错误: {}", e))?;
    // 🌟 修复：强制防御路径穿越注入
    let safe_output_name = Path::new(&params.output_name)
        .file_name() // 强行只取纯文件名，过滤所有斜杠
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let out_path = target_folder.join(&safe_output_name);

    let task = video_processor::MediaTask::ConvertFormat(video_processor::FormatConvertPayload {
        input_path: Arc::new(params.input_path),
        output_path: out_path.to_string_lossy().to_string(),
    });

    let logs = video_processor::run_concurrent_tasks(vec![task], 1).await;
    Ok(logs.join("\n"))
}



/// 🚀 自动化脚本 1：一键分离所有音轨
pub async fn run_extract_all_audio(params: AutoExtractAudioParams) -> Result<String, String> {
    let target_folder = Path::new(&params.output_dir);
    std::fs::create_dir_all(&target_folder).map_err(|e| e.to_string())?;

    let input_arc = Arc::new(params.input_path.clone());
    let video_stem = Path::new(&params.input_path).file_stem().unwrap_or_default().to_string_lossy();

    let mut tasks = Vec::new();

    for stream in params.audio_streams {
        // 动态适配后缀
        // 动态适配后缀
        let ext = match stream.codec.as_str() {
            "aac" => "m4a",
            "eac3" => "eac3", // 👈 新增 EAC3 支持
            "ac3" => "ac3",
            "wav" => "wav",
            "mp3" => "mp3",
            _ => "mka" // 👈 兜底改为万能的 mka
        };
        // 生成如：VideoName_Track1_chi.m4a
        let out_name = format!("{}_Track{}_{}.{}", video_stem, stream.index, stream.language, ext);
        let out_path = target_folder.join(out_name).to_string_lossy().to_string();

        tasks.push(MediaTask::ExtractSpecificAudio(SpecificAudioPayload {
            input_path: Arc::clone(&input_arc),
            output_path: out_path,
            stream_index: stream.index,
        }));
    }

    // 满载并发执行（假设 4 线程）
    let logs = video_processor::run_concurrent_tasks(tasks, 4).await;
    Ok(logs.join("\n"))
}

/// 🚀 自动化脚本 2：一键剥离出纯视频
pub async fn run_export_pure_video(params: PureVideoParams) -> Result<String, String> {
    let target_folder = Path::new(&params.output_dir);
    std::fs::create_dir_all(&target_folder).map_err(|e| e.to_string())?;

    let input_path_obj = Path::new(&params.input_path);
    let video_stem = input_path_obj.file_stem().unwrap_or_default().to_string_lossy();
    let ext = input_path_obj.extension().unwrap_or_default().to_string_lossy();

    let out_name = format!("{}_PureVideo.{}", video_stem, ext);
    let out_path = target_folder.join(out_name).to_string_lossy().to_string();

    let task = MediaTask::ExportPureVideo(PureVideoPayload {
        input_path: Arc::new(params.input_path),
        output_path: out_path,
    });

    let logs = video_processor::run_concurrent_tasks(vec![task], 1).await;
    Ok(logs.join("\n"))
}