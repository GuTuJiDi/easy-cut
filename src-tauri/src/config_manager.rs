// src-tauri/src/config_manager.rs
use std::env;
use std::fs;
use std::path::PathBuf;

/// 获取易剪的全局工作区根目录
pub fn get_workspace_dir() -> PathBuf {
    // 策略 1：便携模式检查。尝试在 .exe 同级目录寻找或创建 "EasyCut_Data"
    if let Ok(mut exe_path) = env::current_exe() {
        exe_path.pop(); // 退到 exe 所在目录
        let portable_dir = exe_path.join("EasyCut_Data");

        // 如果能成功创建或已存在，说明有权限，启用便携模式
        if fs::create_dir_all(&portable_dir).is_ok() {
            return portable_dir;
        }
    }

    // 策略 2：安全回退模式。放到系统标准的 "我的文档 (Documents)/EasyCut_Data" 中
    let docs_dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from("C:\\"));
    let workspace_dir = docs_dir.join("EasyCut_Data");

    let _ = fs::create_dir_all(&workspace_dir);
    workspace_dir
}

/// 获取标记专属存储目录
pub fn get_markers_dir() -> PathBuf {
    let dir = get_workspace_dir().join("Markers");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// 获取配置专属存储目录
pub fn get_config_dir() -> PathBuf {
    let dir = get_workspace_dir().join("Config");
    let _ = fs::create_dir_all(&dir);
    dir
}

// src-tauri/src/config_manager.rs
// ... 保持原有的 get_workspace_dir, get_markers_dir, get_config_dir 不变 ...

/// 获取/初始化 Trash (回收站) 存放目录
pub fn get_trash_dir() -> PathBuf {
    let dir = get_workspace_dir().join("Trash");
    let _ = fs::create_dir_all(&dir);
    dir
}