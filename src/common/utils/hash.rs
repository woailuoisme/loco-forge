use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use loco_rs::prelude::*;

/// Laravel 风格的密码哈希门面 (Hash Facade)
///
/// 遵循 OWASP 推荐的安全规范，底层采用 Argon2id 强抗 GPU/ASIC 破解算法，
/// 自动注入高熵随机 Salt 并支持常数时间 (Constant-time) 密码比对校验。
pub struct Hash;

impl Hash {
    /// 为明文密码生成 OWASP 推荐标准的 Argon2id 哈希字符串
    ///
    /// # 示例 (Example)
    /// ```rust,ignore
    /// use locos::common::utils::Hash;
    ///
    /// let hash = Hash::make("secret123")?;
    /// assert!(hash.starts_with("$argon2id$"));
    /// ```
    pub fn make(password: impl AsRef<str>) -> Result<String, Error> {
        let password_bytes = password.as_ref().as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2.hash_password(password_bytes, &salt).map_err(|e| {
            tracing::error!("Argon2id 密码哈希生成失败: {:?}", e);
            Error::InternalServerError
        })?;

        Ok(password_hash.to_string())
    }

    /// 校验明文密码与已知哈希串是否匹配 (常数时间比较，防范时序攻击 Timing Attacks)
    ///
    /// # 示例 (Example)
    /// ```rust,ignore
    /// use locos::common::utils::Hash;
    ///
    /// let is_valid = Hash::check("secret123", &hash_str);
    /// assert!(is_valid);
    /// ```
    pub fn check(password: impl AsRef<str>, hash: impl AsRef<str>) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(hash.as_ref()) else {
            return false;
        };

        Argon2::default()
            .verify_password(password.as_ref().as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// 检查给定的哈希字符串是否需要根据最新的安全参数进行重新哈希升级
    pub fn needs_rehash(hash: impl AsRef<str>) -> bool {
        let Ok(parsed_hash) = PasswordHash::new(hash.as_ref()) else {
            return true;
        };

        parsed_hash.algorithm != argon2::ARGON2ID_IDENT
    }
}
