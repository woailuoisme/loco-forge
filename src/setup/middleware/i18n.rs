//! # 多语言国际化 (i18n) 中间件与工具库
//!
//! 该模块实现了基于 `rust-i18n` 的请求级 Locale 解析中间件。
//! 遵循纯函数式组合设计，按优先级协商匹配语言标识：
//! 1. Query 参数 (`?lang=zh-CN` 或 `?locale=zh-CN`)
//! 2. Cookie (`lang=zh-CN`)
//! 3. HTTP 请求头 (`Accept-Language: zh-CN,zh;q=0.9,en-US;q=0.8`)
//! 4. 默认语言 (`en`)

use axum::{extract::Request, http::header, middleware::Next, response::Response};

/// 系统支持的语言列表 (对应 locales/ 目录下的语言包)
pub const SUPPORTED_LOCALES: &[&str] = &["en", "zh-CN"];

/// 通用键值对查找，零堆内存分配
fn find_kv(raw: &str, delimiter: char, matches: impl Fn(&str) -> bool) -> Option<&str> {
    raw.split(delimiter).find_map(|item| {
        let (k, v) = item.trim().split_once('=')?;
        matches(k).then_some(v)
    })
}

/// 智能匹配系统可用 locale（优先精确匹配，其次主语言前缀降级）
fn match_supported_locale(lang: &str) -> Option<&'static str> {
    let lang = lang.trim();
    if lang.is_empty() {
        return None;
    }
    SUPPORTED_LOCALES
        .iter()
        .find(|&&l| l.eq_ignore_ascii_case(lang))
        .or_else(|| {
            let primary = lang.split(['-', '_']).next()?;
            SUPPORTED_LOCALES.iter().find(|&&l| {
                l.split(['-', '_'])
                    .next()
                    .is_some_and(|p| p.eq_ignore_ascii_case(primary))
            })
        })
        .copied()
}

/// 从 URL Query 参数提取语言标识 (?lang= 或 ?locale=)
fn extract_query_lang(req: &Request) -> Option<&'static str> {
    req.uri()
        .query()
        .and_then(|q| find_kv(q, '&', |k| k == "lang" || k == "locale"))
        .and_then(match_supported_locale)
}

/// 从 Cookie 提取语言标识 (lang=)
fn extract_cookie_lang(req: &Request) -> Option<&'static str> {
    req.headers()
        .get(header::COOKIE)?
        .to_str()
        .ok()
        .and_then(|c| find_kv(c, ';', |k| k == "lang"))
        .and_then(match_supported_locale)
}

/// 从 Accept-Language 请求头协商提取语言标识
fn extract_header_lang(req: &Request) -> Option<&'static str> {
    req.headers()
        .get(header::ACCEPT_LANGUAGE)?
        .to_str()
        .ok()?
        .split(',')
        .find_map(|part| match_supported_locale(part.split(';').next()?))
}

/// HTTP 多语言解析中间件处理函数
pub async fn i18n_middleware(req: Request, next: Next) -> Response {
    let locale = extract_query_lang(&req)
        .or_else(|| extract_cookie_lang(&req))
        .or_else(|| extract_header_lang(&req))
        .unwrap_or("en");

    rust_i18n::set_locale(locale);
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_supported_locale() {
        assert_eq!(match_supported_locale("zh-CN"), Some("zh-CN"));
        assert_eq!(match_supported_locale("en"), Some("en"));
        assert_eq!(match_supported_locale("en-US"), Some("en"));
        assert_eq!(match_supported_locale("en-GB"), Some("en"));
        assert_eq!(match_supported_locale("zh-TW"), Some("zh-CN"));
        assert_eq!(match_supported_locale("zh-HK"), Some("zh-CN"));
        assert_eq!(match_supported_locale("fr-FR"), None);
    }

    #[test]
    fn test_i18n_translation() {
        assert_eq!(
            rust_i18n::t!("errors.unauthorized", locale = "zh-CN"),
            "需要身份验证方可访问该资源。"
        );
        assert_eq!(
            rust_i18n::t!("errors.unauthorized", locale = "en"),
            "Authentication required to access this resource."
        );
    }
}
