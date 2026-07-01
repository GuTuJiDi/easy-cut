use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use crate::config_manager;

/// 将标记的 JSON 文件安全移动到回收站 (异步处理版)
pub async fn move_to_trash(json_path: &Path) -> Result<(), String> {
    // 🌟 异步判断
    if !tokio::fs::metadata(json_path).await.is_ok() {
        return Ok(()); // 文件本来就不存在，视作成功
    }

    let trash_dir = config_manager::get_trash_dir();

    let file_name = json_path.file_name()
        .ok_or("无效的文件名")?
        .to_string_lossy();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();

    let trash_file_name = format!("{}_{}", timestamp, file_name);
    let target_path = trash_dir.join(trash_file_name);

    // 🌟 异步移动
    tokio::fs::rename(json_path, &target_path)
        .await
        .map_err(|e| format!("移动到回收站失败: {}", e))?;

    Ok(())
}

/// 定期清理回收站：删除超过 retention_days 天的文件
/// (保持同步，因为仅在应用刚启动 `main` 函数准备期间执行，不会阻碍事件循环)
pub fn clean_expired_trash(retention_days: u32) -> Result<u32, String> {
    use std::fs;

    if retention_days == 0 {
        return Ok(0);
    }

    let trash_dir = config_manager::get_trash_dir();
    let mut deleted_count = 0;
    let retention_duration = Duration::from_secs((retention_days * 24 * 60 * 60) as u64);
    let now = SystemTime::now();

    if let Ok(entries) = fs::read_dir(trash_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified_time) = metadata.modified() {
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