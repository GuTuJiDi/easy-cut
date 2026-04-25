use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::State;
use crate::video_processor;
use crate::config_manager;
use crate::marker_manager;
use crate::auth;
use crate::auth::SecurityGuardian;
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
    // pub license_str: String, // 🛡️ 强制大闸凭证
}

// 🌟 视频格式转换入参 (PRO)
#[derive(serde::Deserialize)]
pub struct FormatConvertParams {
    pub input_path: String,
    pub output_dir: String,
    pub output_name: String, // 必须带有目标扩展名，如 .mkv
    // pub license_str: String, // 🛡️ 强制大闸凭证
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
    // pub license_str: String,
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
pub async fn run_duration_split(
    params: DurationSplitParams,
    guardian: &State<'_, SecurityGuardian> // 👈 注入锁以判定免费/付费身份
) -> Result<String, String> {
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
    // 🚀 计算安全并发：即使是免费版的功能，如果用户没买 PRO，并发也会被压到 1
    let final_concurrency = resolve_safe_concurrency(params.max_concurrent_tasks, guardian);
    let logs = video_processor::run_concurrent_tasks(tasks, final_concurrency).await;
    Ok(logs.join("\n"))
}

/// 👑 旗舰 PRO 工作流：固定数量分割 (自带商业大闸)
/// /// 现在接收 SecurityGuardian 引用和 session_token 进行零信任校验
pub async fn run_count_split(
    params: CountSplitParams,
    guardian: &State<'_, SecurityGuardian>, // 注入安全守护者
    token: &str                        // 注入前端传来的令牌
) -> Result<String, String> {
    // 🛡️ 铁穹门禁：内核级校验令牌与 PRO 状态
    auth::check_pro_gate(guardian, token)?;

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
    // 之前是硬编码 4 线程，现在走智能裁决 (假设前端没有传设置的并发数，默认给最大能力)
    let hardware_max = num_cpus::get();
    let final_concurrency = resolve_safe_concurrency(hardware_max, guardian);

    let logs = video_processor::run_concurrent_tasks(tasks, final_concurrency).await;
    Ok(logs.join("\n"))
}

/// 🟢/👑 混合工作流：智能打轴导出
pub async fn run_marker_split(
    params: MarkerSplitParams,
    guardian: &State<'_, SecurityGuardian>, // 👈 必须强制注入内核锁
    session_token: &str                     // 👈 必须强制注入令牌
) -> Result<video_processor::ExportResult, String> {
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
        // 🛡️ 彻底抛弃 params.license_str，走铁穹统一大闸！
        if let Err(_) = auth::check_pro_gate(guardian, session_token) {
            return Err(format!("🚨 免费版单次最多支持导出 5 个片段（当前: {}）。请升级 PRO 旗舰版！", marker_count));
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
    // 🚀 降维打击介入：计算真实安全并发数
    let final_concurrency = resolve_safe_concurrency(params.max_concurrent_tasks, guardian);
    let mut logs = video_processor::run_concurrent_tasks(tasks, final_concurrency).await;
    logs.insert(0, format!("🚀 引擎启动：已分配 {} 个核心进行并发处理...", final_concurrency));

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
pub async fn run_audio_extract(
    params: AudioExtractParams,
    guardian: &State<'_, SecurityGuardian>,
    token: &str
) -> Result<String, String> {
    // 🛡️ 铁穹门禁
    auth::check_pro_gate(guardian, token)?;

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
pub async fn run_format_convert(
    params: FormatConvertParams,
    guardian: &State<'_, SecurityGuardian>,
    token: &str
) -> Result<String, String> {
    // 🛡️ 铁穹门禁
    auth::check_pro_gate(guardian, token)?;

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
pub async fn run_extract_all_audio(
    params: AutoExtractAudioParams,
    guardian: &State<'_, SecurityGuardian> ,// 👈 增加参数
    session_token: &str // 👈 必须接收令牌
) -> Result<String, String> {
    // 🛡️ 铁穹门禁：第一行必须查票！
    auth::check_pro_gate(guardian, session_token)?;
    
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
    // 之前是硬编码 4 线程，现在走智能裁决 (假设前端没有传设置的并发数，默认给最大能力)
    let hardware_max = num_cpus::get();
    let final_concurrency = resolve_safe_concurrency(hardware_max, guardian);

    let logs = video_processor::run_concurrent_tasks(tasks, final_concurrency).await;
    Ok(logs.join("\n"))
}

/// 🚀 自动化脚本 2：一键剥离出纯视频
pub async fn run_export_pure_video(
    params: PureVideoParams,
    guardian: &State<'_, SecurityGuardian>, // 👈 必须强制注入内核锁
    session_token: &str                     // 👈 必须强制注入动态令牌
) -> Result<String, String> {
    // 1. 🛡️ 铁穹大闸：零信任校验，绝杀一切绕过尝试
    auth::check_pro_gate(guardian, session_token)?;

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

    // 2. 🚀 动态降维：获取安全并发数
    // 因为一键剥离单视频只有一个任务，所以请求的并发数 (requested) 传入 1 即可
    let final_concurrency = resolve_safe_concurrency(1, guardian);

    // 3. 执行底层任务
    let logs = video_processor::run_concurrent_tasks(vec![task], final_concurrency).await;
    Ok(logs.join("\n"))
}

// 🛡️ 核心引擎：智能并发裁决器
fn resolve_safe_concurrency(requested: usize, guardian: &State<'_, SecurityGuardian>) -> usize {
    let is_pro = *guardian.is_pro.lock().unwrap();
    let hardware_max = num_cpus::get(); // 获取用户电脑真实的逻辑核心数

    if !is_pro {
        // 免费版降维打击：无论传多少，死锁在 1 个线程
        1
    } else {
        // 旗舰版：尊重用户设置，但绝对不能超过硬件极限，防止死机
        requested.clamp(1, hardware_max)
    }
}