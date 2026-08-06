//! # 多语言国际化 (i18n) 中间件与工具库
//!
//! 该模块实现了基于 `rust-i18n` 的请求级 Locale 解析中间件。
//! 支持从以下渠道按优先级顺序协商提取语言标识：
//! 1. Query 参数 (`?lang=zh-CN` 或 `?locale=zh-CN`)
//! 2. Cookie (`lang=zh-CN`)
//! 3. HTTP 请求头 (`Accept-Language: zh-CN,zh;q=0.9,en;q=0.8`)
//! 4. 默认语言 (`en`)

use axum::{extract::Request, http::header, middleware::Next, response::Response};

/// 通用分割键值对查找函数，零堆内存分配
fn find_kv(raw: &str, delimiter: char, matches: impl Fn(&str) -> bool) -> Option<&str> {
    raw.split(delimiter).find_map(|item| {
        let (k, v) = item.trim().split_once('=')?;
        matches(k).then_some(v)
    })
}

/// 从 URL Query 参数提取语言标识 (?lang= 或 ?locale=)
fn extract_query_lang(req: &Request) -> Option<&str> {
    let query = req.uri().query()?;
    find_kv(query, '&', |k| k == "lang" || k == "locale")
}

/// 从 Cookie 提取语言标识 (lang=)
fn extract_cookie_lang(req: &Request) -> Option<&str> {
    let cookie = req.headers().get(header::COOKIE)?.to_str().ok()?;
    find_kv(cookie, ';', |k| k == "lang")
}

/// 从 Accept-Language 请求头提取首选语言标识
fn extract_header_lang(req: &Request) -> Option<&str> {
    let accept_lang = req.headers().get(header::ACCEPT_LANGUAGE)?.to_str().ok()?;
    accept_lang
        .split(',')
        .map(|part| part.split(';').next().unwrap_or("").trim())
        .find(|lang| !lang.is_empty())
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
