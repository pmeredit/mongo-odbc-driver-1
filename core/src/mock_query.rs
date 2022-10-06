use crate::{
    col_metadata::{ColumnNullability, MongoColMetadata},
    conn::MongoConnection,
    err::Result,
    json_schema::{self, simplified::ObjectSchema, BsonTypeName},
    stmt::MongoStatement,
    Error,
};
use bson::{doc, Bson, Document};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset: Vec<Document>,
    // The result set metadata, sorted alphabetically by collection and field name.
    resultset_metadata: Vec<MongoColMetadata>,
    // The current index in the resultset.
    current: usize,
}

impl MongoQuery {
    pub fn new(resultset: Vec<Document>, resultset_metadata: Vec<MongoColMetadata>) -> Self {
        MongoQuery {
            resultset,
            resultset_metadata,
            current: 0,
        }
    }
}

impl MongoStatement for MongoQuery {
    // Move the current index to the next Document in the Vec.
    // Return true if moving was successful, false otherwise.
    fn next(&mut self) -> Result<bool> {
        self.current += 1;
        if self.current < self.resultset.len() {
            return Ok(true);
        }
        Ok(false)
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        let md = self.get_col_metadata(col_index)?;
        let datasource = self.resultset[self.current]
            .get_document(&md.table_name)
            .map_err(Error::ValueAccess)?;
        let column = datasource.get(&md.col_name);
        Ok(column.cloned())
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &self.resultset_metadata
    }
}
