use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// 日志输出 Channel / Driver 模式 (类 Laravel 通用 Channel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LogDriver {
    /// 仅标准控制台输出 (Stdout)
    Console,
    /// 仅按天自动滚动文件输出 (Daily Rolling File)
    Daily,
    /// 单一固定文件输出 (Single File)
    Single,
    /// 结构化 JSON Lines 格式输出 (适合 ELK / Loki / Vector 收集)
    Json,
    /// 组合模式：控制台 + 按天文件同时输出 (Stack)
    #[default]
    Stack,
}

impl fmt::Display for LogDriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Console => write!(f, "console"),
            Self::Daily => write!(f, "daily"),
            Self::Single => write!(f, "single"),
            Self::Json => write!(f, "json"),
            Self::Stack => write!(f, "stack"),
        }
    }
}

impl FromStr for LogDriver {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "console" => Ok(Self::Console),
            "daily" => Ok(Self::Daily),
            "single" => Ok(Self::Single),
            "json" => Ok(Self::Json),
            "stack" => Ok(Self::Stack),
            invalid => Err(format!(
                "Unknown LogDriver: '{invalid}'. Expected 'console', 'daily', 'single', 'json', or 'stack'."
            )),
        }
    }
}

/// Logger 配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// 输出驱动模式 (`console`, `daily`, `stack`)
    #[serde(default)]
    pub driver: LogDriver,

    /// 日志存储目录 (默认: `./storages/logs`)
    #[serde(default = "default_log_dir")]
    pub log_dir: PathBuf,

    /// 文件名前缀 (默认: `loco-`)
    #[serde(default = "default_filename_prefix")]
    pub filename_prefix: String,

    /// 是否开启控制台 ANSI 颜色
    #[serde(default = "default_true")]
    pub enable_ansi: bool,
}

// 从全局常量模块引入/重导出日志默认配置常量
pub use crate::common::consts::logger::{DEFAULT_FILENAME_PREFIX, DEFAULT_LOG_DIR};

fn default_log_dir() -> PathBuf {
    PathBuf::from(DEFAULT_LOG_DIR)
}

fn default_filename_prefix() -> String {
    DEFAULT_FILENAME_PREFIX.to_string()
}

fn default_true() -> bool {
    true
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            driver: LogDriver::Stack,
            log_dir: default_log_dir(),
            filename_prefix: default_filename_prefix(),
            enable_ansi: true,
        }
    }
}

impl LoggerConfig {
    /// 创建一个新的默认 LoggerConfig
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置驱动模式
    pub fn with_driver(mut self, driver: LogDriver) -> Self {
        self.driver = driver;
        self
    }

    /// 设置日志目录
    pub fn with_log_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.log_dir = path.as_ref().to_path_buf();
        self
    }

    /// 设置日志文件前缀
    pub fn with_filename_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.filename_prefix = prefix.into();
        self
    }

    /// 设置 ANSI 颜色开启状态
    pub fn with_ansi(mut self, enable: bool) -> Self {
        self.enable_ansi = enable;
        self
    }

    /// 获取日志目录的 `&Path` 引用
    pub fn log_path(&self) -> &Path {
        &self.log_dir
    }

    /// 确保日志存储目录存在，若不存在则自动创建
    pub fn ensure_log_dir(&self) -> std::io::Result<&Path> {
        if !self.log_dir.exists() {
            std::fs::create_dir_all(&self.log_dir)?;
        }
        Ok(&self.log_dir)
    }
}
