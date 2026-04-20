use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rsa::{pkcs1::DecodeRsaPublicKey, Pkcs1v15Sign, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// 🌟 数据结构：激活码解析后的载荷
#[derive(Serialize, Deserialize, Debug)]
pub struct LicensePayload {
    pub machine_id: String, // 绑定的机器码
    pub expiry: u64,        // 过期时间戳 (秒)
    pub tier: String,       // 授权级别 (如 "PRO")
}

// 👑 你的 RSA 公钥 (测试用，后续替换为你自己生成的)
// 👑 自动生成的纯净公钥 (PKCS#1 格式)
const PUBLIC_KEY_PEM: &str = "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEA1iyiBiqomHVvdNQt/ixLPUS6msdqmOQMnhEotx2RaPDrU7UPodv1\niJ3t1ZPaaoJtPrN/KZ5+bqdMAxzHdwaX8aXTIdj5MhKYO7WsXEslneL4zaWrgNOk\nR4hPxK6lmjE7FtunRgXg5UsREOkICDOKbBEwVFWxX64/SrFG3pW3HTRceYoU4CJ0\nvLDxhed4ffTycmHql2+WjH0kwM9eL7LflJoA89kgA+BDxIU/Gu1tqI1UKE8j2+G4\nqKUQUU0fMK6oV76eehXrRx+JDKgVQiQWaWw6znc6FF4kD9nUmPw4BxX8C2dbzhuf\nJ1xQXONIzsGQSIq3KsgmtxPWVRj45L0nFwIDAQAB\n-----END RSA PUBLIC KEY-----\n";

// ==========================================
// 前端接口层 (Tauri Commands)
// ==========================================

/// 供前端调用：获取当前电脑的唯一机器码
#[tauri::command]
pub fn get_machine_code() -> Result<String, String> {
    match machine_uid::get() {
        Ok(uid) => Ok(uid),
        Err(e) => Err(format!("无法获取机器码: {}", e)),
    }
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
    let payload_json: LicensePayload =
        serde_json::from_slice(&payload_bytes).map_err(|_| "授权数据损坏")?;

    let current_machine_id = machine_uid::get().unwrap_or_default();
    let mut is_machine_match = payload_json.machine_id == current_machine_id;

    // 🌟 开发者测试通道：仅在开发环境(dev)编译生效，发版(build)时自动抹除物理文件！
    #[cfg(debug_assertions)]
    {
        if payload_json.machine_id == "DEV_EASYCUT_TEST_888" {
            is_machine_match = true;
            println!("⚠️ [DEV MODE] 检测到测试专用机器码，已强制放行鉴权通道！");
        }
    }

    if !is_machine_match {
        return Err("机器码不匹配！该激活码属于另一台设备。".to_string());
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