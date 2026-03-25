// src-tauri/src/trash_manager.rs
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use crate::config_manager;

/// 将标记的 JSON 文件安全移动到回收站
pub fn move_to_trash(json_path: &Path) -> Result<(), String> {
    if !json_path.exists() {
        return Ok(()); // 文件本来就不存在，视作成功
    }

    let trash_dir = config_manager::get_trash_dir();

    // 获取原文件名，并附加时间戳防止重名覆盖
    let file_name = json_path.file_name()
        .ok_or("无效的文件名")?
        .to_string_lossy();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();

    let trash_file_name = format!("{}_{}", timestamp, file_name);
    let target_path = trash_dir.join(trash_file_name);

    // 使用 fs::rename 实现剪切移动 (同盘极速，跨盘则会自动复制后删除)
    fs::rename(json_path, &target_path)
        .map_err(|e| format!("移动到回收站失败: {}", e))?;

    Ok(())
}

/// 定期清理回收站：删除超过 retention_days 天的文件
pub fn clean_expired_trash(retention_days: u32) -> Result<u32, String> {
    if retention_days == 0 {
        return Ok(0); // 如果设置为0，表示永不自动清理
    }

    let trash_dir = config_manager::get_trash_dir();
    let mut deleted_count = 0;

    // 计算过期的时间阈值
    let retention_duration = Duration::from_secs((retention_days * 24 * 60 * 60) as u64);
    let now = SystemTime::now();

    if let Ok(entries) = fs::read_dir(trash_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified_time) = metadata.modified() {
                        // 如果当前时间 - 修改时间 > 保留期限，则彻底删除
                        if let Ok(age) = now.duration_since(modified_time) {
                            if age > retention_duration {
                                if fs::remove_file(&path).is_ok() {
                                    deleted_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(deleted_count)
}