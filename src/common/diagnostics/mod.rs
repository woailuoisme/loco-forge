//! # 系统健康诊断与指标探针模块 (Health & Diagnostics Subsystem)
//!
//! 提供基础设施组件健康度检测（数据库、缓存、对象存储）及 Kubernetes Liveness/Readiness 探针。

mod hub;

pub use hub::{ComponentHealth, DiagnosticsHub, HealthChecks, ReadinessData};
