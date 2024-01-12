use crate::{
    col_metadata::{MongoColMetadata, SqlGetSchemaResponse},
    conn::MongoConnection,
    err::Result,
    stmt::MongoStatement,
    Error, TypeMode,
};
use bson::{doc, document::ValueAccessError, Bson, Document};
use mongodb::{sync::{Collection, Client}, options::AggregateOptions, sync::Cursor};
use std::time::Duration;

#[derive(Debug)]
pub struct MongoQuery {
    // The cursor on the result set.
    resultset_cursor: Option<Cursor<Document>>,
    // The result set metadata, sorted alphabetically by collection and field name.
    resultset_metadata: Vec<MongoColMetadata>,
    // The current deserialized "row".
    current: Option<Document>,
    // The current database
    pub current_db: Option<String>,
    // The query
    pub query: String,
    // The query timeout
    pub query_timeout: Option<u32>,
}

use std::fs::File;
use std::io::{BufWriter, Write};
fn write_url(url: &str) {
     // Create a new file for writing
     let file = File::create("C:\\Logs\\test.txt").unwrap();
        
     // Create a buffered writer to write to the file
     let mut writer = BufWriter::new(file);
        
     // Write some data to the file
     writer.write_all(url.as_bytes()).unwrap();
     writer.write_all(b"Rust is awesome.\n").unwrap();
        
     // Flush the writer to ensure all data is written to disk
     writer.flush().unwrap();
}

impl MongoQuery {
    // Create a MongoQuery with only the resultset_metadata.
    pub fn prepare(
        client: &MongoConnection,
        current_db: Option<String>,
        query_timeout: Option<u32>,
        query: &str,
        type_mode: TypeMode,
    ) -> Result<Self> {
        let current_db = current_db.ok_or(Error::NoDatabase)?;
        let db = client.client.database(&current_db);

        write_url("test1");
        let uri = "mongodb://localhost:27017/test?authMechanism=MONGODB-OIDC&authMechanismProperties=ISSUER_DOMAIN:https://dev-bzkxrnbykc6fb01i.us.auth0.com,CLIENT_ID:80OQwYGwA5JkCFnnQIdcITg3zlOjWfTO,CLIENT_SECRET:-hynidlScgOCoq0FAHreppw-jPRWUzXQ0y9NRJYckF5G6sMfOjZA5B8uvzCenXm0";
        let c = Client::with_uri_str(uri).unwrap();
        let database = c.database("test");
        let my_coll: Collection<Document> = database.collection("array");
        let found = my_coll.find_one(doc! { "x": [] }, None).unwrap();
        println!("Found:\n{:#?}", found);
        write_url("test2");

        // 1. Run the sqlGetResultSchema command to get the result set
        // metadata. Column metadata is sorted alphabetically by table
        // and column name.
        let get_result_schema_cmd =
            doc! {"sqlGetResultSchema": 1, "query": query, "schemaVersion": 1};


        let get_result_schema_response: SqlGetSchemaResponse = bson::from_document(
            db.run_command(get_result_schema_cmd, None)
                .map_err(Error::QueryExecutionFailed)?,
        )
        .map_err(Error::QueryDeserialization)?;

        let metadata =
            get_result_schema_response.process_result_metadata(&current_db, type_mode)?;

        Ok(Self {
            resultset_cursor: None,
            resultset_metadata: metadata,
            current: None,
            current_db: Some(current_db),
            query: query.to_string(),
            query_timeout,
        })
    }
}

impl MongoStatement for MongoQuery {
    // Move the cursor to the next document and update the current row.
    // Return true if moving was successful, false otherwise.
    // This method deserializes the current row and stores it in self.
    fn next(&mut self, _: Option<&MongoConnection>) -> Result<(bool, Vec<Error>)> {
        let res = self
            .resultset_cursor
            .as_mut()
            .map_or(Err(Error::StatementNotExecuted), |c| {
                c.advance().map_err(Error::QueryCursorUpdate)
            });

        // Cursor::advance must return Ok(true) before Cursor::deserialize_current can be invoked.
        // Calling Cursor::deserialize_current after Cursor::advance does not return true or without
        // calling Cursor::advance at all may result in a panic
        if let Ok(true) = res {
            self.current = Some(
                self.resultset_cursor
                    .as_ref()
                    .unwrap()
                    .deserialize_current()
                    .map_err(Error::QueryCursorUpdate)?,
            );
        } else {
            self.current = None;
        }

        Ok((res?, vec![]))
    }

    // Get the BSON value for the cell at the given colIndex on the current row.
    // Fails if the first row as not been retrieved (next must be called at least once before getValue).
    fn get_value(&self, col_index: u16) -> Result<Option<Bson>> {
        let current = self.current.as_ref().ok_or(Error::InvalidCursorState)?;
        let md = self
            .get_col_metadata(col_index)
            .map_err(|_| Error::ColIndexOutOfBounds(col_index))?;
        let datasource = current
            .get_document(&md.table_name)
            .map_err(|e: ValueAccessError| Error::ValueAccess(col_index.to_string(), e))?;
        let column = datasource.get(&md.col_name);
        Ok(column.cloned())
    }

    fn get_resultset_metadata(&self) -> &Vec<MongoColMetadata> {
        &self.resultset_metadata
    }

    // Execute the $sql aggregation for the query and initialize the result set
    // cursor. If there is a timeout, the query must finish before the timeout
    // or an error is returned.
    fn execute(&mut self, connection: &MongoConnection) -> Result<bool> {
        let current_db = self.current_db.as_ref().ok_or(Error::NoDatabase)?;
        let db = connection.client.database(current_db);

        // 2. Run the $sql aggregation to get the result set cursor.
        let pipeline = vec![doc! {"$sql": {
            "statement": &self.query,
        }}];

        let cursor: Cursor<Document> = match self.query_timeout {
            Some(i) => {
                if i > 0 {
                    let opt = AggregateOptions::builder()
                        .max_time(Duration::from_millis(i as u64))
                        .build();
                    db.aggregate(pipeline, opt)
                        .map_err(Error::QueryExecutionFailed)?
                } else {
                    // If the query timeout is 0, it means "no timeout"
                    db.aggregate(pipeline, None)
                        .map_err(Error::QueryExecutionFailed)?
                }
            }
            _ => db
                .aggregate(pipeline, None)
                .map_err(Error::QueryExecutionFailed)?,
        };
        self.resultset_cursor = Some(cursor);
        Ok(true)
    }
}
