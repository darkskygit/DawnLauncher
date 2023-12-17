mod classification;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rusqlite::{Connection, Result as RusqliteResult};
use serde::{Deserialize, Serialize};

pub use classification::*;

#[napi]
#[derive(Debug)]
pub struct DataSource {
    conn: Connection,
}

#[napi]
impl DataSource {
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        let conn = Connection::open(path).map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to open database: {}", e),
            )
        })?;
        Ok(Self { conn })
    }

    #[napi]
    pub fn get_classification(&self, parent_id: Option<i64>) -> Result<Vec<Classification>> {
        let mut stmt = if let Some(parent_id) = parent_id {
            self.conn.prepare(&format!(
                "SELECT * FROM classification WHERE parent_id = {parent_id} order by `order` asc",
            ))
        } else {
            self.conn
                .prepare("SELECT * FROM classification order by `order` asc")
        }
        .map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to prepare statement: {}", e),
            )
        })?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Classification {
                    id: row.get(0)?,
                    parent_id: row.get(1)?,
                    name: row.get(2)?,
                    type_: row.get(3)?,
                    data: row.get(4)?,
                    shortcut_key: row.get(5)?,
                    global_shortcut_key: row.get(6)?,
                    order: row.get(7)?,
                })
            })
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Failed to query map: {}", e),
                )
            })?;

        Ok(rows.collect::<RusqliteResult<Vec<_>>>().map_err(|e| {
            Error::new(
                Status::GenericFailure,
                format!("Failed to collect rows: {}", e),
            )
        })?)
    }
}
