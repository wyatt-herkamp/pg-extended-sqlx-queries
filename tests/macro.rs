use chrono::{DateTime, FixedOffset};
use pg_extended_sqlx_queries::{prelude::*, ValueExprType};
use sqlx::prelude::Type;

#[derive(Debug, Clone, TableType)]
#[table(name = "test_table")]
pub struct TestTable {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    #[column(skip)]
    pub age: i32,
    #[column(name = "email_address")]
    pub email: String,
    #[column(name = "phone_number")]
    pub phone: String,
    pub another_table_id: Option<i32>,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}

#[test]
fn test_names() {
    assert_eq!(TestTableColumn::Email.column_name(), "email_address");
    assert_eq!(TestTableColumn::Phone.column_name(), "phone_number");
    assert_eq!(TestTableColumn::CreatedAt.column_name(), "created_at");

    assert_eq!(TestTable::table_name(), "test_table");
}
#[derive(Debug, Clone, TableType)]
#[table(name = "test_table")]
#[column(column_enum_name = CustomColumns)]
pub struct CustomColumnName {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    #[column(skip)]
    pub age: i32,
    #[column(name = "email_address")]
    pub email: String,
    #[column(name = "phone_number")]
    pub phone: String,
    pub another_table_id: Option<i32>,
    pub updated_at: DateTime<FixedOffset>,
    pub created_at: DateTime<FixedOffset>,
}
#[derive(Type, ValueExprType)]
pub enum SqlxTypeTest {
    Variant1,
    Variant2,
}