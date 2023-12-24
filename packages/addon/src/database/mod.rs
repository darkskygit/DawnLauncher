mod classification;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rusqlite::{params, Connection, Result as RusqliteResult};
use serde::{Deserialize, Serialize};

pub use classification::*;

#[napi]
#[derive(Debug)]
pub struct DataSource {
  conn: Connection,
}

const CLASSIFICATION_INIT: &str = "CREATE TABLE IF NOT EXISTS classification (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER,
    name TEXT NOT NULL,
    type INTEGER NOT NULL,
    data TEXT NOT NULL,
    shortcut_key TEXT, 
    global_shortcut_key INTEGER NOT NULL,
    `order` INTEGER NOT NULL
  )";

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

    conn
      .prepare(CLASSIFICATION_INIT)
      .and_then(|mut stmt| stmt.execute([]))
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to init database: {}", e),
        )
      })?;

    Ok(Self { conn })
  }

  #[napi]
  pub fn insert_classification(
    &self,
    parent_id: Option<i64>,
    name: String,
    shortcut_key: Option<String>,
    global_shortcut_key: bool,
    data: String,
    type_: Option<i64>,
  ) -> Result<Option<Classification>> {
    let new_order = self.get_classification_max_order(parent_id)? + 1;
    let insert = "INSERT INTO classification (parent_id, name, type, data, shortcut_key, global_shortcut_key, `order`) VALUES (?, ?, ?, ?, ?, ?, ?)";
    match self.conn.prepare(insert).and_then(|mut stmt| {
      stmt.insert(params![
        parent_id,
        Some(name),
        type_,
        data,
        shortcut_key,
        global_shortcut_key,
        new_order,
      ])
    }) {
      Ok(id) => {
        if let Some(parent_id) = parent_id {
          self
            .conn
            .prepare("UPDATE item SET classification_id = ? WHERE classification_id = ?")
            .and_then(|mut stmt| stmt.execute([id, parent_id]))
            .map_err(|e| {
              Error::new(
                Status::GenericFailure,
                format!("Failed to execute statement: {}", e),
              )
            })?;
        }
        self.get_classification_by_id(id)
      }
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("Failed to execute statement: {}", e),
      )),
    }
  }

  #[napi]
  pub fn update_classification(
    &self,
    id: i64,
    name: String,
    shortcut_key: Option<String>,
    global_shortcut_key: bool,
    data: String,
    type_: Option<i64>,
  ) -> Result<bool> {
    let update = "UPDATE classification SET name = ?, type = ?, data = ?, shortcut_key = ?, global_shortcut_key = ? WHERE id = ?";
    self
      .conn
      .prepare(update)
      .and_then(|mut stmt| {
        stmt.execute(params![
          name,
          type_,
          data,
          shortcut_key,
          global_shortcut_key,
          id
        ])
      })
      .map(|affect| affect > 0)
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to execute statement: {}", e),
        )
      })
  }

  #[napi]
  pub fn get_classification_count(&self) -> Result<i64> {
    self
      .conn
      .prepare("SELECT COUNT(id) FROM classification")
      .and_then(|mut stmt| {
        stmt
          .query_map([], |row| Ok(row.get(0)?))
          .and_then(|mut row| {
            row
              .next()
              .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?
          })
      })
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to query map: {}", e),
        )
      })
  }

  #[napi]
  pub fn get_classification(&self, parent_id: Option<i64>) -> Result<Vec<Classification>> {
    let mut stmt = self
      .conn
      .prepare(&format!(
        "SELECT * FROM classification {} order by `order` asc",
        if let Some(parent_id) = parent_id {
          format!("WHERE parent_id = {}", parent_id)
        } else {
          "".into()
        }
      ))
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

  #[napi]
  pub fn get_classification_by_id(&self, id: i64) -> Result<Option<Classification>> {
    let mut stmt = self
      .conn
      .prepare("SELECT * FROM classification WHERE id = ?")
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to prepare statement: {}", e),
        )
      })?;

    let mut rows = stmt
      .query_map([id], |row| {
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
      .and_then(|row| row.collect::<RusqliteResult<Vec<_>>>())
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to query map: {}", e),
        )
      })?;

    Ok(rows.pop())
  }

  /// 查询最大序号
  #[napi]
  pub fn get_classification_max_order(&self, parent_id: Option<i64>) -> Result<i64> {
    self
      .conn
      .prepare(&format!(
        "SELECT MAX(`order`) FROM classification WHERE parent_id {}",
        if let Some(parent_id) = parent_id {
          format!(" = {}", parent_id)
        } else {
          " IS NULL".to_string()
        }
      ))
      .and_then(|mut stmt| {
        stmt
          .query_map([], |row| Ok(row.get::<_, Option<i64>>(0)?))
          .and_then(|mut row| {
            Ok(
              row
                .next()
                .ok_or(rusqlite::Error::QueryReturnedNoRows)??
                .unwrap_or(0),
            )
          })
      })
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to prepare statement: {}", e),
        )
      })
  }

  #[napi]
  pub fn update_classification_data(&self, id: i64, data: String) -> Result<i64> {
    self
      .conn
      .prepare("UPDATE classification SET data = ? WHERE id = ?")
      .and_then(|mut stmt| stmt.execute([data, id.to_string()]))
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to prepare statement: {}", e),
        )
      })
      .map(|affect: usize| affect as i64)
  }

  #[napi]
  pub fn has_child_classification(&self, parent_id: Option<i64>) -> Result<bool> {
    self.get_classification(parent_id).map(|v| !v.is_empty())
  }

  /// 重新排序
  #[napi]
  pub fn reorder_classification(&self, parent_id: Option<i64>) -> Result<()> {
    let mut stmt = self
      .conn
      .prepare("UPDATE classification SET `order` = ? WHERE id = ?")
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to prepare statement: {}", e),
        )
      })?;

    for (i, classification) in self.get_classification(parent_id)?.iter().enumerate() {
      stmt
        .execute([(i as i32 + 1), classification.id])
        .map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Failed to execute statement: {}", e),
          )
        })?;
    }
    Ok(())
  }
}
