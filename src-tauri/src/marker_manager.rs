use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};
use tokio::fs; // 🌟 核心突破：引入 tokio 异步文件系统，释放阻塞

// ==========================================
// 🚀 V2.0 终极信封数据架构 (The Envelope Pattern)
// ==========================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EasyCutProject {
    pub version: String,
    pub meta: ProjectMeta,
    pub markers: Vec<Marker>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMeta {
    pub project_id: String,
    pub project_name: String,
    pub created_at: u64,
    pub linked_file_hash: Option<String>,
    pub linked_file_name: Option<String>,
    pub source_type: String,
    #[serde(flatten)]
    pub ext: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Marker {
    pub id: String,
    #[serde(rename = "startTime")]
    pub start_time: f64,
    #[serde(rename = "endTime")]
    pub end_time: f64,
    pub label: String,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(flatten)]
    pub payload: HashMap<String, Value>,
}

// ==========================================
// ⚡ 极速基础工具
// ==========================================

pub fn get_json_path(storage_dir: &Path, project_id: &str) -> PathBuf {
    storage_dir.join(format!("{}.json", project_id))
}

/// 极速提取本地视频指纹 (异步升级版：防脱机报错、防阻塞)
pub async fn calculate_video_fingerprint(video_path: &str) -> Result<String, String> {
    let normalized_path = video_path.replace("\\", "/");

    // 🌟 异步探测元数据，不卡顿
    let file_size_str = match fs::metadata(video_path).await {
        Ok(meta) => meta.len().to_string(),
        Err(_) => "OFFLINE_OR_DELETED".to_string(),
    };

    let unique_str = format!("{}_{}", normalized_path, file_size_str);
    let mut hasher = Sha256::new();
    hasher.update(unique_str.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

// ==========================================
// 📦 异步数据读取与平滑迁移
// ==========================================

pub async fn load_project_logic(storage_dir: &Path, video_path: &str) -> Result<EasyCutProject, String> {
    let fingerprint = calculate_video_fingerprint(video_path).await?;
    let json_path = get_json_path(storage_dir, &fingerprint);

    let video_name = Path::new(video_path)
        .file_stem().unwrap_or_default()
        .to_string_lossy().to_string();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // 🌟 异步检测文件存不存在
    if !fs::metadata(&json_path).await.is_ok() {
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

    // 🌟 异步读取大型 JSON 文件
    let json_str = fs::read_to_string(&json_path).await.map_err(|e| e.to_string())?;

    if let Ok(project) = serde_json::from_str::<EasyCutProject>(&json_str) {
        return Ok(project);
    }

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

/// 将项目安全序列化落盘 (异步)
pub async fn save_project_logic(storage_dir: &Path, project: &EasyCutProject) -> Result<(), String> {
    let json_path = get_json_path(storage_dir, &project.meta.project_id);
    let json_string = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;

    // 🌟 异步落盘，彻底解决后台保存时的 UI 掉帧卡顿
    fs::write(json_path, json_string).await.map_err(|e| e.to_string())?;
    Ok(())
}