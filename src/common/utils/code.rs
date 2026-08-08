//! 验证码与随机码通用生成工具类 (CodeUtil)
//!
//! 提供企业级 CSPRNG 加密安全伪随机 6 位 OTP 验证码、自定义长度数字码与字母数字混合随机码生成能力。

use rand::RngExt;

/// 企业级通用验证码与随机码生成工具类 (CodeUtil)
pub struct CodeUtil;

impl CodeUtil {
    /// 生成标准 6 位数字 OTP 验证码 (例如: `"049281"`, `"819204"`)
    ///
    /// # 示例 (Example)
    /// ```rust,ignore
    /// use locos::common::utils::CodeUtil;
    ///
    /// let otp = CodeUtil::generate_otp();
    /// assert_eq!(otp.len(), 6);
    /// ```
    pub fn generate_otp() -> String {
        Self::generate_numeric(6)
    }

    /// 生成指定长度 `len` 的纯数字随机验证码 (前导零自动补齐)
    ///
    /// # 参数 (Parameters)
    /// - `len`: 期望生成的数字码长度 (若传入 0 则默认纠正为 6)
    pub fn generate_numeric(len: usize) -> String {
        let length = if len == 0 { 6 } else { len };
        let mut rng = rand::rng();
        let mut result = String::with_capacity(length);
        for _ in 0..length {
            let digit: u8 = rng.random_range(0..10);
            result.push((b'0' + digit) as char);
        }
        result
    }

    /// 生成指定长度 `len` 的字母数字混合随机串 (默认排除易混淆字符 0/O, 1/I/l)
    ///
    /// 可用于生成邀请码、兑换码、临时 Token 等业务随机码。
    pub fn generate_alphanumeric(len: usize) -> String {
        let length = if len == 0 { 8 } else { len };
        const CHARSET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz";
        let mut rng = rand::rng();
        let mut result = String::with_capacity(length);
        for _ in 0..length {
            let idx = rng.random_range(0..CHARSET.len());
            result.push(CHARSET[idx] as char);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_otp() {
        for _ in 0..100 {
            let otp = CodeUtil::generate_otp();
            assert_eq!(otp.len(), 6);
            assert!(otp.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_generate_numeric_custom_length() {
        let code_4 = CodeUtil::generate_numeric(4);
        assert_eq!(code_4.len(), 4);
        assert!(code_4.chars().all(|c| c.is_ascii_digit()));

        let code_8 = CodeUtil::generate_numeric(8);
        assert_eq!(code_8.len(), 8);
        assert!(code_8.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_alphanumeric() {
        let code = CodeUtil::generate_alphanumeric(10);
        assert_eq!(code.len(), 10);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
