use calamine::{Data, Reader, Xlsx};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

/// 校验失败的单行错误明细（类似于 Laravel Excel `ValidationException` 结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowError {
    /// Excel 中的行号（从 1 开始计数，第 1 行为 Header，数据行从 2 开始）
    pub line: usize,
    /// 触发错误的列标题 / 字段名
    pub column: String,
    /// 错误发生时的单元格原始输入文本
    pub raw_value: String,
    /// 具体的错误原因与校验描述
    pub message: String,
}

/// Excel 导入汇总结果包装
///
/// # 使用方式
///
/// ```ignore
/// // 从 import_from_bytes 获取结果后，先判断是否有错误，再使用有效数据
/// let result: ImportResult<UserImport> = import_from_bytes(&file_bytes)?;
///
/// // 1. 有任何行级错误时，拒绝整批导入（返回 422 错误明细）
/// if result.has_errors() {
///     return Err(Error::BadRequest(
///         serde_json::to_string(&result.errors).unwrap_or_default()
///     ));
/// }
///
/// // 2. 无错误，使用有效行写库
/// for row in &result.valid_rows {
///     User::insert_from_import(&ctx.db, row).await?;
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult<T> {
    /// 成功解析并通过校验的强类型数据行
    pub valid_rows: Vec<T>,
    /// 解析失败或校验不通过的错误集合（按行号与列记录）
    pub errors: Vec<RowError>,
}

impl<T> ImportResult<T> {
    /// 是否存在校验或解析错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Excel 导入接口（仿 Laravel Excel `ToModel` / `WithHeadingRow` / `WithValidation` 规范）
///
/// # 使用方式
///
/// ## 1. 定义导入结构体并实现本 trait
///
/// ```ignore
/// use std::collections::HashMap;
/// use crate::common::excel::import::{ExcelImport, RowError};
///
/// pub struct UserImport {
///     pub name:  String,
///     pub email: String,
///     pub role:  String,
/// }
///
/// impl ExcelImport for UserImport {
///     fn from_row(line: usize, map: &HashMap<String, String>) -> Result<Self, Vec<RowError>> {
///         let mut errs = vec![];
///
///         let name = map.get("姓名").cloned().unwrap_or_default();
///         let email = map.get("邮箱").cloned().unwrap_or_default();
///         let role  = map.get("角色").cloned().unwrap_or_else(|| "user".into());
///
///         if name.is_empty() {
///             errs.push(RowError {
///                 line,
///                 column: "姓名".into(),
///                 raw_value: name.clone(),
///                 message: "姓名不能为空".into(),
///             });
///         }
///         if !email.contains('@') {
///             errs.push(RowError {
///                 line,
///                 column: "邮箱".into(),
///                 raw_value: email.clone(),
///                 message: "邮箱格式不合法".into(),
///             });
///         }
///
///         if errs.is_empty() {
///             Ok(UserImport { name, email, role })
///         } else {
///             Err(errs)
///         }
///     }
///
///     // 可选：指定读取的工作表名称，不设置则读取第一个工作表
///     fn sheet_name() -> Option<String> {
///         Some("用户列表".into())
///     }
/// }
/// ```
///
/// ## 2. 在控制器中接收上传文件并调用 `import_from_bytes`
///
/// ```ignore
/// use crate::common::excel::import::import_from_bytes;
///
/// async fn import_users(
///     State(ctx): State<AppContext>,
///     mut multipart: Multipart,
/// ) -> impl IntoResponse {
///     // 读取 multipart 表单中的文件字节
///     let mut file_bytes = vec![];
///     while let Some(field) = multipart.next_field().await.unwrap() {
///         if field.name() == Some("file") {
///             file_bytes = field.bytes().await.unwrap().to_vec();
///         }
///     }
///
///     let result = import_from_bytes::<UserImport>(&file_bytes)?;
///
///     if result.has_errors() {
///         return format::json(result.errors); // 返回行级错误列表
///     }
///
///     for row in result.valid_rows {
///         User::create_from_import(&ctx.db, &row).await?;
///     }
///
///     format::json(serde_json::json!({ "imported": result.valid_rows.len() }))
/// }
/// ```
pub trait ExcelImport: Sized {
    /// 根据 Header 列名与单元格映射构造数据对象
    ///
    /// # 参数
    /// - `line`: 当前数据在 Excel 中的物理行号（表头为 1，数据行从 2 开始）
    /// - `map`: 列名 -> 单元格文本字符串 的 key-value 映射表
    fn from_row(line: usize, map: &HashMap<String, String>) -> Result<Self, Vec<RowError>>;

    /// 指定要导入的工作表名称（默认读取第一个工作表）
    fn sheet_name() -> Option<String> {
        None
    }
}

/// 将二进制 `.xlsx` 字节流解析为强类型列表与错误集合
///
/// # 使用方式
///
/// ```ignore
/// use crate::common::excel::import::{import_from_bytes, ImportResult};
///
/// // file_bytes: Vec<u8>，通常来自 multipart 上传字段
/// let result: ImportResult<UserImport> = import_from_bytes(&file_bytes)?;
/// ```
///
/// # 错误
///
/// - 文件格式不可识别：返回 `Error::BadRequest`
/// - 内部写入失败：返回 `Error::InternalServerError`
/// - 单行业务校验失败：**不抛错**，写入 `result.errors`，调用方自行处理
pub fn import_from_bytes<T: ExcelImport>(bytes: &[u8]) -> Result<ImportResult<T>, Error> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(cursor).map_err(|e| {
        tracing::error!("打开 Excel 文件字节流失败: {:?}", e);
        Error::BadRequest("无法识别的 Excel 文件格式".to_string())
    })?;

    // 获取目标 Sheet 范围
    let sheet_name = T::sheet_name();
    let range = match sheet_name {
        Some(name) => workbook.worksheet_range(&name).map_err(|e| {
            tracing::error!("未找到指定的 Excel 工作表 '{}': {:?}", name, e);
            Error::BadRequest(format!("未找到工作表: {name}"))
        })?,
        None => {
            let sheet_names = workbook.sheet_names().to_vec();
            let first_sheet = sheet_names
                .first()
                .ok_or_else(|| Error::BadRequest("Excel 文件中不存在任何工作表".to_string()))?;
            workbook.worksheet_range(first_sheet).map_err(|e| {
                tracing::error!("读取工作表失败: {:?}", e);
                Error::InternalServerError
            })?
        }
    };

    let mut rows_iter = range.rows();

    // 1. 读取首行作为表头 Header
    let Some(header_row) = rows_iter.next() else {
        return Ok(ImportResult {
            valid_rows: Vec::new(),
            errors: Vec::new(),
        });
    };

    let headers: Vec<String> = header_row
        .iter()
        .map(|cell| format_cell_value(cell).trim().to_string())
        .collect();

    let mut valid_rows = Vec::new();
    let mut errors = Vec::new();

    // 2. 遍历数据行（第 1 行是 Header，数据行从物理第 2 行开始）
    for (row_idx, row) in rows_iter.enumerate() {
        let line_number = row_idx + 2; // 1-based index: Header is line 1, data starts at line 2

        // 检查是否全空行（calamine Data 无 is_empty 方法，直接匹配 Data::Empty 变体）
        if row.iter().all(|cell| matches!(cell, Data::Empty)) {
            continue;
        }

        let mut row_map = HashMap::new();
        for (header_name, cell) in headers.iter().zip(row.iter()) {
            if !header_name.is_empty() {
                row_map.insert(header_name.clone(), format_cell_value(cell));
            }
        }

        match T::from_row(line_number, &row_map) {
            Ok(item) => valid_rows.push(item),
            Err(mut errs) => errors.append(&mut errs),
        }
    }

    Ok(ImportResult { valid_rows, errors })
}

/// 格式化 `calamine::Data` 为纯文本字符串
fn format_cell_value(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::Bool(b) => b.to_string(),
        // DateTime 在 calamine 中存储为 f64 序号，to_string() 输出浮点数序号
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(dt) => dt.clone(),
        Data::DurationIso(d) => d.clone(),
        Data::Error(e) => format!("ERROR: {e:?}"),
    }
}
