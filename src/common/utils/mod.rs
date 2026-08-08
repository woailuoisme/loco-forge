//! # 常用业务工具模块 (Utils Module)
//!
//! 提供高并发防重订单号生成与解析工具 (`OrderUtil`)。
//!
//! > **说明**：密码哈希统一采用 Loco 原生 `loco_rs::prelude::hash` (`hash::hash_password` / `hash::verify_password`)。
//!
//! ## 示例 (Example)
//! ```rust,ignore
//! use locos::common::utils::{OrderUtil, OrderPrefix};
//!
//! // 高并发防重订单号生成
//! let order_no = OrderUtil::generate(OrderPrefix::ORDER, None);
//! let parsed = OrderUtil::parse(&order_no).unwrap();
//! assert_eq!(parsed.prefix, "ORD");
//! ```

pub mod code;
pub mod order_no;
pub mod upload;

pub use code::CodeUtil;
pub use order_no::{OrderNoOptions, OrderPrefix, OrderUtil, ParsedOrderNo};
pub use upload::{NamingStrategy, ProcessedUpload, UploadConfig, UploadUtil};
