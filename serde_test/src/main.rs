use mongo_odbc_core::col_metadata::SqlGetSchemaResponse;
use bson::{self, doc};

fn main() {
    let get_result_schema_response: Result<SqlGetSchemaResponse, _> = bson::from_document(
        doc!{
            "ok": 1,
            "schema": {
                "version": 1,
                "jsonSchema": {
                    "bsonType": "object",
                    "properties": {
                        "x": { "bsonType": "int"},
                        "y": { "bsonType": "int"},
                    },
                    "required": ["x", "y"],
                    "additionalProperties": false,
                }
            }
        }
    );
    dbg!(get_result_schema_response);
}
