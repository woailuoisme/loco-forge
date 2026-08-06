//! # 常用安全与业务工具模块 (Utils Module)
//!
//! 提供仿 Laravel `Hash` 门面密码哈希、高并发防重订单号生成与解析工具 (`OrderUtil`)。
//!
//! ## 示例 (Example)
//! ```rust,ignore
//! use locos::common::utils::{Hash, OrderUtil, OrderPrefix};
//!
//! // 1. 密码哈希
//! let hash = Hash::make("secret123")?;
//! assert!(Hash::check("secret123", &hash));
//!
//! // 2. 高并发防重订单号生成
//! let order_no = OrderUtil::generate(OrderPrefix::ORDER, None);
//! let parsed = OrderUtil::parse(&order_no).unwrap();
//! assert_eq!(parsed.prefix, "ORD");
//! ```

pub mod hash;
pub mod order_no;

pub use hash::Hash;
pub use order_no::{OrderNoOptions, OrderPrefix, OrderUtil, ParsedOrderNo};
