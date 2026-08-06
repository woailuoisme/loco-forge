use locos::common::excel::{export_to_bytes, spawn_excel_export, CellValue, ExcelExport};

struct TestUserExport {
    id: i64,
    name: String,
    score: f64,
    is_active: bool,
}

impl ExcelExport for TestUserExport {
    fn headings() -> Vec<String> {
        vec!["ID".into(), "姓名".into(), "得分".into(), "是否激活".into()]
    }

    fn row_cells(&self) -> Vec<CellValue> {
        vec![
            self.id.into(),
            self.name.clone().into(),
            self.score.into(),
            self.is_active.into(),
        ]
    }

    fn sheet_name() -> String {
        "测试数据".to_string()
    }
}

use rstest::rstest;

#[rstest]
#[case(CellValue::from("hello"), CellValue::String("hello".to_string()))]
#[case(CellValue::from(42i64), CellValue::Int(42))]
#[case(CellValue::from(3.5f64), CellValue::Float(3.5))]
#[case(CellValue::from(true), CellValue::Bool(true))]
#[case(CellValue::from(Option::<i32>::None), CellValue::Empty)]
#[case(CellValue::from(Some(100i32)), CellValue::Int(100))]
fn test_cell_value_conversions(#[case] actual: CellValue, #[case] expected: CellValue) {
    assert_eq!(actual, expected);
}

#[test]
fn test_excel_export_native_types() {
    let rows = vec![
        TestUserExport {
            id: 1,
            name: "Alice".into(),
            score: 95.5,
            is_active: true,
        },
        TestUserExport {
            id: 2,
            name: "Bob".into(),
            score: 88.0,
            is_active: false,
        },
    ];

    let bytes = export_to_bytes(&rows).expect("Should export excel to bytes");
    assert!(!bytes.is_empty());

    // 校验 PKZIP / XLSX 魔数标识 (50 4B 03 04)
    assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);
}

#[tokio::test]
async fn test_async_spawn_excel_export() {
    let rows = vec![TestUserExport {
        id: 10,
        name: "Charlie".into(),
        score: 99.0,
        is_active: true,
    }];

    let bytes = spawn_excel_export(move || export_to_bytes(&rows))
        .await
        .expect("Async excel export should succeed");

    assert!(!bytes.is_empty());
}
