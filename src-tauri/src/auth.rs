use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rsa::{pkcs1::DecodeRsaPublicKey, Pkcs1v15Sign, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{State, Manager};
use std::sync::Mutex;
use uuid::Uuid;
use keyring::Entry; // 🌟 引入系统密钥链
use std::fs;
use obfstr::obfstr;
// 🌟 新增
use sysinfo::System;
use crate::config_manager; // 🌟 新增
// 🌟 数据结构：激活码解析后的载荷
// 🌟 1. 升级激活码载荷结构 (兼容老版本)
#[derive(Serialize, Deserialize, Debug)]
pub struct LicensePayload {
    pub machine_id: String, // 留作向下兼容老版本的单机验证

    #[serde(default)]
    pub hw_components: Vec<String>, // 🌟 新增：独立硬件指纹库 (存储 CPU、主板、硬盘的独立哈希)

    pub expiry: u64, //过期时间戳
    pub tier: String, //授权级别
}
// 🌟 铁穹架构核心：全局安全守护者
pub struct SecurityGuardian {
    pub is_pro: Mutex<bool>,
    pub session_token: Mutex<String>,
    pub machine_id: String,
    pub hub_port: Mutex<u16>, // 🌟 新增：记录当前分配的动态端口
    // 🌟 新增：防暴破状态锁
    pub failed_attempts: Mutex<u8>,
    pub lockout_until: Mutex<u64>,
}
// 👑 你的 RSA 公钥 (测试用，后续替换为你自己生成的)
// 👑 自动生成的纯净公钥 (PKCS#1 格式)
const PUBLIC_KEY_PEM: &str = "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEA1iyiBiqomHVvdNQt/ixLPUS6msdqmOQMnhEotx2RaPDrU7UPodv1\niJ3t1ZPaaoJtPrN/KZ5+bqdMAxzHdwaX8aXTIdj5MhKYO7WsXEslneL4zaWrgNOk\nR4hPxK6lmjE7FtunRgXg5UsREOkICDOKbBEwVFWxX64/SrFG3pW3HTRceYoU4CJ0\nvLDxhed4ffTycmHql2+WjH0kwM9eL7LflJoA89kgA+BDxIU/Gu1tqI1UKE8j2+G4\nqKUQUU0fMK6oV76eehXrRx+JDKgVQiQWaWw6znc6FF4kD9nUmPw4BxX8C2dbzhuf\nJ1xQXONIzsGQSIq3KsgmtxPWVRj45L0nFwIDAQAB\n-----END RSA PUBLIC KEY-----\n";
// ✅ 铁穹级防御：编译期混淆公钥
pub fn get_public_key() -> String {
    // 黑客无法在 .exe 中搜到这段连续的文本
    obfstr!("-----BEGIN RSA PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAy...[你的公钥内容]...\n-----END RSA PUBLIC KEY-----").to_string()
}

// 定义我们在系统凭据管理器中的标识符
const KEYRING_SERVICE: &str = "easy_cut_pro_license";
const KEYRING_USER: &str = "current_user";

// ✅ 商业发版标准命名
// 加入你们的域名或反向公司名，避免系统凭证冲突
// const KEYRING_SERVICE: &str = "com.yourcompany.easycut.license_v1";

// ✅ 动态用户标识 (极高安全性)
// 不要用写死的 current_user，改为使用当前操作系统的登录用户名。
// 这样即使黑客在一台电脑上用两个不同的 Windows 账号，他们的 PRO 状态也是隔离的！
pub fn get_keyring_user() -> String {
    // 自动获取当前操作系统用户名，如果获取失败则降级
    std::env::var("USERNAME") // Windows
        .or_else(|_| std::env::var("USER")) // macOS/Linux
        .unwrap_or_else(|_| "default_user".to_string())
}

// 在存取时：
// let entry = keyring::Entry::new(KEYRING_SERVICE, &get_keyring_user()).unwrap();
// ==========================================
// 前端接口层 (Tauri Commands)
// ==========================================

/// 供前端调用：获取当前电脑的唯一机器码
/// 供前端调用的机器码生成接口（用于用户发给你们生成激活码）
#[tauri::command]
pub fn get_machine_code() -> Result<String, String> {
    // 🌟 降维打击：将数组序列化并转为 Base64，生成一段类似 "Hw1-Hw2-Hw3" 的代码
    let fingerprints = get_current_hardware_fingerprints();
    let combined = fingerprints.join("|");
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    Ok(BASE64.encode(combined.as_bytes()))
}

/// 供前端调用：UI 层的激活验证
#[tauri::command]
pub fn verify_license_cmd(license_str: String) -> Result<bool, String> {
    // 复用底层核心逻辑，只返回 true/false 给前端
    match verify_license_internal(&license_str) {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

// ==========================================
// 后端拦截层 (Internal Core Logic)
// ==========================================

/// 核心大闸：供 Rust 后端其他核心模块调用的验证函数
/// 返回解析后的 Payload，以便后续判断是 PRO 还是 Ultimate 版本
pub fn verify_license_internal(license_str: &str) -> Result<LicensePayload, String> {
    // 1. 拆分激活码 (格式: PayloadBase64.SignatureBase64)
    let parts: Vec<&str> = license_str.split('.').collect();
    if parts.len() != 2 {
        return Err("激活码格式错误 (非法篡改)".to_string());
    }

    let payload_b64 = parts[0];
    let signature_b64 = parts[1];

    // 2. Base64 解码
    let payload_bytes = BASE64.decode(payload_b64).map_err(|_| "载荷解析失败")?;
    let signature_bytes = BASE64.decode(signature_b64).map_err(|_| "签名解析失败")?;

    // 3. 验证机器码是否匹配当前电脑 (防止一码多用)
    /* let payload_json: LicensePayload =
         serde_json::from_slice(&payload_bytes).map_err(|_| "授权数据损坏")?;
 */    
    /*let current_machine_id = machine_uid::get().unwrap_or_default();
    if payload_json.machine_id != current_machine_id {
        return Err("机器码不匹配！该激活码属于另一台设备。".to_string());
    }*/

    // 机器码测试开始
    // 3. 验证机器码是否匹配当前电脑 (防一码多用)
    // 3. 验证机器码是否匹配当前电脑 (防止一码多用)
    // 假设 RSA 校验已通过，解析出了 payload_json
    let payload_json: LicensePayload = serde_json::from_slice(&payload_bytes).map_err(|_| "授权数据损坏")?;
    // ==========================================
    // 🛡️ 铁穹架构 V2.0：模糊匹配硬件指纹 (Fuzzy Hardware Matching)
    // ==========================================
    let mut is_machine_match = false;

    // 🌟 1. 优先尝试模糊匹配 (针对新版激活码)
    if !payload_json.hw_components.is_empty() {
        let current_fingerprints = get_current_hardware_fingerprints();
        let mut match_count = 0;

        for current_hash in &current_fingerprints {
            if payload_json.hw_components.contains(current_hash) {
                match_count += 1;
            }
        }

        // 🎯 核心裁决：3 个硬件中，只要有 2 个对得上（比如主板和 CPU 没变，只换了硬盘），就放行！
        let required_matches = 2;

        if match_count >= required_matches {
            is_machine_match = true;
            println!("🛡️ 硬件指纹模糊匹配成功！契合度: {}/3", match_count);
        } else {
            println!("⚠️ 硬件发生重大变更！匹配度 ({}/3) 低于安全阈值。", match_count);
        }
    }
    // 🌟 2. 兼容老版本单机码逻辑 (Backward Compatibility)
    else {
        let current_machine_id = machine_uid::get().unwrap_or_default();
        if payload_json.machine_id == current_machine_id {
            is_machine_match = true;
            println!("🛡️ 传统单机码匹配成功！");
        }
    }

    // 🌟 3. 开发者应急通道
    #[cfg(debug_assertions)]
    {
        if payload_json.machine_id == "DEV_EASYCUT_TEST_888" {
            is_machine_match = true;
            println!("⚠️ [DEV MODE] 检测到测试专用通道，强行放行！");
        }
    }

    // 4. 最终裁决
    if !is_machine_match {
        return Err("🚨 硬件特征严重不符！系统判定该激活码属于另一台设备，或核心硬件已被大面积更换。".to_string());
    }
    // 机器码测试结束
    // 4. 验证是否过期
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if current_time > payload_json.expiry {
        return Err("授权已过期，请续费。".to_string());
    }

    // 5. 👑 终极防御：验证 RSA 数字签名
    let public_key = RsaPublicKey::from_pkcs1_pem(PUBLIC_KEY_PEM)
        .map_err(|_| "内部公钥错误")?;

    /*let mut hasher = Sha256::new();
    hasher.update(&payload_bytes);
    let hashed = hasher.finalize();
    match public_key.verify(Pkcs1v15Sign::new_unprefixed(), &hashed, &signature_bytes) {
        Ok(_) => Ok(payload_json), // 验证通过，返回载荷
        Err(_) => Err("数字签名无效！检测到盗版或非法凭证！".to_string()),
    }*/
    let mut hasher = Sha256::new();
    hasher.update(&payload_bytes);
    let hashed = hasher.finalize();

    // 🌟 核心修复：使用标准的带有 Sha256 前缀的验证模式！
    match public_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed, &signature_bytes) {
        Ok(_) => Ok(payload_json), // 验证通过，返回载荷
        Err(_) => Err("数字签名无效！检测到盗版或非法凭证！".to_string()),
    }
}

impl SecurityGuardian {
    pub fn new() -> Self {
        Self {
            is_pro: Mutex::new(false),
            session_token: Mutex::new(String::new()),
            machine_id: machine_uid::get().unwrap_or_default(),
            hub_port: Mutex::new(0),
            failed_attempts: Mutex::new(0),
            lockout_until: Mutex::new(0),
        }
    }
}

// 🌟 重构：初始化安全会话 (每次启动生成唯一令牌)
// 🌟 修改 1：静默初始化 (应用启动时调用)
#[tauri::command]
pub async fn init_security_session(guardian: State<'_, SecurityGuardian>) -> Result<String, String> {
    // 1. 生成动态令牌
    let mut token = guardian.session_token.lock().unwrap();
    // 🛡️ 核心修复：如果令牌已存在，说明是子窗口（悬浮球）在初始化，直接返回现有令牌，阻止覆盖！
    if !token.is_empty() {
        println!("🛡️ 铁穹系统：检测到子窗口请求，复用当前会话令牌。");
        return Ok(token.clone());
    }
    *token = Uuid::new_v4().to_string();
    // 🌟 核心：在此刻把端口和 Token 写出去！
    let current_port = *guardian.hub_port.lock().unwrap();
    if current_port != 0 {
        write_connection_lockfile(current_port, &token);
    }

    // 2. 尝试从操作系统底层捞取激活码
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|_| "密钥链访问异常")?;

    if let Ok(saved_license) = entry.get_password() {
        // 3. 拿到激活码，送去进行 RSA 和 机器码 的严格校验
        if verify_license_internal(&saved_license).is_ok() {
            let mut pro_status = guardian.is_pro.lock().unwrap();
            *pro_status = true;
            println!("🛡️ 铁穹系统：从系统密钥链静默恢复 PRO 授权成功！");
        } else {
            println!("⚠️ 铁穹系统：系统密钥链中的凭证已失效或机器码不符。");
            // 可选：如果发现凭证无效，直接从系统删除它
            let _ = entry.delete_credential();
        }
    } else {
        println!("🛡️ 铁穹系统：未在系统中发现授权凭证，以免费版模式启动。");
    }

    Ok(token.clone())
}
// 🌟 重构：带有状态机更新的鉴权指令
#[tauri::command]
pub fn verify_license_and_activate(
    license_str: String,
    guardian: tauri::State<'_, SecurityGuardian>
) -> Result<bool, String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // 1. 检查是否在锁定惩罚期
    let lockout = *guardian.lockout_until.lock().unwrap();
    if now < lockout {
        return Err(format!("🚨 尝试次数过多，安全系统已锁定！请在 {} 秒后再试。", lockout - now));
    }

    match verify_license_internal(&license_str) {
        Ok(_) => {
            // 验证通过，重置错误次数并激活
            *guardian.failed_attempts.lock().unwrap() = 0;
            let mut pro_status = guardian.is_pro.lock().unwrap();
            *pro_status = true;

            let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
                .map_err(|e| format!("系统密钥链初始化失败: {}", e))?;
            entry.set_password(&license_str)
                .map_err(|e| format!("写入系统密钥链失败: {}", e))?;

            Ok(true)
        },
        Err(e) => {
            // 🌟 防御：记录失败次数，超过 5 次锁定 60 秒
            let mut attempts = guardian.failed_attempts.lock().unwrap();
            *attempts += 1;
            if *attempts >= 5 {
                *guardian.lockout_until.lock().unwrap() = now + 60; // 锁定 60 秒
                *attempts = 0; // 重置计数器，等待下个周期
                return Err("🚨 连续 5 次输入非法凭证，系统已强制锁定 60 秒！".to_string());
            }
            Err(e)
        }
    }
}

// 🌟 新增：零信任检查中间件函数 (供 Rust 内部其他模块调用)
pub fn check_pro_gate(guardian: &State<'_, SecurityGuardian>, client_token: &str) -> Result<(), String> {
    // 1. 验证“工牌”（防止外挂网页强行调用 API）
    let server_token = guardian.session_token.lock().unwrap();
    if client_token != *server_token || server_token.is_empty() {
        return Err("🚨 拒绝访问：非法或过期的会话令牌！".to_string());
    }
    // 2. 验证“级别”（防止前端修改 Vue 内存）
    // 🌟 这里是绝对真理！我们根本不看前端传来的 isPro 参数，我们只查 Rust 内存！
    let is_pro_memory = *guardian.is_pro.lock().unwrap();
    if !is_pro_memory {
        return Err("🚨 拒绝访问：该功能需要 PRO 旗舰版授权！检测到前端环境与内核状态不符。".to_string());
    }
    Ok(())
}

// 新增一个只允许拿着合法 Token 来查询状态的安全接口
#[tauri::command]
pub fn get_pro_status_safe(
    guardian: State<'_, SecurityGuardian>,
    session_token: String
) -> bool {
    let server_token = guardian.session_token.lock().unwrap();
    if session_token == *server_token {
        let is_pro = guardian.is_pro.lock().unwrap();
        *is_pro
    } else {
        false // 乱问直接装死
    }
}

// 🌟 新增：安全注销指令，物理删除系统凭据
// 🌟 修复：安全注销指令，严格处理系统凭据的物理删除
#[tauri::command]
pub fn deactivate_license_safe(
    guardian: tauri::State<'_, SecurityGuardian>,
    session_token: String
) -> Result<(), String> {
    // 1. 验证动态令牌
    let server_token = guardian.session_token.lock().unwrap();
    if session_token != *server_token {
        return Err("非法越权操作".to_string());
    }

    // 2. 物理删除系统 Keyring 存储
    let entry = keyring::Entry::new(crate::auth::KEYRING_SERVICE, crate::auth::KEYRING_USER)
        .map_err(|e| format!("凭证库初始化异常: {}", e))?;

    // 🌟 核心修复：使用 Rust 标准的模式匹配，抛弃脆弱的硬编码字符串匹配
    match entry.delete_credential() {
        Ok(_) => {
            log::info!("✅ 凭证从系统中物理删除成功");
        },
        Err(keyring::Error::NoEntry) => {
            // keyring 库官方定义的 "未找到凭证" 枚举，完美跨平台，直接放行
            // 标准未找到错误，直接放行 (保证幂等性)
            log::info!("⚠️ 凭证本就不存在，视作删除成功");
        },
        Err(e) => {
            // 跨平台兜底：转换为小写进行模糊匹配，兼容中英文及不同操作系统的“未找到”提示
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("could not be found")
                || err_str.contains("element not found")
                || err_str.contains("找不到")
                || err_str.contains("no such entry") {

                log::info!("⚠️ 凭证不存在 (OS底层拦截)，视作删除成功");
            } else {
                // 真正的权限不足或系统 I/O 报错，必须拦截并抛出给前端
                return Err(format!("设备解绑失败！可能是操作系统权限不足，请尝试以管理员身份运行软件重试。底层错误：{}", e));
            }
        }
    }

    // 3. 物理删除成功后，安全翻转内存状态
    let mut pro_status = guardian.is_pro.lock().unwrap();
    *pro_status = false;

    Ok(())
}
fn get_secure_lockfile_path() -> std::path::PathBuf {
    // 🌟 写入系统临时目录，如 Windows 的 AppData/Local/Temp
    std::env::temp_dir().join(".easycut_hub.lock")
}

fn write_connection_lockfile(port: u16, token: &str) {
    let lockfile_path = get_secure_lockfile_path();
    let payload = serde_json::json!({
        "port": port,
        "token": token
    });
    let _ = fs::write(lockfile_path, payload.to_string());
}
// 🌟 新增辅助函数：将端口和 Token 写为本地隐形信标文件
/*fn write_connection_lockfile(port: u16, token: &str) {
    let workspace = config_manager::get_workspace_dir();
    let lockfile_path = workspace.join(".easycut_connection.json"); // 隐藏文件

    let payload = serde_json::json!({
        "port": port,
        "token": token
    });

    // 写入文件。未来插件只需读取此文件即可知道往哪个端口发数据、带什么 Token
    let _ = fs::write(lockfile_path, payload.to_string());
}*/

// 🌟 新增供插件或主程序调用的“阅后即焚”销毁指令
#[tauri::command]
pub fn consume_hub_lockfile() -> Result<(), String> {
    let lockfile_path = get_secure_lockfile_path();
    if lockfile_path.exists() {
        std::fs::remove_file(lockfile_path).map_err(|e| e.to_string())?;
        println!("🔥 信标文件已被安全焚毁 (阅后即焚)");
    }
    Ok(())
}

// 🌟 2. 硬件探针：获取并计算当前机器的独立硬件指纹
pub fn get_current_hardware_fingerprints() -> Vec<String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut components = Vec::new();

    // A. 提取 CPU 信息 (型号 + 核心数作为基础特征)
    let cpus = sys.cpus();
    if !cpus.is_empty() {
        let cpu_info = format!("{}_{}", cpus[0].brand(), cpus.len());
        components.push(hash_string(&cpu_info));
    }

    // B. 提取主板/主机名信息 (跨平台稳定标识)
    let host_name = System::host_name().unwrap_or_else(|| "unknown_host".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "unknown_os".to_string());
    components.push(hash_string(&format!("{}_{}", host_name, os_version)));

    // C. 提取 Mac 地址或底层 UUID (这里使用系统底层标识符作为第三维度)
    // 注意：如果有条件，你可以使用 wmic csproduct get uuid 获取更底层的主板 UUID
    let base_uuid = machine_uid::get().unwrap_or_else(|_| "unknown_uuid".to_string());
    components.push(hash_string(&base_uuid));

    components
}

// 辅助函数：将字符串计算为 SHA256 Hex
fn hash_string(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}