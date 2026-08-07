// 初始化 rust-i18n 国际化翻译资源包 (递归扫描读取根目录 locales/ 文件夹，回退语言为 en)
rust_i18n::i18n!("locales", fallback = "en");

pub mod app;
pub mod common;
pub mod controllers;
pub mod data;
pub mod dtos;
pub mod errors;
pub mod initializers;
pub mod mailers;
pub mod models;
pub mod services;
pub mod setup;
pub mod tasks;
pub mod views;
pub mod workers;
