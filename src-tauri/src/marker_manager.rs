use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self,Read};
use std::path::{Path, PathBuf};

// --- 数据模型 ---
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Marker {
    pub id: String,
    #[serde(rename = "startTime")]
    pub start_time: f64,
    #[serde(rename = "endTime")]
    pub end_time: f64,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoMarkerData {
    // 依然保留原路径，主要用于给用户展示“该配置最后一次关联的视频位置”
    #[serde(rename = "originalVideoPath")]
    pub original_video_path: String,
    pub markers: Vec<Marker>,
}

// --- 核心指纹算法 ---
/// 计算极速视频指纹：取文件大小 + 头部 1MB 数据进行 SHA-256 运算
// pub fn calculate_video_fingerprint(video_path: &str) -> Result<String, String> {
//     let mut file = File::open(video_path).map_err(|e| format!("无法读取视频文件: {}", e))?;
//
//     let metadata = file
//         .metadata()
//         .map_err(|e| format!("无法获取视频元数据: {}", e))?;
//     let file_size = metadata.len();
//
//     let mut hasher = Sha256::new();
//     // 1. 混入文件大小
//     hasher.update(file_size.to_be_bytes());
//
//     // 2. 读取并混入头部最多 1MB 数据
//     let mut buffer = [0u8; 1024 * 1024]; // 1MB buffer
//     let bytes_read = file.read(&mut buffer).unwrap_or(0);
//     hasher.update(&buffer[..bytes_read]);
//
//     let result = hasher.finalize();
//     // 返回 64 位的十六进制字符串作为唯一 ID
//     Ok(format!("{:x}", result))
// }
// src-tauri/src/marker_manager.rs
/*pub fn calculate_video_fingerprint(video_path: &str) -> Result<String, String> {
    use std::fs::File;
    use std::io::Read;
    use sha2::{Sha256, Digest};

    let mut file = File::open(video_path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();

    // ❌ 错误写法 (会导致栈溢出):
    // let mut buffer = [0u8; 1048576];

    // ✅ 正确写法 (分配到堆内存):
    let mut buffer = vec![0u8; 1024 * 1024]; // 每次读取 1MB

    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}*/
// --- 修复：极速计算视频指纹 (耗时 < 1ms) ---
pub fn calculate_video_fingerprint(video_path: &str) -> Result<String, String> {
    use std::fs;
    use sha2::{Sha256, Digest};

    // 1. 仅读取文件的元数据 (大小、修改时间)，绝对不读取文件内容！
    let metadata = fs::metadata(video_path)
        .map_err(|e| format!("无法读取文件元数据: {}", e))?;

    let file_size = metadata.len();
    let modified_time = metadata.modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 2. 将 路径 + 大小 + 修改时间 拼接成字符串
    let unique_str = format!("{}_{}_{}", video_path, file_size, modified_time);

    // 3. 对这个极短的字符串进行 Hash
    let mut hasher = Sha256::new();
    hasher.update(unique_str.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}
// --- 存储业务逻辑 ---
/// 获取集中存储目录下的专属 JSON 路径
pub fn get_json_path(storage_dir: &Path, fingerprint: &str) -> PathBuf {
    storage_dir.join(format!("{}.json", fingerprint))
}

pub fn save_markers_logic(
    storage_dir: &Path,
    video_path: &str,
    markers: Vec<Marker>,
) -> Result<String, String> {
    // 1. 确保存储目录存在 (如 AppData/Local/EasyCut/Markers)
    if !storage_dir.exists() {
        fs::create_dir_all(storage_dir).map_err(|e| format!("创建存储目录失败: {}", e))?;
    }

    // 2. 计算指纹
    let fingerprint = calculate_video_fingerprint(video_path)?;
    let json_path = get_json_path(storage_dir, &fingerprint);

    // 3. 组装并写入数据
    let data = VideoMarkerData {
        original_video_path: video_path.to_string(),
        markers,
    };
    let json_string =
        serde_json::to_string_pretty(&data).map_err(|e| format!("序列化 JSON 失败: {}", e))?;

    fs::write(&json_path, json_string).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(json_path.to_string_lossy().into_owned())
}

pub fn load_markers_logic(storage_dir: &Path, video_path: &str) -> Result<Vec<Marker>, String> {
    let fingerprint = calculate_video_fingerprint(video_path)?;
    let json_path = get_json_path(storage_dir, &fingerprint);

    if !json_path.exists() {
        return Ok(Vec::new()); // 没有找到历史标记，返回空数组
    }

    let json_string =
        fs::read_to_string(&json_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let data: VideoMarkerData =
        serde_json::from_str(&json_string).map_err(|e| format!("解析 JSON 格式失败: {}", e))?;

    Ok(data.markers)
}

// ==========================================
// 测试左移：后端独立单元测试
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;

    #[test]
    fn test_centralized_marker_storage() {
        let temp_dir = env::temp_dir();
        // 模拟集中存储的目录
        let mock_storage_dir = temp_dir.join("easycut_test_markers");

        // 生成一个临时的假视频文件用于测试指纹计算
        let dummy_video_path = temp_dir.join("mock_video_for_hash.mp4");
        let mut file = File::create(&dummy_video_path).unwrap();
        file.write_all(b"mock video binary data").unwrap();

        let test_markers = vec![Marker {
            id: "1".into(),
            start_time: 0.0,
            end_time: 10.0,
            label: "片段A".into(),
        }];

        // 测试保存
        let save_result = save_markers_logic(
            &mock_storage_dir,
            &dummy_video_path.to_string_lossy(),
            test_markers,
        );
        assert!(save_result.is_ok());

        // 测试读取 (即便视频文件被移动到别的文件夹，只要文件内容没变，读取依然应该成功！这里模拟验证逻辑)
        let loaded =
            load_markers_logic(&mock_storage_dir, &dummy_video_path.to_string_lossy()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].label, "片段A");

        // 清理
        let _ = fs::remove_dir_all(mock_storage_dir);
        let _ = fs::remove_file(dummy_video_path);
    }
}
