use crate::{
    api::functions::SQLFetch,
    handles::definitions::{MongoHandle, Statement, StatementState},
};
use bson::doc;
use mongo_odbc_core::{
    col_metadata::{ColumnNullability, MongoColMetadata},
    json_schema::{
        simplified::{Atomic, Schema},
        BsonTypeName,
    },
    mock_query::MongoQuery,
};
use odbc_sys::SqlReturn;
use std::sync::RwLock;

mod unit {
    use odbc_sys::SQLMoreResults;

    use super::*;
    // test unallocated_statement tests SQLFetch when the mongo_statement inside
    // of the statement handle has not been allocated (before an execute or tables function
    // has been called).
    #[test]
    fn unallocated_statement_sql_fetch() {
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(Statement::with_state(
            std::ptr::null_mut(),
            StatementState::Allocated,
        )));

        unsafe {
            assert_eq!(SqlReturn::ERROR, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(
                format!("[MongoDB][API] No ResultSet"),
                format!(
                    "{}",
                    (*stmt_handle)
                        .as_statement()
                        .unwrap()
                        .read()
                        .unwrap()
                        .errors[0]
                ),
            )
        }
    }

    #[test]
    fn sql_fetch_and_more_results_basic_functionality() {
        let mut stmt = Statement::with_state(std::ptr::null_mut(), StatementState::Allocated);
        stmt.mongo_statement = Some(Box::new(MongoQuery::new(
            vec![
                doc! {"a": {"b": 42}},
                doc! {"a": {"b": 43}},
                doc! {"a": {"b": 44}},
            ],
            vec![MongoColMetadata::new(
                "",
                "a".to_string(),
                "b".to_string(),
                Schema::Atomic(Atomic::Scalar(BsonTypeName::Int)),
                ColumnNullability::NoNulls,
            )],
        )));
        let stmt_handle: *mut _ = &mut MongoHandle::Statement(RwLock::new(stmt));
        unsafe {
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(SqlReturn::SUCCESS, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(SqlReturn::NO_DATA, SQLFetch(stmt_handle as *mut _,));
            assert_eq!(SqlReturn::NO_DATA, SQLMoreResults(stmt_handle as *mut _,));
        }
    }
}
