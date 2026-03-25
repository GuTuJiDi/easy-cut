// src-tauri/src/marker_manager.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

// ==========================================
// 🚀 V2.0 终极信封数据架构 (The Envelope Pattern)
// ==========================================

/// 1. 顶层信封：项目 (Project)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EasyCutProject {
    pub version: String,      // 结构版本号，例如 "1.1.0"
    pub meta: ProjectMeta,    // 项目元数据
    pub markers: Vec<Marker>, // 标记列表
}

/// 2. 项目元数据：彻底解耦物理视频，支持跨源打轴
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMeta {
    pub project_id: String,           // 唯一项目 UUID (若是本地视频，当前直接用其特征码)
    pub project_name: String,         // 用户自定义命名 (如："S13总决赛网页切片")
    pub created_at: u64,              // 创建时间戳

    // 🌟 核心突破：解耦物理视频 (若无实体视频则为 None)
    pub linked_file_hash: Option<String>,
    pub linked_file_name: Option<String>,

    pub source_type: String,          // 来源："local_file", "browser_extension", "stopwatch"

    // 🌟 无限扩展槽：存啥都行，后端不校验直接透传
    #[serde(flatten)]
    pub ext: HashMap<String, Value>,
}

/// 3. 标记节点：固化核心，动态扩展外围
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Marker {
    // --- 绝对不可变的核心字段 (FFmpeg 切割强依赖) ---
    pub id: String,
    #[serde(rename = "startTime")]
    pub start_time: f64,
    #[serde(rename = "endTime")]
    pub end_time: f64,
    pub label: String,

    #[serde(default)]
    pub is_deleted: bool, // 逻辑删除标志

    // --- 🌟 动态有效载荷：未来前端新增任何字段（如 color, ai_summary），自动装入此字典 ---
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

// ==========================================
// ⚡ 极速基础工具
// ==========================================

/// 获取项目专属 JSON 路径 (现改为以 project_id 命名)
pub fn get_json_path(storage_dir: &Path, project_id: &str) -> PathBuf {
    storage_dir.join(format!("{}.json", project_id))
}

/// 极速提取本地视频指纹 (仅读元数据，耗时 < 1ms)
pub fn calculate_video_fingerprint(video_path: &str) -> Result<String, String> {
    let metadata = fs::metadata(video_path).map_err(|e| format!("读取元数据失败: {}", e))?;
    let file_size = metadata.len();
    let modified_time = metadata.modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let unique_str = format!("{}_{}_{}", video_path, file_size, modified_time);
    let mut hasher = Sha256::new();
    hasher.update(unique_str.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

// ==========================================
// 📦 数据读取与平滑迁移 (Data Migration)
// ==========================================

/// 加载项目。如果文件不存在，则在内存中初始化一个全新信封；
/// 🌟 包含自动平滑升级旧版 JSON 数组的能力！
pub fn load_project_logic(storage_dir: &Path, video_path: &str) -> Result<EasyCutProject, String> {
    let fingerprint = calculate_video_fingerprint(video_path)?;
    let json_path = get_json_path(storage_dir, &fingerprint);

    let video_name = Path::new(video_path)
        .file_stem().unwrap_or_default()
        .to_string_lossy().to_string();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // 如果是第一次导入，生成全新信封
    if !json_path.exists() {
        return Ok(EasyCutProject {
            version: "1.1.0".to_string(),
            meta: ProjectMeta {
                project_id: fingerprint.clone(),
                project_name: video_name.clone(),
                created_at: now,
                linked_file_hash: Some(fingerprint),
                linked_file_name: Some(video_name),
                source_type: "local_file".to_string(),
                ext: HashMap::new(),
            },
            markers: vec![],
        });
    }

    let json_str = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;

    // 尝试一：按最新 V1.1.0 信封格式解析
    if let Ok(project) = serde_json::from_str::<EasyCutProject>(&json_str) {
        return Ok(project);
    }

    // 尝试二：兼容旧版本，如果是单纯的标记数组，自动包装升级为新信封！
    if let Ok(old_markers) = serde_json::from_str::<Vec<Marker>>(&json_str) {
        return Ok(EasyCutProject {
            version: "1.1.0".to_string(),
            meta: ProjectMeta {
                project_id: fingerprint.clone(),
                project_name: video_name.clone(),
                created_at: now,
                linked_file_hash: Some(fingerprint),
                linked_file_name: Some(video_name),
                source_type: "local_file_migrated".to_string(),
                ext: HashMap::new(),
            },
            markers: old_markers,
        });
    }

    Err("JSON 数据格式损坏，无法解析".to_string())
}

/// 将项目安全序列化落盘
pub fn save_project_logic(storage_dir: &Path, project: &EasyCutProject) -> Result<(), String> {
    let json_path = get_json_path(storage_dir, &project.meta.project_id);
    let json_string = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;
    fs::write(json_path, json_string).map_err(|e| e.to_string())?;
    Ok(())
}